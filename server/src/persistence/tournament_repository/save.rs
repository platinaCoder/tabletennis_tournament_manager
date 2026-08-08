use chrono::{DateTime, Utc};
use sqlx::query::query;
use sqlx::transaction::Transaction;
use sqlx_postgres::Postgres;
use uuid::Uuid;

use crate::application::TournamentApplication;
use crate::backend::auth::UserId;
use crate::results::MatchFormat;
use crate::tournament::{Tournament, TournamentState};

use super::round_writer::save_rounds;
use super::{NewTournament, StoredTournament, TournamentRepository, TournamentRepositoryError};

impl TournamentRepository {
    pub async fn create(
        &self,
        user_id: UserId,
        new_tournament: NewTournament,
        now: DateTime<Utc>,
    ) -> Result<StoredTournament, TournamentRepositoryError> {
        let id = Uuid::new_v4();
        let application = TournamentApplication::new(Tournament::new(
            new_tournament.title,
            new_tournament.match_format,
            new_tournament.table_count,
            new_tournament.maximum_round_count,
        ));
        let mut transaction = self.pool.begin().await?;
        query::<Postgres>(
            "INSERT INTO tournaments (
                id, created_by_user_id, domain_id, status, match_format,
                table_count, maximum_round_count, active_pairing_policy_version,
                active_scoring_policy_version, revision, created_at, updated_at
             ) VALUES ($1, $2, $3, 'draft', $4, $5, $6,
                       'blossom_v2', 'elo_expectation_delta_v1', 0, $7, $7)",
        )
        .bind(id)
        .bind(user_id.as_uuid())
        .bind(application.tournament().id().as_str())
        .bind(match_format(application.tournament().match_format()))
        .bind(i32::from(application.tournament().table_count().value()))
        .bind(i32::from(
            application.tournament().maximum_round_count().value(),
        ))
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        query::<Postgres>(
            "INSERT INTO tournament_members (
                tournament_id, user_id, role, created_at, updated_at
             ) VALUES ($1, $2, 'owner', $3, $3)",
        )
        .bind(id)
        .bind(user_id.as_uuid())
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(StoredTournament {
            id,
            revision: 0,
            application,
        })
    }

    pub async fn save(
        &self,
        stored: &StoredTournament,
        expected_revision: u64,
        now: DateTime<Utc>,
    ) -> Result<u64, TournamentRepositoryError> {
        let expected_revision = i64::try_from(expected_revision)
            .map_err(|_| invalid("tournament revision exceeds storage limit"))?;
        let mut transaction = self.pool.begin().await?;
        let update = query::<Postgres>(
            "UPDATE tournaments
             SET status = $3, match_format = $4, table_count = $5,
                 maximum_round_count = $6, revision = revision + 1,
                 updated_at = $7
             WHERE id = $1 AND revision = $2",
        )
        .bind(stored.id)
        .bind(expected_revision)
        .bind(tournament_state(stored.application.tournament().state()))
        .bind(match_format(stored.application.tournament().match_format()))
        .bind(i32::from(
            stored.application.tournament().table_count().value(),
        ))
        .bind(i32::from(
            stored
                .application
                .tournament()
                .maximum_round_count()
                .value(),
        ))
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        if update.rows_affected() == 0 {
            return Err(TournamentRepositoryError::RevisionConflict);
        }
        save_entrants(&mut transaction, stored, now).await?;
        save_rounds(&mut transaction, stored, now).await?;
        transaction.commit().await?;
        u64::try_from(expected_revision + 1)
            .map_err(|_| invalid("tournament revision exceeds storage limit"))
    }
}

async fn save_entrants(
    transaction: &mut Transaction<'_, Postgres>,
    stored: &StoredTournament,
    now: DateTime<Utc>,
) -> Result<(), TournamentRepositoryError> {
    query::<Postgres>(
        "UPDATE entrants SET is_active = false, updated_at = $2 WHERE tournament_id = $1",
    )
    .bind(stored.id)
    .bind(now)
    .execute(&mut **transaction)
    .await?;
    for entrant in stored.application.entrants() {
        query::<Postgres>(
            "INSERT INTO entrants (
                tournament_id, entrant_id, display_name, club_id, club_name,
                starting_elo, is_active, created_at, updated_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
             ON CONFLICT (tournament_id, entrant_id) DO UPDATE SET
                display_name = EXCLUDED.display_name,
                club_id = EXCLUDED.club_id,
                club_name = EXCLUDED.club_name,
                starting_elo = EXCLUDED.starting_elo,
                is_active = EXCLUDED.is_active,
                updated_at = EXCLUDED.updated_at",
        )
        .bind(stored.id)
        .bind(entrant.entrant_id.as_str())
        .bind(&entrant.name)
        .bind(entrant.club_id.as_str())
        .bind(&entrant.club_name)
        .bind(i32::try_from(entrant.starting_elo.value()).map_err(invalid)?)
        .bind(stored.application.is_entrant_active(&entrant.entrant_id))
        .bind(now)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

const fn tournament_state(value: TournamentState) -> &'static str {
    match value {
        TournamentState::Draft => "draft",
        TournamentState::Started => "started",
    }
}

const fn match_format(value: MatchFormat) -> &'static str {
    match value {
        MatchFormat::BestOfThree => "best_of_three",
        MatchFormat::BestOfFive => "best_of_five",
    }
}

fn invalid(error: impl std::fmt::Display) -> TournamentRepositoryError {
    TournamentRepositoryError::InvalidStoredData(error.to_string())
}
