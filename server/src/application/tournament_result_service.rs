use chrono::Utc;
use uuid::Uuid;

use crate::api_contract::TournamentAccessRole;
use crate::backend::auth::UserId;
use crate::backend::persistence::{StoredTournament, TournamentRepositoryError};
use crate::backend::server::error::ApiError;
use crate::identity::MatchId;
use crate::results::GameScore;

use super::tournament_input::domain_error;
use super::tournament_service::{TournamentService, repository_error};

const RESULT_SAVE_ATTEMPTS: usize = 4;

impl TournamentService {
    pub async fn record_result(
        &self,
        user_id: UserId,
        tournament_id: Uuid,
        match_id: &MatchId,
        expected_match_revision: u64,
        games: &[GameScore],
        correction_reason: Option<&str>,
    ) -> Result<(StoredTournament, TournamentAccessRole), ApiError> {
        for _ in 0..RESULT_SAVE_ATTEMPTS {
            let (mut stored, role) = self.load_for_edit(user_id, tournament_id).await?;
            let current_revision = stored
                .application
                .match_result(match_id)
                .map_or(0, |result| u64::from(result.revision().value()));
            if current_revision != expected_match_revision {
                return Err(ApiError::ResultRevisionConflict);
            }
            if current_revision == 0 {
                if correction_reason.is_some() {
                    return Err(ApiError::invalid(
                        "unexpected_correction_reason",
                        "An initial result cannot contain a correction reason.",
                    ));
                }
                stored
                    .application
                    .enter_match_result(match_id, games.to_vec())
                    .map_err(domain_error)?;
            } else {
                stored
                    .application
                    .correct_match_result(
                        match_id,
                        games.to_vec(),
                        correction_reason.map(str::to_owned),
                    )
                    .map_err(domain_error)?;
            }
            match self
                .repository
                .save(&stored, stored.revision, Utc::now())
                .await
            {
                Ok(revision) => {
                    stored.revision = revision;
                    return Ok((stored, role));
                }
                Err(TournamentRepositoryError::RevisionConflict) => continue,
                Err(error) => return Err(repository_error(error)),
            }
        }
        Err(ApiError::RevisionConflict)
    }
}

#[cfg(test)]
mod postgres_tests {
    use chrono::Utc;
    use sqlx::query::query;
    use sqlx_postgres::PgPoolOptions;

    use super::*;
    use crate::application::TournamentEntrant;
    use crate::backend::persistence::{NewTournament, TournamentRepository, migrate_test_database};
    use crate::identity::{ClubId, EntrantId};
    use crate::pairing::EloRating;
    use crate::pairing::algorithms::blossom_v2::BlossomV2Policy;
    use crate::results::{GameScore, MatchFormat};
    use crate::tournament::{MaximumRoundCount, TableCount, TournamentId};

    #[tokio::test]
    #[ignore = "requires an isolated TEST_DATABASE_URL PostgreSQL database"]
    async fn different_matches_can_be_recorded_concurrently_without_lost_results() {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(4)
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
        .bind(format!("concurrency-{}@test.invalid", user_id.as_uuid()))
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
        let repository = TournamentRepository::new(pool.clone());
        let mut stored = repository
            .create(
                user_id,
                NewTournament {
                    title: TournamentId::new(format!("concurrent-{}", Uuid::new_v4())),
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
        repository.save(&stored, 0, Utc::now()).await.unwrap();

        let service = TournamentService::new(repository.clone());
        let first_games = vec![
            GameScore::new(1, 11, 7).unwrap(),
            GameScore::new(2, 11, 8).unwrap(),
        ];
        let second_games = vec![
            GameScore::new(1, 7, 11).unwrap(),
            GameScore::new(2, 8, 11).unwrap(),
        ];
        let first = service.record_result(user_id, stored.id, &match_ids[0], 0, &first_games, None);
        let second =
            service.record_result(user_id, stored.id, &match_ids[1], 0, &second_games, None);
        let (first_result, second_result) = tokio::join!(first, second);
        first_result.unwrap();
        second_result.unwrap();

        let loaded = repository.load(stored.id).await.unwrap().unwrap();
        assert_eq!(loaded.application.active_round().unwrap().results.len(), 2);
        assert_eq!(loaded.revision, 3);

        service
            .record_result(
                user_id,
                stored.id,
                &match_ids[0],
                1,
                &second_games,
                Some("Scores were entered on the wrong sides"),
            )
            .await
            .unwrap();
        let stale_correction = service
            .record_result(
                user_id,
                stored.id,
                &match_ids[0],
                1,
                &first_games,
                Some("Stale correction"),
            )
            .await;
        assert!(matches!(
            stale_correction,
            Err(ApiError::ResultRevisionConflict)
        ));
        let corrected = repository.load(stored.id).await.unwrap().unwrap();
        let result = corrected.application.match_result(&match_ids[0]).unwrap();
        assert_eq!(result.revision().value(), 2);
        assert_eq!(
            result.winner_id(),
            &corrected
                .application
                .active_round()
                .unwrap()
                .scheduled_matches
                .iter()
                .find(|scheduled| scheduled.match_id == match_ids[0])
                .unwrap()
                .away_entrant_id
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
}
