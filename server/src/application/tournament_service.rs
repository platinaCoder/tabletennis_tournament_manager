use chrono::Utc;
use uuid::Uuid;

use crate::api_contract::TournamentAccessRole;
use crate::backend::auth::{AuthenticatedUser, UserId};
use crate::backend::persistence::{
    NewTournament, StoredTournament, TournamentRepository, TournamentRepositoryError,
    TournamentSummary,
};
use crate::backend::server::error::ApiError;

use super::tournament_access::TournamentAccessPolicy;

#[derive(Clone)]
pub struct TournamentService {
    pub(super) repository: TournamentRepository,
}

impl TournamentService {
    pub const fn new(repository: TournamentRepository) -> Self {
        Self { repository }
    }

    pub async fn list(&self, user: &AuthenticatedUser) -> Result<Vec<TournamentSummary>, ApiError> {
        self.repository
            .claim_invitations(user, Utc::now())
            .await
            .map_err(repository_error)?;
        self.repository
            .list_for_user(user.user_id)
            .await
            .map_err(repository_error)
    }

    pub async fn create(
        &self,
        user_id: UserId,
        tournament: NewTournament,
    ) -> Result<StoredTournament, ApiError> {
        self.repository
            .create(user_id, tournament, Utc::now())
            .await
            .map_err(repository_error)
    }

    pub async fn load_for_view(
        &self,
        user_id: UserId,
        tournament_id: Uuid,
    ) -> Result<(StoredTournament, TournamentAccessRole), ApiError> {
        let role = TournamentAccessPolicy::require_view(
            self.repository
                .access_role(tournament_id, user_id)
                .await
                .map_err(repository_error)?,
        )?;
        let stored = self
            .repository
            .load(tournament_id)
            .await
            .map_err(repository_error)?
            .ok_or(ApiError::NotFound)?;
        Ok((stored, role))
    }

    pub async fn load_for_edit(
        &self,
        user_id: UserId,
        tournament_id: Uuid,
    ) -> Result<(StoredTournament, TournamentAccessRole), ApiError> {
        let role = TournamentAccessPolicy::require_edit(
            self.repository
                .access_role(tournament_id, user_id)
                .await
                .map_err(repository_error)?,
        )?;
        let stored = self
            .repository
            .load(tournament_id)
            .await
            .map_err(repository_error)?
            .ok_or(ApiError::NotFound)?;
        Ok((stored, role))
    }

    pub async fn save(
        &self,
        user_id: UserId,
        stored: &mut StoredTournament,
        expected_revision: u64,
    ) -> Result<(), ApiError> {
        TournamentAccessPolicy::require_edit(
            self.repository
                .access_role(stored.id, user_id)
                .await
                .map_err(repository_error)?,
        )?;
        let revision = self
            .repository
            .save(stored, expected_revision, Utc::now())
            .await
            .map_err(repository_error)?;
        stored.revision = revision;
        Ok(())
    }

    pub async fn delete(
        &self,
        user_id: UserId,
        tournament_id: Uuid,
        expected_revision: u64,
    ) -> Result<(), ApiError> {
        self.require_owner(user_id, tournament_id).await?;
        self.repository
            .delete(tournament_id, expected_revision)
            .await
            .map_err(repository_error)
    }

    pub(super) async fn require_owner(
        &self,
        user_id: UserId,
        tournament_id: Uuid,
    ) -> Result<(), ApiError> {
        TournamentAccessPolicy::require_owner(
            self.repository
                .access_role(tournament_id, user_id)
                .await
                .map_err(repository_error)?,
        )?;
        Ok(())
    }
}

pub(super) fn repository_error(error: TournamentRepositoryError) -> ApiError {
    match error {
        TournamentRepositoryError::RevisionConflict => ApiError::RevisionConflict,
        TournamentRepositoryError::OwnerRoleImmutable => ApiError::invalid(
            "owner_role_immutable",
            "The tournament owner cannot be changed or removed.",
        ),
        TournamentRepositoryError::MemberNotFound
        | TournamentRepositoryError::InvitationNotFound => ApiError::NotFound,
        TournamentRepositoryError::Database(error) => ApiError::Database(error),
        TournamentRepositoryError::InvalidPairingJson(_)
        | TournamentRepositoryError::InvalidStoredData(_) => {
            tracing::error!(error = ?error, "invalid persisted tournament data");
            ApiError::Internal
        }
    }
}
