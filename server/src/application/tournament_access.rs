use crate::api_contract::TournamentAccessRole;
use crate::backend::server::error::ApiError;

pub struct TournamentAccessPolicy;

impl TournamentAccessPolicy {
    pub fn require_view(
        role: Option<TournamentAccessRole>,
    ) -> Result<TournamentAccessRole, ApiError> {
        // Do not reveal whether a tournament exists to a non-member.
        role.ok_or(ApiError::NotFound)
    }

    pub fn require_edit(
        role: Option<TournamentAccessRole>,
    ) -> Result<TournamentAccessRole, ApiError> {
        let role = Self::require_view(role)?;
        if role.can_edit() {
            Ok(role)
        } else {
            Err(ApiError::Forbidden)
        }
    }

    pub fn require_owner(
        role: Option<TournamentAccessRole>,
    ) -> Result<TournamentAccessRole, ApiError> {
        let role = Self::require_view(role)?;
        if role.is_owner() {
            Ok(role)
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_member_is_inaccessible() {
        assert!(matches!(
            TournamentAccessPolicy::require_view(None),
            Err(ApiError::NotFound)
        ));
    }

    #[test]
    fn editor_can_edit_but_viewer_cannot() {
        assert_eq!(
            TournamentAccessPolicy::require_edit(Some(TournamentAccessRole::Editor)).unwrap(),
            TournamentAccessRole::Editor
        );
        assert!(matches!(
            TournamentAccessPolicy::require_edit(Some(TournamentAccessRole::Viewer)),
            Err(ApiError::Forbidden)
        ));
    }

    #[test]
    fn only_owner_can_manage_access() {
        assert_eq!(
            TournamentAccessPolicy::require_owner(Some(TournamentAccessRole::Owner)).unwrap(),
            TournamentAccessRole::Owner
        );
        assert!(matches!(
            TournamentAccessPolicy::require_owner(Some(TournamentAccessRole::Editor)),
            Err(ApiError::Forbidden)
        ));
    }
}
