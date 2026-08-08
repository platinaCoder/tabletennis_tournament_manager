use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api_contract::TournamentAccessRole;
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
    pub revision: u64,
    pub application: TournamentApplication,
}

pub struct TournamentSummary {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub revision: u64,
    pub access_role: TournamentAccessRole,
    pub updated_at: DateTime<Utc>,
}

pub struct TournamentMember {
    pub user_id: UserId,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: TournamentAccessRole,
}

pub struct TournamentInvitation {
    pub id: Uuid,
    pub email: String,
    pub role: TournamentAccessRole,
    pub created_at: DateTime<Utc>,
}

pub struct ReceivedTournamentInvitation {
    pub id: Uuid,
    pub tournament_id: Uuid,
    pub tournament_title: String,
    pub role: TournamentAccessRole,
    pub invited_by_email: String,
    pub invited_by_display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct TournamentSharing {
    pub members: Vec<TournamentMember>,
    pub invitations: Vec<TournamentInvitation>,
}
