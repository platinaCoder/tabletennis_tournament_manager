use chrono::Utc;
use sqlx::query::query;
use sqlx_postgres::PgPoolOptions;

use crate::application::TournamentEntrant;
use crate::backend::auth::UserId;
use crate::backend::persistence::migrate_test_database;
use crate::identity::{ClubId, EntrantId};
use crate::pairing::EloRating;
use crate::pairing::algorithms::blossom_v2::BlossomV2Policy;
use crate::results::{GameScore, MatchFormat};
use crate::tournament::{MaximumRoundCount, TableCount, TournamentId};

use super::{NewTournament, TournamentRepository};

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
