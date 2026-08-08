use chrono::Utc;
use uuid::Uuid;

use crate::backend::auth::AuthenticatedUser;
use crate::backend::persistence::ReceivedTournamentInvitation;
use crate::backend::server::error::ApiError;

use super::tournament_service::{TournamentService, repository_error};

impl TournamentService {
    pub async fn received_invitations(
        &self,
        user: &AuthenticatedUser,
    ) -> Result<Vec<ReceivedTournamentInvitation>, ApiError> {
        self.repository
            .received_invitations(user)
            .await
            .map_err(repository_error)
    }

    pub async fn accept_invitation(
        &self,
        user: &AuthenticatedUser,
        invitation_id: Uuid,
    ) -> Result<(), ApiError> {
        self.repository
            .accept_invitation(user, invitation_id, Utc::now())
            .await
            .map_err(repository_error)
    }

    pub async fn decline_invitation(
        &self,
        user: &AuthenticatedUser,
        invitation_id: Uuid,
    ) -> Result<(), ApiError> {
        self.repository
            .decline_invitation(user, invitation_id)
            .await
            .map_err(repository_error)
    }
}
