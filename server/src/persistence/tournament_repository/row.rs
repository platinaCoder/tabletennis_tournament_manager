use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::from_row::FromRow;
use sqlx::row::Row;
use sqlx_postgres::PgRow;
use uuid::Uuid;

pub(super) struct TournamentRow {
    pub id: Uuid,
    pub created_by_user_id: Uuid,
    pub domain_id: String,
    pub status: String,
    pub match_format: String,
    pub table_count: i32,
    pub maximum_round_count: i32,
    pub revision: i64,
}

impl<'row> FromRow<'row, PgRow> for TournamentRow {
    fn from_row(row: &'row PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            created_by_user_id: row.try_get("created_by_user_id")?,
            domain_id: row.try_get("domain_id")?,
            status: row.try_get("status")?,
            match_format: row.try_get("match_format")?,
            table_count: row.try_get("table_count")?,
            maximum_round_count: row.try_get("maximum_round_count")?,
            revision: row.try_get("revision")?,
        })
    }
}

pub(super) struct EntrantRow {
    pub entrant_id: String,
    pub display_name: String,
    pub club_id: String,
    pub club_name: String,
    pub starting_elo: i32,
    pub is_active: bool,
}

impl<'row> FromRow<'row, PgRow> for EntrantRow {
    fn from_row(row: &'row PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            entrant_id: row.try_get("entrant_id")?,
            display_name: row.try_get("display_name")?,
            club_id: row.try_get("club_id")?,
            club_name: row.try_get("club_name")?,
            starting_elo: row.try_get("starting_elo")?,
            is_active: row.try_get("is_active")?,
        })
    }
}

pub(super) struct RoundRow {
    pub id: Uuid,
    pub round_number: i32,
    pub status: String,
    pub pairing_snapshot: Value,
    pub pairing_proposal: Value,
    pub bye_entrant_id: Option<String>,
}

impl<'row> FromRow<'row, PgRow> for RoundRow {
    fn from_row(row: &'row PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            round_number: row.try_get("round_number")?,
            status: row.try_get("status")?,
            pairing_snapshot: row.try_get("pairing_snapshot")?,
            pairing_proposal: row.try_get("pairing_proposal")?,
            bye_entrant_id: row.try_get("bye_entrant_id")?,
        })
    }
}

pub(super) struct MatchRow {
    pub id: Uuid,
    pub match_id: String,
    pub home_entrant_id: String,
    pub away_entrant_id: String,
    pub table_number: Option<i32>,
    pub publication_status: String,
    pub round_activity: String,
    pub winner_entrant_id: Option<String>,
    pub home_games_won: Option<i32>,
    pub away_games_won: Option<i32>,
    pub revision: i64,
}

impl<'row> FromRow<'row, PgRow> for MatchRow {
    fn from_row(row: &'row PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            match_id: row.try_get("match_id")?,
            home_entrant_id: row.try_get("home_entrant_id")?,
            away_entrant_id: row.try_get("away_entrant_id")?,
            table_number: row.try_get("table_number")?,
            publication_status: row.try_get("publication_status")?,
            round_activity: row.try_get("round_activity")?,
            winner_entrant_id: row.try_get("winner_entrant_id")?,
            home_games_won: row.try_get("home_games_won")?,
            away_games_won: row.try_get("away_games_won")?,
            revision: row.try_get("revision")?,
        })
    }
}

pub(super) struct ResultRow {
    pub winner_entrant_id: String,
    pub home_games_won: i32,
    pub away_games_won: i32,
    pub entered_at: DateTime<Utc>,
    pub corrected_at: Option<DateTime<Utc>>,
}

impl<'row> FromRow<'row, PgRow> for ResultRow {
    fn from_row(row: &'row PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            winner_entrant_id: row.try_get("winner_entrant_id")?,
            home_games_won: row.try_get("home_games_won")?,
            away_games_won: row.try_get("away_games_won")?,
            entered_at: row.try_get("entered_at")?,
            corrected_at: row.try_get("corrected_at")?,
        })
    }
}

pub(super) struct GameRow {
    pub game_number: i32,
    pub home_points: i32,
    pub away_points: i32,
}

impl<'row> FromRow<'row, PgRow> for GameRow {
    fn from_row(row: &'row PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            game_number: row.try_get("game_number")?,
            home_points: row.try_get("home_points")?,
            away_points: row.try_get("away_points")?,
        })
    }
}
