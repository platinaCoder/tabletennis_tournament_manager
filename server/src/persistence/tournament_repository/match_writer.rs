use chrono::{DateTime, Utc};
use sqlx::query::query;
use sqlx::row::Row;
use sqlx::transaction::Transaction;
use sqlx_postgres::Postgres;
use uuid::Uuid;

use crate::results::{MatchResult, RoundActivity};
use crate::scheduling::ScheduledMatch;

use super::TournamentRepositoryError;

pub(super) async fn save_match(
    transaction: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
    round_id: Uuid,
    scheduled: &ScheduledMatch,
    result: Option<&MatchResult>,
    now: DateTime<Utc>,
) -> Result<(), TournamentRepositoryError> {
    let row = query::<Postgres>(
        "INSERT INTO matches (
            id, tournament_id, round_id, match_id, home_entrant_id,
            away_entrant_id, table_number, publication_status, round_activity,
            revision, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'published', $8, 0, $9, $9)
         ON CONFLICT (tournament_id, match_id) DO UPDATE SET
            round_id = EXCLUDED.round_id,
            table_number = EXCLUDED.table_number,
            round_activity = EXCLUDED.round_activity,
            updated_at = EXCLUDED.updated_at
         RETURNING id, revision",
    )
    .bind(Uuid::new_v4())
    .bind(tournament_id)
    .bind(round_id)
    .bind(scheduled.match_id.as_str())
    .bind(scheduled.home_entrant_id.as_str())
    .bind(scheduled.away_entrant_id.as_str())
    .bind(
        scheduled
            .table_number()
            .map(|table| i32::from(table.value())),
    )
    .bind(round_activity(scheduled.round_activity))
    .bind(now)
    .fetch_one(&mut **transaction)
    .await?;
    let database_match_id: Uuid = row.try_get("id")?;
    let stored_revision: i64 = row.try_get("revision")?;
    match result {
        Some(result) => {
            let desired_revision = i64::from(result.revision().value());
            if stored_revision == desired_revision {
                return Ok(());
            }
            if stored_revision + 1 != desired_revision {
                return Err(TournamentRepositoryError::RevisionConflict);
            }
            insert_result_revision(transaction, tournament_id, database_match_id, result, now)
                .await?;
        }
        None if stored_revision != 0 => {
            return Err(invalid(
                "persisted result disappeared from application state",
            ));
        }
        None => {}
    }
    Ok(())
}

async fn insert_result_revision(
    transaction: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
    match_id: Uuid,
    result: &MatchResult,
    now: DateTime<Utc>,
) -> Result<(), TournamentRepositoryError> {
    let revision = i64::from(result.revision().value());
    let entered_at = DateTime::<Utc>::from(result.entered_at());
    let corrected_at = result.corrected_at().map(DateTime::<Utc>::from);
    query::<Postgres>(
        "INSERT INTO match_result_revisions (
            match_id, tournament_id, revision, winner_entrant_id,
            home_games_won, away_games_won, entered_at, corrected_at,
            correction_reason
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL)",
    )
    .bind(match_id)
    .bind(tournament_id)
    .bind(revision)
    .bind(result.winner_id().as_str())
    .bind(i32::from(result.home_games_won().value()))
    .bind(i32::from(result.away_games_won().value()))
    .bind(entered_at)
    .bind(corrected_at)
    .execute(&mut **transaction)
    .await?;
    for game in result.games() {
        query::<Postgres>(
            "INSERT INTO game_scores (
                match_id, result_revision, game_number, home_points, away_points
             ) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(match_id)
        .bind(revision)
        .bind(i32::from(game.game_number.value()))
        .bind(i32::from(game.home_points.value()))
        .bind(i32::from(game.away_points.value()))
        .execute(&mut **transaction)
        .await?;
    }
    let update = query::<Postgres>(
        "UPDATE matches
         SET winner_entrant_id = $2, home_games_won = $3,
             away_games_won = $4, revision = $5, updated_at = $6
         WHERE id = $1 AND revision = $7",
    )
    .bind(match_id)
    .bind(result.winner_id().as_str())
    .bind(i32::from(result.home_games_won().value()))
    .bind(i32::from(result.away_games_won().value()))
    .bind(revision)
    .bind(now)
    .bind(revision - 1)
    .execute(&mut **transaction)
    .await?;
    if update.rows_affected() != 1 {
        return Err(TournamentRepositoryError::RevisionConflict);
    }
    Ok(())
}

const fn round_activity(value: RoundActivity) -> &'static str {
    match value {
        RoundActivity::Active => "active",
        RoundActivity::Inactive => "inactive",
    }
}

fn invalid(error: impl std::fmt::Display) -> TournamentRepositoryError {
    TournamentRepositoryError::InvalidStoredData(error.to_string())
}
