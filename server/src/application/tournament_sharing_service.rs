use chrono::Utc;
use uuid::Uuid;

use crate::api_contract::TournamentAccessRole;
use crate::backend::auth::{AuthenticatedUser, UserId};
use crate::backend::persistence::TournamentSharing;
use crate::backend::server::error::ApiError;

use super::tournament_service::{TournamentService, repository_error};

impl TournamentService {
    pub async fn sharing(
        &self,
        user_id: UserId,
        tournament_id: Uuid,
    ) -> Result<TournamentSharing, ApiError> {
        self.require_owner(user_id, tournament_id).await?;
        self.repository
            .sharing(tournament_id)
            .await
            .map_err(repository_error)
    }

    pub async fn grant_access(
        &self,
        owner: &AuthenticatedUser,
        tournament_id: Uuid,
        email: &str,
        role: TournamentAccessRole,
    ) -> Result<TournamentSharing, ApiError> {
        self.require_owner(owner.user_id, tournament_id).await?;
        validate_share_role(role)?;
        validate_email(email)?;
        if email.trim().eq_ignore_ascii_case(&owner.email) {
            return Err(ApiError::invalid(
                "owner_role_immutable",
                "The tournament owner already has permanent owner access.",
            ));
        }
        self.repository
            .grant_access(tournament_id, owner.user_id, email, role, Utc::now())
            .await
            .map_err(repository_error)?;
        self.repository
            .sharing(tournament_id)
            .await
            .map_err(repository_error)
    }

    pub async fn update_member_role(
        &self,
        owner_id: UserId,
        tournament_id: Uuid,
        member_user_id: UserId,
        role: TournamentAccessRole,
    ) -> Result<TournamentSharing, ApiError> {
        self.require_owner(owner_id, tournament_id).await?;
        validate_share_role(role)?;
        self.repository
            .update_member_role(tournament_id, member_user_id, role, Utc::now())
            .await
            .map_err(repository_error)?;
        self.repository
            .sharing(tournament_id)
            .await
            .map_err(repository_error)
    }

    pub async fn remove_member(
        &self,
        owner_id: UserId,
        tournament_id: Uuid,
        member_user_id: UserId,
    ) -> Result<TournamentSharing, ApiError> {
        self.require_owner(owner_id, tournament_id).await?;
        self.repository
            .remove_member(tournament_id, member_user_id)
            .await
            .map_err(repository_error)?;
        self.repository
            .sharing(tournament_id)
            .await
            .map_err(repository_error)
    }

    pub async fn delete_invitation(
        &self,
        owner_id: UserId,
        tournament_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<TournamentSharing, ApiError> {
        self.require_owner(owner_id, tournament_id).await?;
        self.repository
            .delete_invitation(tournament_id, invitation_id)
            .await
            .map_err(repository_error)?;
        self.repository
            .sharing(tournament_id)
            .await
            .map_err(repository_error)
    }
}

fn validate_share_role(role: TournamentAccessRole) -> Result<(), ApiError> {
    if role.is_owner() {
        Err(ApiError::invalid(
            "invalid_shared_role",
            "Shared access must be editor or viewer.",
        ))
    } else {
        Ok(())
    }
}

fn validate_email(email: &str) -> Result<(), ApiError> {
    let email = email.trim();
    let valid = email.len() <= 320
        && !email.is_empty()
        && !email.chars().any(char::is_whitespace)
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && domain.contains('.') && !domain.starts_with('.')
        });
    if valid {
        Ok(())
    } else {
        Err(ApiError::invalid(
            "invalid_share_email",
            "Enter a valid email address.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_role_rejects_owner() {
        assert!(validate_share_role(TournamentAccessRole::Editor).is_ok());
        assert!(validate_share_role(TournamentAccessRole::Viewer).is_ok());
        assert!(validate_share_role(TournamentAccessRole::Owner).is_err());
    }

    #[test]
    fn share_email_validation_rejects_malformed_values() {
        assert!(validate_email("manager@example.com").is_ok());
        assert!(validate_email("missing-domain@").is_err());
        assert!(validate_email("contains space@example.com").is_err());
    }
}
