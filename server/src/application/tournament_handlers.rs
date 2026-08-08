use std::collections::{HashMap, HashSet};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use uuid::Uuid;

use crate::api_contract::{
    CreateTournamentRequest, EntrantInput, GameScoreInput, RecordMatchResultRequest,
    ReplaceRosterRequest, TournamentMutationRequest, TournamentSummaryView, TournamentView,
    UpdateTournamentConfigurationRequest,
};
use crate::application::{TournamentApplicationError, TournamentEntrant};
use crate::backend::auth::{AuthState, AuthenticatedUser};
use crate::backend::persistence::{NewTournament, StoredTournament};
use crate::backend::server::error::ApiError;
use crate::identity::{ClubId, EntrantId, MatchId};
use crate::pairing::EloRating;
use crate::pairing::algorithms::blossom_v2::BlossomV2Policy;
use crate::results::{GameNumber, GamePoints, GameScore};
use crate::tournament::{MaximumRoundCount, TableCount, TournamentId};

use super::TournamentService;

#[derive(Clone)]
pub struct TournamentApiState {
    auth: AuthState,
    service: TournamentService,
}

impl TournamentApiState {
    pub const fn new(auth: AuthState, service: TournamentService) -> Self {
        Self { auth, service }
    }
}

pub fn routes(state: TournamentApiState) -> Router {
    Router::new()
        .route("/api/tournaments", get(list).post(create))
        .route("/api/tournaments/{tournament_id}", get(load))
        .route(
            "/api/tournaments/{tournament_id}/configuration",
            put(update_configuration),
        )
        .route(
            "/api/tournaments/{tournament_id}/entrants",
            put(replace_roster),
        )
        .route(
            "/api/tournaments/{tournament_id}/start",
            post(start_tournament),
        )
        .route(
            "/api/tournaments/{tournament_id}/pairings/calculate",
            post(calculate_pairings),
        )
        .route(
            "/api/tournaments/{tournament_id}/pairings/publish",
            post(publish_pairings),
        )
        .route(
            "/api/tournaments/{tournament_id}/rounds/complete",
            post(complete_round),
        )
        .route(
            "/api/tournaments/{tournament_id}/matches/{match_id}/result",
            put(record_result),
        )
        .with_state(state)
}

async fn list(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<TournamentSummaryView>>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let summaries = state.service.list(user.user_id).await?;
    Ok(Json(
        summaries
            .into_iter()
            .map(|summary| TournamentSummaryView {
                id: summary.id.to_string(),
                title: summary.title,
                status: summary.status,
                updated_at: summary.updated_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn create(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateTournamentRequest>,
) -> Result<(StatusCode, Json<TournamentView>), ApiError> {
    let user = require_user(&state, &headers).await?;
    let title = request.title.trim();
    if title.is_empty() || title.len() > 200 {
        return Err(ApiError::invalid(
            "invalid_tournament_title",
            "Tournament title must contain between 1 and 200 bytes.",
        ));
    }
    let table_count = TableCount::try_from(request.table_count)
        .map_err(|error| ApiError::invalid("invalid_table_count", error.to_string()))?;
    let maximum_round_count = MaximumRoundCount::try_from(request.maximum_round_count)
        .map_err(|error| ApiError::invalid("invalid_maximum_round_count", error.to_string()))?;
    let stored = state
        .service
        .create(
            user.user_id,
            NewTournament {
                title: TournamentId::new(title),
                match_format: request.match_format,
                table_count,
                maximum_round_count,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(view(&stored))))
}

async fn load(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
) -> Result<Json<TournamentView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let id = parse_id(&tournament_id)?;
    let stored = state.service.load_owned(user.user_id, id).await?;
    Ok(Json(view(&stored)))
}

async fn replace_roster(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<ReplaceRosterRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let mut stored = state
        .service
        .load_owned(user.user_id, parse_id(&tournament_id)?)
        .await?;
    let replacements = roster(&stored, request.entrants)?;
    stored
        .application
        .replace_active_roster(replacements)
        .map_err(domain_error)?;
    state
        .service
        .save(&mut stored, request.expected_tournament_revision)
        .await?;
    Ok(Json(view(&stored)))
}

async fn update_configuration(
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
    let mut stored = state
        .service
        .load_owned(user.user_id, parse_id(&tournament_id)?)
        .await?;
    stored
        .application
        .update_draft_configuration(request.match_format, table_count, maximum_round_count)
        .map_err(domain_error)?;
    state
        .service
        .save(&mut stored, request.expected_tournament_revision)
        .await?;
    Ok(Json(view(&stored)))
}

async fn start_tournament(
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

async fn calculate_pairings(
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

async fn publish_pairings(
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

async fn complete_round(
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

async fn record_result(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path((tournament_id, match_id)): Path<(String, String)>,
    Json(request): Json<RecordMatchResultRequest>,
) -> Result<Json<TournamentView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let mut stored = state
        .service
        .load_owned(user.user_id, parse_id(&tournament_id)?)
        .await?;
    let current_revision = stored
        .application
        .active_round()
        .and_then(|round| {
            round
                .results
                .iter()
                .find(|result| result.match_id().as_str() == match_id)
        })
        .map_or(0, |result| u64::from(result.revision().value()));
    if current_revision != request.expected_revision {
        return Err(ApiError::RevisionConflict);
    }
    let games = request
        .games
        .into_iter()
        .map(game_score)
        .collect::<Result<Vec<_>, _>>()?;
    stored
        .application
        .enter_match_result(&MatchId::new(match_id), games)
        .map_err(domain_error)?;
    let expected_tournament_revision = stored.revision;
    state
        .service
        .save(&mut stored, expected_tournament_revision)
        .await?;
    Ok(Json(view(&stored)))
}

async fn mutate(
    state: TournamentApiState,
    headers: HeaderMap,
    tournament_id: String,
    request: TournamentMutationRequest,
    operation: impl FnOnce(&mut StoredTournament) -> Result<(), TournamentApplicationError>,
) -> Result<Json<TournamentView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let mut stored = state
        .service
        .load_owned(user.user_id, parse_id(&tournament_id)?)
        .await?;
    operation(&mut stored).map_err(domain_error)?;
    state
        .service
        .save(&mut stored, request.expected_tournament_revision)
        .await?;
    Ok(Json(view(&stored)))
}

async fn require_user(
    state: &TournamentApiState,
    headers: &HeaderMap,
) -> Result<AuthenticatedUser, ApiError> {
    state
        .auth
        .authenticated_user(headers)
        .await?
        .ok_or(ApiError::Unauthorized)
}

fn roster(
    stored: &StoredTournament,
    inputs: Vec<EntrantInput>,
) -> Result<Vec<TournamentEntrant>, ApiError> {
    if inputs.len() > 64 {
        return Err(ApiError::invalid(
            "entrant_limit_exceeded",
            "At most 64 active entrants are supported.",
        ));
    }
    let existing = stored
        .application
        .entrants()
        .iter()
        .map(|entrant| (entrant.entrant_id.as_str(), entrant))
        .collect::<HashMap<_, _>>();
    let mut used_ids = HashSet::new();
    let mut clubs = stored
        .application
        .entrants()
        .iter()
        .map(|entrant| (entrant.club_name.to_lowercase(), entrant.club_id.clone()))
        .collect::<HashMap<_, _>>();
    inputs
        .into_iter()
        .map(|input| {
            validate_roster_text(&input.display_name, "display_name")?;
            validate_roster_text(&input.club_name, "club_name")?;
            let entrant_id = match input.entrant_id {
                Some(value) => {
                    if !existing.contains_key(value.as_str()) {
                        return Err(ApiError::invalid(
                            "unknown_entrant",
                            "An entrant ID does not belong to this tournament.",
                        ));
                    }
                    EntrantId::new(value)
                }
                None => EntrantId::new(format!("entrant-{}", Uuid::new_v4())),
            };
            if !used_ids.insert(entrant_id.clone()) {
                return Err(ApiError::invalid(
                    "duplicate_entrant",
                    "An entrant occurs more than once in the roster.",
                ));
            }
            let club_key = input.club_name.trim().to_lowercase();
            let club_id = clubs
                .entry(club_key)
                .or_insert_with(|| ClubId::new(format!("club-{}", Uuid::new_v4())))
                .clone();
            let starting_elo = u32::try_from(input.starting_elo).map_err(|_| {
                ApiError::invalid("invalid_elo", "Starting ELO must be a positive integer.")
            })?;
            Ok(TournamentEntrant {
                entrant_id,
                name: input.display_name.trim().to_owned(),
                club_id,
                club_name: input.club_name.trim().to_owned(),
                starting_elo: EloRating::new(starting_elo),
            })
        })
        .collect()
}

fn validate_roster_text(value: &str, field: &'static str) -> Result<(), ApiError> {
    if value.trim().is_empty() || value.len() > 200 {
        Err(ApiError::invalid(
            "invalid_roster_field",
            format!("{field} must contain between 1 and 200 bytes."),
        ))
    } else {
        Ok(())
    }
}

fn game_score(input: GameScoreInput) -> Result<GameScore, ApiError> {
    Ok(GameScore {
        game_number: GameNumber::try_from(input.game_number)
            .map_err(|error| ApiError::invalid("invalid_game_number", error.to_string()))?,
        home_points: GamePoints::try_from(input.home_points)
            .map_err(|error| ApiError::invalid("invalid_game_score", error.to_string()))?,
        away_points: GamePoints::try_from(input.away_points)
            .map_err(|error| ApiError::invalid("invalid_game_score", error.to_string()))?,
    })
}

fn parse_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::NotFound)
}

fn domain_error(error: TournamentApplicationError) -> ApiError {
    let code = match &error {
        TournamentApplicationError::MatchResult(error) => error.code(),
        TournamentApplicationError::Tournament(error) => error.code(),
        _ => "invalid_tournament_state",
    };
    ApiError::invalid(code, error.to_string())
}

fn view(stored: &StoredTournament) -> TournamentView {
    TournamentView {
        id: stored.id.to_string(),
        revision: stored.revision,
        application: stored.application.snapshot(),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;
    use sqlx_postgres::{PgConnectOptions, PgPoolOptions};

    use super::*;
    use crate::backend::auth::AuthRepository;
    use crate::backend::persistence::TournamentRepository;

    #[tokio::test]
    async fn unauthenticated_tournament_access_is_rejected_without_a_database_query() {
        let pool = PgPoolOptions::new().connect_lazy_with(
            PgConnectOptions::new()
                .host("unreachable.invalid")
                .database("unused")
                .username("unused"),
        );
        let auth = AuthState::without_external_provider(AuthRepository::new(pool.clone()));
        let state = TournamentApiState::new(
            auth,
            TournamentService::new(TournamentRepository::new(pool)),
        );
        let result = list(State(state), HeaderMap::new()).await;
        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }
}
