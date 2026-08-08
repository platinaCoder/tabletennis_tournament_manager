use chrono::{DateTime, Utc};
use sqlx::query::query;
use sqlx::row::Row;
use sqlx_postgres::Postgres;
use uuid::Uuid;

use crate::api_contract::TournamentAccessRole;
use crate::backend::auth::UserId;

use super::access::{access_role_name, normalize_email, parse_access_role};
use super::{
    TournamentInvitation, TournamentMember, TournamentRepository, TournamentRepositoryError,
    TournamentSharing,
};

impl TournamentRepository {
    pub async fn sharing(
        &self,
        tournament_id: Uuid,
    ) -> Result<TournamentSharing, TournamentRepositoryError> {
        let member_rows = query::<Postgres>(
            "SELECT app_user.id, app_user.email, app_user.display_name,
                    app_user.avatar_url, member.role
             FROM tournament_members AS member
             JOIN users AS app_user ON app_user.id = member.user_id
             WHERE member.tournament_id = $1
             ORDER BY CASE member.role
                        WHEN 'owner' THEN 0 WHEN 'editor' THEN 1 ELSE 2
                      END,
                      LOWER(app_user.email), app_user.id",
        )
        .bind(tournament_id)
        .fetch_all(&self.pool)
        .await?;
        let members = member_rows
            .into_iter()
            .map(|row| {
                Ok(TournamentMember {
                    user_id: UserId::from_uuid(row.try_get("id")?),
                    email: row.try_get("email")?,
                    display_name: row.try_get("display_name")?,
                    avatar_url: row.try_get("avatar_url")?,
                    role: parse_access_role(row.try_get("role")?)?,
                })
            })
            .collect::<Result<Vec<_>, TournamentRepositoryError>>()?;
        let invitation_rows = query::<Postgres>(
            "SELECT id, invited_email, role, created_at
             FROM tournament_invitations
             WHERE tournament_id = $1
             ORDER BY invited_email, id",
        )
        .bind(tournament_id)
        .fetch_all(&self.pool)
        .await?;
        let invitations = invitation_rows
            .into_iter()
            .map(|row| {
                Ok(TournamentInvitation {
                    id: row.try_get("id")?,
                    email: row.try_get("invited_email")?,
                    role: parse_access_role(row.try_get("role")?)?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect::<Result<Vec<_>, TournamentRepositoryError>>()?;
        Ok(TournamentSharing {
            members,
            invitations,
        })
    }

    pub async fn grant_access(
        &self,
        tournament_id: Uuid,
        invited_by_user_id: UserId,
        email: &str,
        role: TournamentAccessRole,
        now: DateTime<Utc>,
    ) -> Result<(), TournamentRepositoryError> {
        let email = normalize_email(email);
        let mut transaction = self.pool.begin().await?;
        let existing_user = query::<Postgres>(
            "SELECT id FROM users
             WHERE LOWER(email) = $1
             ORDER BY created_at, id
             LIMIT 1",
        )
        .bind(&email)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = existing_user {
            let user_id: Uuid = row.try_get("id")?;
            let owner = query::<Postgres>(
                "SELECT 1 FROM tournament_members
                 WHERE tournament_id = $1 AND user_id = $2 AND role = 'owner'",
            )
            .bind(tournament_id)
            .bind(user_id)
            .fetch_optional(&mut *transaction)
            .await?
            .is_some();
            if owner {
                return Err(TournamentRepositoryError::OwnerRoleImmutable);
            }
            query::<Postgres>(
                "INSERT INTO tournament_members (
                    tournament_id, user_id, role, created_at, updated_at
                 ) VALUES ($1, $2, $3, $4, $4)
                 ON CONFLICT (tournament_id, user_id) DO UPDATE SET
                    role = EXCLUDED.role,
                    updated_at = EXCLUDED.updated_at
                 WHERE tournament_members.role <> 'owner'",
            )
            .bind(tournament_id)
            .bind(user_id)
            .bind(access_role_name(role))
            .bind(now)
            .execute(&mut *transaction)
            .await?;
            query::<Postgres>(
                "DELETE FROM tournament_invitations
                 WHERE tournament_id = $1 AND invited_email = $2",
            )
            .bind(tournament_id)
            .bind(email)
            .execute(&mut *transaction)
            .await?;
        } else {
            query::<Postgres>(
                "INSERT INTO tournament_invitations (
                    id, tournament_id, invited_email, role, invited_by_user_id,
                    created_at, updated_at
                 ) VALUES ($1, $2, $3, $4, $5, $6, $6)
                 ON CONFLICT (tournament_id, invited_email) DO UPDATE SET
                    role = EXCLUDED.role,
                    invited_by_user_id = EXCLUDED.invited_by_user_id,
                    updated_at = EXCLUDED.updated_at",
            )
            .bind(Uuid::new_v4())
            .bind(tournament_id)
            .bind(email)
            .bind(access_role_name(role))
            .bind(invited_by_user_id.as_uuid())
            .bind(now)
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn update_member_role(
        &self,
        tournament_id: Uuid,
        member_user_id: UserId,
        role: TournamentAccessRole,
        now: DateTime<Utc>,
    ) -> Result<(), TournamentRepositoryError> {
        let result = query::<Postgres>(
            "UPDATE tournament_members
             SET role = $3, updated_at = $4
             WHERE tournament_id = $1 AND user_id = $2 AND role <> 'owner'",
        )
        .bind(tournament_id)
        .bind(member_user_id.as_uuid())
        .bind(access_role_name(role))
        .bind(now)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return self
                .member_mutation_error(tournament_id, member_user_id)
                .await;
        }
        Ok(())
    }

    pub async fn remove_member(
        &self,
        tournament_id: Uuid,
        member_user_id: UserId,
    ) -> Result<(), TournamentRepositoryError> {
        let result = query::<Postgres>(
            "DELETE FROM tournament_members
             WHERE tournament_id = $1 AND user_id = $2 AND role <> 'owner'",
        )
        .bind(tournament_id)
        .bind(member_user_id.as_uuid())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return self
                .member_mutation_error(tournament_id, member_user_id)
                .await;
        }
        Ok(())
    }

    pub async fn delete_invitation(
        &self,
        tournament_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), TournamentRepositoryError> {
        let result = query::<Postgres>(
            "DELETE FROM tournament_invitations
             WHERE tournament_id = $1 AND id = $2",
        )
        .bind(tournament_id)
        .bind(invitation_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(TournamentRepositoryError::InvitationNotFound);
        }
        Ok(())
    }

    async fn member_mutation_error(
        &self,
        tournament_id: Uuid,
        member_user_id: UserId,
    ) -> Result<(), TournamentRepositoryError> {
        let role = query::<Postgres>(
            "SELECT role FROM tournament_members
             WHERE tournament_id = $1 AND user_id = $2",
        )
        .bind(tournament_id)
        .bind(member_user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        match role
            .map(|row| row.try_get::<String, _>("role"))
            .transpose()?
        {
            Some(role) if role == "owner" => Err(TournamentRepositoryError::OwnerRoleImmutable),
            _ => Err(TournamentRepositoryError::MemberNotFound),
        }
    }
}
