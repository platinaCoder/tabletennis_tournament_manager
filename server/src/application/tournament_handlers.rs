use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use uuid::Uuid;

use crate::api_contract::{
    CreateTournamentRequest, DeleteTournamentRequest, DeleteTournamentResponse,
    TournamentAccessRole, TournamentSummaryView, TournamentView,
};
use crate::backend::auth::{AuthState, AuthenticatedUser};
use crate::backend::persistence::{NewTournament, StoredTournament};
use crate::backend::server::error::ApiError;
use crate::tournament::{MaximumRoundCount, TableCount, TournamentId};

use super::TournamentService;
use super::tournament_sharing_handlers as sharing;
use super::tournament_workflow_handlers as workflow;

#[derive(Clone)]
pub struct TournamentApiState {
    pub(super) auth: AuthState,
    pub(super) service: TournamentService,
}

impl TournamentApiState {
    pub const fn new(auth: AuthState, service: TournamentService) -> Self {
        Self { auth, service }
    }
}

pub fn routes(state: TournamentApiState) -> Router {
    Router::new()
        .route("/api/tournaments", get(list).post(create))
        .route(
            "/api/tournaments/{tournament_id}",
            get(load).delete(delete_tournament),
        )
        .route(
            "/api/tournaments/{tournament_id}/sharing",
            get(sharing::load).post(sharing::grant),
        )
        .route(
            "/api/tournaments/{tournament_id}/members/{user_id}",
            put(sharing::update_member).delete(sharing::remove_member),
        )
        .route(
            "/api/tournaments/{tournament_id}/invitations/{invitation_id}",
            axum::routing::delete(sharing::delete_invitation),
        )
        .route(
            "/api/tournaments/{tournament_id}/configuration",
            put(workflow::update_configuration),
        )
        .route(
            "/api/tournaments/{tournament_id}/entrants",
            put(workflow::replace_roster),
        )
        .route(
            "/api/tournaments/{tournament_id}/start",
            post(workflow::start_tournament),
        )
        .route(
            "/api/tournaments/{tournament_id}/pairings/calculate",
            post(workflow::calculate_pairings),
        )
        .route(
            "/api/tournaments/{tournament_id}/pairings/publish",
            post(workflow::publish_pairings),
        )
        .route(
            "/api/tournaments/{tournament_id}/rounds/complete",
            post(workflow::complete_round),
        )
        .route(
            "/api/tournaments/{tournament_id}/matches/{match_id}/result",
            put(workflow::record_result),
        )
        .with_state(state)
}

async fn list(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<TournamentSummaryView>>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let summaries = state.service.list(&user).await?;
    Ok(Json(
        summaries
            .into_iter()
            .map(|summary| TournamentSummaryView {
                id: summary.id.to_string(),
                title: summary.title,
                status: summary.status,
                revision: summary.revision,
                access_role: summary.access_role,
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
    Ok((
        StatusCode::CREATED,
        Json(view(&stored, TournamentAccessRole::Owner)),
    ))
}

async fn load(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
) -> Result<Json<TournamentView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let (stored, role) = state
        .service
        .load_for_view(user.user_id, parse_id(&tournament_id)?)
        .await?;
    Ok(Json(view(&stored, role)))
}

async fn delete_tournament(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<DeleteTournamentRequest>,
) -> Result<Json<DeleteTournamentResponse>, ApiError> {
    let user = require_user(&state, &headers).await?;
    state
        .service
        .delete(
            user.user_id,
            parse_id(&tournament_id)?,
            request.expected_tournament_revision,
        )
        .await?;
    Ok(Json(DeleteTournamentResponse { deleted: true }))
}

pub(super) async fn require_user(
    state: &TournamentApiState,
    headers: &HeaderMap,
) -> Result<AuthenticatedUser, ApiError> {
    state
        .auth
        .authenticated_user(headers)
        .await?
        .ok_or(ApiError::Unauthorized)
}

pub(super) fn parse_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::NotFound)
}

pub(super) fn view(stored: &StoredTournament, access_role: TournamentAccessRole) -> TournamentView {
    TournamentView {
        id: stored.id.to_string(),
        revision: stored.revision,
        access_role,
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
