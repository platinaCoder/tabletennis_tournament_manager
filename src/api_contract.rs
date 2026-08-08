//! JSON contracts shared by the WASM client and server API.
//!
//! These types contain no SQL, OAuth-provider or UI framework details.

use serde::{Deserialize, Serialize};

use crate::application::TournamentApplicationSnapshot;
use crate::results::MatchFormat;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthenticatedUserView {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthenticationView {
    pub authenticated: bool,
    pub user: Option<AuthenticatedUserView>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TournamentSummaryView {
    pub id: String,
    pub title: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TournamentView {
    pub id: String,
    pub revision: u64,
    pub application: TournamentApplicationSnapshot,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateTournamentRequest {
    pub title: String,
    pub match_format: MatchFormat,
    pub table_count: i64,
    pub maximum_round_count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpdateTournamentConfigurationRequest {
    pub expected_tournament_revision: u64,
    pub match_format: MatchFormat,
    pub table_count: i64,
    pub maximum_round_count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EntrantInput {
    pub entrant_id: Option<String>,
    pub display_name: String,
    pub club_id: Option<String>,
    pub club_name: String,
    pub starting_elo: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReplaceRosterRequest {
    pub expected_tournament_revision: u64,
    pub entrants: Vec<EntrantInput>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TournamentMutationRequest {
    pub expected_tournament_revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameScoreInput {
    pub game_number: i64,
    pub home_points: i64,
    pub away_points: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RecordMatchResultRequest {
    pub expected_revision: u64,
    pub games: Vec<GameScoreInput>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiErrorView {
    pub code: String,
    pub message: String,
}
