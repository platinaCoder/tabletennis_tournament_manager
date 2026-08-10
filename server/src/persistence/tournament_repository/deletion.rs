use sqlx::query::query;
use sqlx_postgres::Postgres;
use uuid::Uuid;

use super::{TournamentRepository, TournamentRepositoryError};

impl TournamentRepository {
    pub async fn delete(
        &self,
        tournament_id: Uuid,
        expected_revision: u64,
    ) -> Result<(), TournamentRepositoryError> {
        let expected_revision = i64::try_from(expected_revision)
            .map_err(|_| invalid("tournament revision exceeds storage limit"))?;
        let mut transaction = self.pool.begin().await?;
        let tournament = query::<Postgres>(
            "SELECT id FROM tournaments
             WHERE id = $1 AND revision = $2
             FOR UPDATE",
        )
        .bind(tournament_id)
        .bind(expected_revision)
        .fetch_optional(&mut *transaction)
        .await?;
        if tournament.is_none() {
            return Err(TournamentRepositoryError::RevisionConflict);
        }

        // Entrants are referenced by rounds, matches, and result revisions. Remove
        // those dependants explicitly so PostgreSQL never has to choose an unsafe
        // order between the intersecting tournament cascades.
        query::<Postgres>("DELETE FROM match_result_revisions WHERE tournament_id = $1")
            .bind(tournament_id)
            .execute(&mut *transaction)
            .await?;
        query::<Postgres>("DELETE FROM matches WHERE tournament_id = $1")
            .bind(tournament_id)
            .execute(&mut *transaction)
            .await?;
        query::<Postgres>("DELETE FROM rounds WHERE tournament_id = $1")
            .bind(tournament_id)
            .execute(&mut *transaction)
            .await?;
        query::<Postgres>("DELETE FROM entrants WHERE tournament_id = $1")
            .bind(tournament_id)
            .execute(&mut *transaction)
            .await?;

        let deleted = query::<Postgres>("DELETE FROM tournaments WHERE id = $1")
            .bind(tournament_id)
            .execute(&mut *transaction)
            .await?;
        if deleted.rows_affected() != 1 {
            return Err(TournamentRepositoryError::RevisionConflict);
        }
        transaction.commit().await?;
        Ok(())
    }
}

fn invalid(error: impl std::fmt::Display) -> TournamentRepositoryError {
    TournamentRepositoryError::InvalidStoredData(error.to_string())
}
