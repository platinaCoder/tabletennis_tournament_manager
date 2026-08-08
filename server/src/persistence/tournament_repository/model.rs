use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::TournamentApplication;
use crate::backend::auth::UserId;
use crate::results::MatchFormat;
use crate::tournament::{MaximumRoundCount, TableCount, TournamentId};

pub struct NewTournament {
    pub title: TournamentId,
    pub match_format: MatchFormat,
    pub table_count: TableCount,
    pub maximum_round_count: MaximumRoundCount,
}

pub struct StoredTournament {
    pub id: Uuid,
    pub created_by_user_id: UserId,
    pub revision: u64,
    pub application: TournamentApplication,
}

pub struct TournamentSummary {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}
