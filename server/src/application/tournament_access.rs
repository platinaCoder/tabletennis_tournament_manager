use crate::backend::auth::UserId;
use crate::backend::server::error::ApiError;

pub struct TournamentAccessPolicy;

impl TournamentAccessPolicy {
    pub fn require_creator(
        authenticated_user_id: UserId,
        created_by_user_id: UserId,
    ) -> Result<(), ApiError> {
        if authenticated_user_id == created_by_user_id {
            Ok(())
        } else {
            // Do not reveal whether another user's tournament exists.
            Err(ApiError::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn another_users_tournament_is_inaccessible() {
        assert!(matches!(
            TournamentAccessPolicy::require_creator(UserId::new(), UserId::new()),
            Err(ApiError::NotFound)
        ));
    }
}
