use chrono::Utc;
use uuid::Uuid;

use crate::backend::auth::UserId;
use crate::backend::persistence::{
    NewTournament, StoredTournament, TournamentRepository, TournamentRepositoryError,
    TournamentSummary,
};
use crate::backend::server::error::ApiError;

use super::tournament_access::TournamentAccessPolicy;

#[derive(Clone)]
pub struct TournamentService {
    repository: TournamentRepository,
}

impl TournamentService {
    pub const fn new(repository: TournamentRepository) -> Self {
        Self { repository }
    }

    pub async fn list(&self, user_id: UserId) -> Result<Vec<TournamentSummary>, ApiError> {
        self.repository
            .list_for_creator(user_id)
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

    pub async fn load_owned(
        &self,
        user_id: UserId,
        tournament_id: Uuid,
    ) -> Result<StoredTournament, ApiError> {
        let stored = self
            .repository
            .load(tournament_id)
            .await
            .map_err(repository_error)?
            .ok_or(ApiError::NotFound)?;
        TournamentAccessPolicy::require_creator(user_id, stored.created_by_user_id)?;
        Ok(stored)
    }

    pub async fn save(
        &self,
        stored: &mut StoredTournament,
        expected_revision: u64,
    ) -> Result<(), ApiError> {
        let revision = self
            .repository
            .save(stored, expected_revision, Utc::now())
            .await
            .map_err(repository_error)?;
        stored.revision = revision;
        Ok(())
    }
}

fn repository_error(error: TournamentRepositoryError) -> ApiError {
    match error {
        TournamentRepositoryError::RevisionConflict => ApiError::RevisionConflict,
        TournamentRepositoryError::Database(error) => ApiError::Database(error),
        TournamentRepositoryError::InvalidPairingJson(_)
        | TournamentRepositoryError::InvalidStoredData(_) => {
            tracing::error!(error = ?error, "invalid persisted tournament data");
            ApiError::Internal
        }
    }
}
