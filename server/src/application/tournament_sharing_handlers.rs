use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use uuid::Uuid;

use crate::api_contract::{
    ShareTournamentRequest, TournamentInvitationView, TournamentMemberView, TournamentSharingView,
    UpdateTournamentMemberRequest,
};
use crate::backend::auth::UserId;
use crate::backend::persistence::TournamentSharing;
use crate::backend::server::error::ApiError;

use super::tournament_handlers::{TournamentApiState, parse_id, require_user};

pub(super) async fn load(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
) -> Result<Json<TournamentSharingView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let tournament_id = parse_id(&tournament_id)?;
    let sharing = state.service.sharing(user.user_id, tournament_id).await?;
    Ok(Json(view(tournament_id, sharing)))
}

pub(super) async fn grant(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path(tournament_id): Path<String>,
    Json(request): Json<ShareTournamentRequest>,
) -> Result<Json<TournamentSharingView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let tournament_id = parse_id(&tournament_id)?;
    let sharing = state
        .service
        .grant_access(&user, tournament_id, &request.email, request.role)
        .await?;
    Ok(Json(view(tournament_id, sharing)))
}

pub(super) async fn update_member(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path((tournament_id, member_user_id)): Path<(String, String)>,
    Json(request): Json<UpdateTournamentMemberRequest>,
) -> Result<Json<TournamentSharingView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let tournament_id = parse_id(&tournament_id)?;
    let sharing = state
        .service
        .update_member_role(
            user.user_id,
            tournament_id,
            UserId::from_uuid(parse_id(&member_user_id)?),
            request.role,
        )
        .await?;
    Ok(Json(view(tournament_id, sharing)))
}

pub(super) async fn remove_member(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path((tournament_id, member_user_id)): Path<(String, String)>,
) -> Result<Json<TournamentSharingView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let tournament_id = parse_id(&tournament_id)?;
    let sharing = state
        .service
        .remove_member(
            user.user_id,
            tournament_id,
            UserId::from_uuid(parse_id(&member_user_id)?),
        )
        .await?;
    Ok(Json(view(tournament_id, sharing)))
}

pub(super) async fn delete_invitation(
    State(state): State<TournamentApiState>,
    headers: HeaderMap,
    Path((tournament_id, invitation_id)): Path<(String, String)>,
) -> Result<Json<TournamentSharingView>, ApiError> {
    let user = require_user(&state, &headers).await?;
    let tournament_id = parse_id(&tournament_id)?;
    let sharing = state
        .service
        .delete_invitation(user.user_id, tournament_id, parse_id(&invitation_id)?)
        .await?;
    Ok(Json(view(tournament_id, sharing)))
}

fn view(tournament_id: Uuid, sharing: TournamentSharing) -> TournamentSharingView {
    TournamentSharingView {
        tournament_id: tournament_id.to_string(),
        members: sharing
            .members
            .into_iter()
            .map(|member| TournamentMemberView {
                user_id: member.user_id.as_uuid().to_string(),
                email: member.email,
                display_name: member.display_name,
                avatar_url: member.avatar_url,
                role: member.role,
            })
            .collect(),
        invitations: sharing
            .invitations
            .into_iter()
            .map(|invitation| TournamentInvitationView {
                id: invitation.id.to_string(),
                email: invitation.email,
                role: invitation.role,
                created_at: invitation.created_at.to_rfc3339(),
            })
            .collect(),
    }
}
