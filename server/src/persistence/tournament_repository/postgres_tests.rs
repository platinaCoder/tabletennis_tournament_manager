use chrono::Utc;
use sqlx::query::query;
use sqlx::row::Row;
use sqlx_postgres::PgPoolOptions;

use crate::api_contract::TournamentAccessRole;
use crate::application::TournamentEntrant;
use crate::backend::auth::AuthenticatedUser;
use crate::backend::auth::UserId;
use crate::backend::persistence::migrate_test_database;
use crate::identity::{ClubId, EntrantId};
use crate::pairing::EloRating;
use crate::pairing::algorithms::blossom_v2::BlossomV2Policy;
use crate::results::{GameScore, MatchFormat};
use crate::tournament::{MaximumRoundCount, TableCount, TournamentId};

use super::{NewTournament, TournamentRepository};

async fn insert_user(pool: &sqlx_postgres::PgPool, user_id: UserId, email: &str) {
    let now = Utc::now();
    query::<sqlx_postgres::Postgres>(
        "INSERT INTO users (
            id, email, created_at, updated_at, last_login_at
         ) VALUES ($1, $2, $3, $3, $3)",
    )
    .bind(user_id.as_uuid())
    .bind(email)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
#[ignore = "requires an isolated TEST_DATABASE_URL PostgreSQL database"]
async fn tournament_entrants_rounds_matches_and_games_round_trip() {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .unwrap();
    migrate_test_database(&pool).await.unwrap();
    let user_id = UserId::new();
    let now = Utc::now();
    query::<sqlx_postgres::Postgres>(
        "INSERT INTO users (
            id, email, created_at, updated_at, last_login_at
         ) VALUES ($1, $2, $3, $3, $3)",
    )
    .bind(user_id.as_uuid())
    .bind(format!("{}@test.invalid", user_id.as_uuid()))
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    let repository = TournamentRepository::new(pool.clone());
    let mut stored = repository
        .create(
            user_id,
            NewTournament {
                title: TournamentId::new(format!("test-{}", uuid::Uuid::new_v4())),
                match_format: MatchFormat::BestOfThree,
                table_count: TableCount::try_from(2).unwrap(),
                maximum_round_count: MaximumRoundCount::try_from(2).unwrap(),
            },
            now,
        )
        .await
        .unwrap();
    for index in 0..4 {
        stored
            .application
            .register_entrant(TournamentEntrant {
                entrant_id: EntrantId::new(format!("entrant-{index}")),
                name: format!("Entrant {index}"),
                club_id: ClubId::new(format!("club-{index}")),
                club_name: format!("Club {index}"),
                starting_elo: EloRating::new(1_000 + index * 100),
            })
            .unwrap();
    }
    stored.application.start_tournament().unwrap();
    stored
        .application
        .calculate_pairings(BlossomV2Policy::default())
        .unwrap();
    stored.application.publish_pairings().unwrap();
    let match_ids = stored
        .application
        .active_round()
        .unwrap()
        .scheduled_matches
        .iter()
        .map(|scheduled| scheduled.match_id.clone())
        .collect::<Vec<_>>();
    for match_id in match_ids {
        stored
            .application
            .enter_match_result(
                &match_id,
                vec![
                    GameScore::new(1, 11, 7).unwrap(),
                    GameScore::new(2, 12, 10).unwrap(),
                ],
            )
            .unwrap();
    }
    stored.application.complete_round().unwrap();
    stored.revision = repository.save(&stored, 0, Utc::now()).await.unwrap();

    let loaded = repository.load(stored.id).await.unwrap().unwrap();
    assert_eq!(loaded.application.snapshot(), stored.application.snapshot());
    assert_eq!(loaded.revision, 1);
    assert_eq!(loaded.application.completed_rounds()[0].results.len(), 2);
    assert_eq!(
        loaded.application.completed_rounds()[0].results[0].games()[1]
            .home_points
            .value(),
        12
    );

    query::<sqlx_postgres::Postgres>("DELETE FROM tournaments WHERE id = $1")
        .bind(stored.id)
        .execute(&pool)
        .await
        .unwrap();
    query::<sqlx_postgres::Postgres>("DELETE FROM users WHERE id = $1")
        .bind(user_id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires an isolated TEST_DATABASE_URL PostgreSQL database"]
async fn sharing_invitations_require_one_time_recipient_decisions() {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .unwrap();
    migrate_test_database(&pool).await.unwrap();
    let owner_id = UserId::new();
    let editor_id = UserId::new();
    let viewer_id = UserId::new();
    let owner_email = format!("owner-{}@test.invalid", owner_id.as_uuid());
    let editor_email = format!("editor-{}@test.invalid", editor_id.as_uuid());
    let viewer_email = format!("viewer-{}@test.invalid", viewer_id.as_uuid());
    insert_user(&pool, owner_id, &owner_email).await;
    insert_user(&pool, editor_id, &editor_email).await;
    let repository = TournamentRepository::new(pool.clone());
    let stored = repository
        .create(
            owner_id,
            NewTournament {
                title: TournamentId::new(format!("sharing-{}", uuid::Uuid::new_v4())),
                match_format: MatchFormat::BestOfThree,
                table_count: TableCount::try_from(2).unwrap(),
                maximum_round_count: MaximumRoundCount::try_from(2).unwrap(),
            },
            Utc::now(),
        )
        .await
        .unwrap();
    repository
        .grant_access(
            stored.id,
            owner_id,
            &editor_email.to_uppercase(),
            TournamentAccessRole::Editor,
            Utc::now(),
        )
        .await
        .unwrap();
    repository
        .grant_access(
            stored.id,
            owner_id,
            &viewer_email,
            TournamentAccessRole::Viewer,
            Utc::now(),
        )
        .await
        .unwrap();
    assert_eq!(
        repository.access_role(stored.id, owner_id).await.unwrap(),
        Some(TournamentAccessRole::Owner)
    );
    assert_eq!(
        repository.access_role(stored.id, editor_id).await.unwrap(),
        None
    );
    let sharing = repository.sharing(stored.id).await.unwrap();
    assert_eq!(sharing.members.len(), 1);
    assert_eq!(sharing.invitations.len(), 2);

    let editor = AuthenticatedUser {
        user_id: editor_id,
        email: editor_email,
        display_name: Some("Editor".to_owned()),
        avatar_url: None,
    };
    let editor_invitations = repository.received_invitations(&editor).await.unwrap();
    assert_eq!(editor_invitations.len(), 1);
    assert_eq!(editor_invitations[0].tournament_id, stored.id);
    assert_eq!(editor_invitations[0].role, TournamentAccessRole::Editor);
    repository
        .accept_invitation(&editor, editor_invitations[0].id, Utc::now())
        .await
        .unwrap();
    assert_eq!(
        repository.access_role(stored.id, editor_id).await.unwrap(),
        Some(TournamentAccessRole::Editor)
    );
    assert!(
        repository
            .accept_invitation(&editor, editor_invitations[0].id, Utc::now())
            .await
            .is_err()
    );

    insert_user(&pool, viewer_id, &viewer_email).await;
    let viewer = AuthenticatedUser {
        user_id: viewer_id,
        email: viewer_email,
        display_name: Some("Viewer".to_owned()),
        avatar_url: None,
    };
    let viewer_invitations = repository.received_invitations(&viewer).await.unwrap();
    assert_eq!(viewer_invitations.len(), 1);
    repository
        .decline_invitation(&viewer, viewer_invitations[0].id)
        .await
        .unwrap();
    assert_eq!(
        repository.access_role(stored.id, viewer_id).await.unwrap(),
        None
    );
    assert!(
        repository
            .decline_invitation(&viewer, viewer_invitations[0].id)
            .await
            .is_err()
    );
    let decided_sharing = repository.sharing(stored.id).await.unwrap();
    assert_eq!(decided_sharing.members.len(), 2);
    assert!(decided_sharing.invitations.is_empty());

    repository.delete(stored.id, 0).await.unwrap();
    let child_count: i64 = query::<sqlx_postgres::Postgres>(
        "SELECT
            (SELECT COUNT(*) FROM tournament_members WHERE tournament_id = $1)
          + (SELECT COUNT(*) FROM tournament_invitations WHERE tournament_id = $1)
          + (SELECT COUNT(*) FROM entrants WHERE tournament_id = $1)
          + (SELECT COUNT(*) FROM rounds WHERE tournament_id = $1)
          + (SELECT COUNT(*) FROM matches WHERE tournament_id = $1)
          AS child_count",
    )
    .bind(stored.id)
    .fetch_one(&pool)
    .await
    .unwrap()
    .try_get("child_count")
    .unwrap();
    assert_eq!(child_count, 0);

    query::<sqlx_postgres::Postgres>("DELETE FROM users WHERE id IN ($1, $2, $3)")
        .bind(owner_id.as_uuid())
        .bind(editor_id.as_uuid())
        .bind(viewer_id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
}
