use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

use crate::api_contract::{ReceivedTournamentInvitationView, TournamentInvitationDecisionView};
use crate::backend::persistence::ReceivedTournamentInvitation;
use crate::backend::server::error::ApiError;

use super::tournament_handlers::{TournamentApiState, parse_id, require_user};

pub(super) async fn list(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ReceivedTournamentInvitationView>>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let invitations = state.service.received_invitations(&user).await?;
    Ok(Json(invitations.into_iter().map(view).collect()))
}

pub(super) async fn accept(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(invitation_id): Path<String>,
) -> Result<Json<TournamentInvitationDecisionView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    state
        .service
        .accept_invitation(&user, parse_id(&invitation_id)?)
        .await?;
    Ok(Json(TournamentInvitationDecisionView { accepted: true }))
}

pub(super) async fn decline(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(invitation_id): Path<String>,
) -> Result<Json<TournamentInvitationDecisionView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    state
        .service
        .decline_invitation(&user, parse_id(&invitation_id)?)
        .await?;
    Ok(Json(TournamentInvitationDecisionView { accepted: false }))
}

fn view(invitation: ReceivedTournamentInvitation) -> ReceivedTournamentInvitationView {
    ReceivedTournamentInvitationView {
        id: invitation.id.to_string(),
        tournament_id: invitation.tournament_id.to_string(),
        tournament_title: invitation.tournament_title,
        role: invitation.role,
        invited_by_email: invitation.invited_by_email,
        invited_by_display_name: invitation.invited_by_display_name,
        created_at: invitation.created_at.to_rfc3339(),
    }
}
