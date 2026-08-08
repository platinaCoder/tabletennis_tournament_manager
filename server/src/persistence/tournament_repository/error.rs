#[derive(Debug, thiserror::Error)]
pub enum TournamentRepositoryError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("stored tournament data is invalid: {0}")]
    InvalidStoredData(String),
    #[error("stored pairing JSON is invalid")]
    InvalidPairingJson(#[from] serde_json::Error),
    #[error("the tournament revision changed")]
    RevisionConflict,
}

impl From<crate::application::TournamentSnapshotError> for TournamentRepositoryError {
    fn from(value: crate::application::TournamentSnapshotError) -> Self {
        Self::InvalidStoredData(value.to_string())
    }
}
