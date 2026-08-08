use chrono::{DateTime, Utc};
use sqlx::query::query;
use sqlx::row::Row;
use sqlx::transaction::Transaction;
use sqlx_postgres::Postgres;
use uuid::Uuid;

use crate::backend::auth::AuthenticatedUser;

use super::access::{access_role_name, normalize_email, parse_access_role};
use super::{ReceivedTournamentInvitation, TournamentRepository, TournamentRepositoryError};

impl TournamentRepository {
    pub async fn received_invitations(
        &self,
        user: &AuthenticatedUser,
    ) -> Result<Vec<ReceivedTournamentInvitation>, TournamentRepositoryError> {
        let rows = query::<Postgres>(
            "SELECT invitation.id, invitation.tournament_id,
                    tournament.domain_id AS tournament_title, invitation.role,
                    inviter.email AS invited_by_email,
                    inviter.display_name AS invited_by_display_name,
                    invitation.created_at
             FROM tournament_invitations AS invitation
             JOIN tournaments AS tournament ON tournament.id = invitation.tournament_id
             JOIN users AS inviter ON inviter.id = invitation.invited_by_user_id
             WHERE invitation.invited_email = $1
             ORDER BY invitation.created_at, invitation.id",
        )
        .bind(normalize_email(&user.email))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(ReceivedTournamentInvitation {
                    id: row.try_get("id")?,
                    tournament_id: row.try_get("tournament_id")?,
                    tournament_title: row.try_get("tournament_title")?,
                    role: parse_access_role(row.try_get("role")?)?,
                    invited_by_email: row.try_get("invited_by_email")?,
                    invited_by_display_name: row.try_get("invited_by_display_name")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn accept_invitation(
        &self,
        user: &AuthenticatedUser,
        invitation_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<(), TournamentRepositoryError> {
        let mut transaction = self.pool.begin().await?;
        let invitation = consume_invitation(
            &mut transaction,
            invitation_id,
            &normalize_email(&user.email),
        )
        .await?;
        query::<Postgres>(
            "INSERT INTO tournament_members (
                tournament_id, user_id, role, created_at, updated_at
             ) VALUES ($1, $2, $3, $4, $4)
             ON CONFLICT (tournament_id, user_id) DO NOTHING",
        )
        .bind(invitation.tournament_id)
        .bind(user.user_id.as_uuid())
        .bind(access_role_name(invitation.role))
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn decline_invitation(
        &self,
        user: &AuthenticatedUser,
        invitation_id: Uuid,
    ) -> Result<(), TournamentRepositoryError> {
        let result = query::<Postgres>(
            "DELETE FROM tournament_invitations
             WHERE id = $1 AND invited_email = $2",
        )
        .bind(invitation_id)
        .bind(normalize_email(&user.email))
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(TournamentRepositoryError::InvitationNotFound);
        }
        Ok(())
    }
}

struct ConsumedInvitation {
    tournament_id: Uuid,
    role: crate::api_contract::TournamentAccessRole,
}

async fn consume_invitation(
    transaction: &mut Transaction<'_, Postgres>,
    invitation_id: Uuid,
    invited_email: &str,
) -> Result<ConsumedInvitation, TournamentRepositoryError> {
    let row = query::<Postgres>(
        "DELETE FROM tournament_invitations
         WHERE id = $1 AND invited_email = $2
         RETURNING tournament_id, role",
    )
    .bind(invitation_id)
    .bind(invited_email)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(TournamentRepositoryError::InvitationNotFound)?;
    Ok(ConsumedInvitation {
        tournament_id: row.try_get("tournament_id")?,
        role: parse_access_role(row.try_get("role")?)?,
    })
}
