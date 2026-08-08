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
    #[error("the tournament owner role cannot be changed or removed")]
    OwnerRoleImmutable,
    #[error("the tournament member was not found")]
    MemberNotFound,
    #[error("the tournament invitation was not found")]
    InvitationNotFound,
    #[error("the invited user already has tournament access")]
    AlreadyMember,
}

impl From<crate::application::TournamentSnapshotError> for TournamentRepositoryError {
    fn from(value: crate::application::TournamentSnapshotError) -> Self {
        Self::InvalidStoredData(value.to_string())
    }
}
