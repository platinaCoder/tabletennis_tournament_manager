use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

use crate::api_contract::{
    RecordMatchResultRequest, ReplaceRosterRequest, TournamentMutationRequest, TournamentView,
    UpdateTournamentConfigurationRequest,
};
use crate::application::TournamentApplicationError;
use crate::backend::persistence::StoredTournament;
use crate::backend::server::error::ApiError;
use crate::identity::MatchId;
use crate::pairing::algorithms::blossom_v2::BlossomV2Policy;
use crate::tournament::{MaximumRoundCount, TableCount};

use super::tournament_handlers::{TournamentApiState, parse_id, require_user, view};
use super::tournament_input::{domain_error, game_score, roster};

pub(super) async fn replace_roster(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<ReplaceRosterRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let (mut stored, role) = state
        .service
        .load_for_edit(user.user_id, parse_id(&tournament_id)?)
        .await?;
    let replacements = roster(&stored, request.entrants)?;
    stored
        .application
        .replace_active_roster(replacements)
        .map_err(domain_error)?;
    state
        .service
        .save(
            user.user_id,
            &mut stored,
            request.expected_tournament_revision,
        )
        .await?;
    Ok(Json(view(&stored, role)))
}

pub(super) async fn update_configuration(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<UpdateTournamentConfigurationRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    let table_count = TableCount::try_from(request.table_count)
        .map_err(|error| ApiError::invalid("invalid_table_count", error.to_string()))?;
    let maximum_round_count = MaximumRoundCount::try_from(request.maximum_round_count)
        .map_err(|error| ApiError::invalid("invalid_maximum_round_count", error.to_string()))?;
    let user = require_user(&state, &headers).await?;
    let (mut stored, role) = state
        .service
        .load_for_edit(user.user_id, parse_id(&tournament_id)?)
        .await?;
    stored
        .application
        .update_draft_configuration(request.match_format, table_count, maximum_round_count)
        .map_err(domain_error)?;
    state
        .service
        .save(
            user.user_id,
            &mut stored,
            request.expected_tournament_revision,
        )
        .await?;
    Ok(Json(view(&stored, role)))
}

pub(super) async fn start_tournament(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<TournamentMutationRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    mutate(state, headers, tournament_id, request, |stored| {
        stored.application.start_tournament().map(|_| ())
    })
    .await
}

pub(super) async fn calculate_pairings(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<TournamentMutationRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    mutate(state, headers, tournament_id, request, |stored| {
        stored
            .application
            .calculate_pairings(BlossomV2Policy::default())
            .map(|_| ())
    })
    .await
}

pub(super) async fn publish_pairings(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<TournamentMutationRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    mutate(state, headers, tournament_id, request, |stored| {
        stored.application.publish_pairings().map(|_| ())
    })
    .await
}

pub(super) async fn complete_round(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<TournamentMutationRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    mutate(state, headers, tournament_id, request, |stored| {
        stored.application.complete_round().map(|_| ())
    })
    .await
}

pub(super) async fn record_result(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path((tournament_id, match_id)): Path<(String, String)>,
    Json(request): Json<RecordMatchResultRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let games = request
        .games
        .into_iter()
        .map(game_score)
        .collect::<Result<Vec<_>, _>>()?;
    let (stored, role) = state
        .service
        .record_result(
            user.user_id,
            parse_id(&tournament_id)?,
            &MatchId::new(match_id),
            request.expected_revision,
            &games,
            request.correction_reason.as_deref(),
        )
        .await?;
    Ok(Json(view(&stored, role)))
}

async fn mutate(
    state: TournamentApiState,
    headers: HeaderMap,
    tournament_id: String,
    request: TournamentMutationRequest,
    operation: impl FnOnce(&mut StoredTournament) -> Result<(), TournamentApplicationError>,
) -> Result<Json<TournamentView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let (mut stored, role) = state
        .service
        .load_for_edit(user.user_id, parse_id(&tournament_id)?)
        .await?;
    operation(&mut stored).map_err(domain_error)?;
    state
        .service
        .save(
            user.user_id,
            &mut stored,
            request.expected_tournament_revision,
        )
        .await?;
    Ok(Json(view(&stored, role)))
}
