use sqlx::query::query;
use sqlx::row::Row;
use sqlx_postgres::Postgres;
use uuid::Uuid;

use crate::api_contract::TournamentAccessRole;
use crate::backend::auth::UserId;

use super::{TournamentRepository, TournamentRepositoryError, TournamentSummary};

impl TournamentRepository {
    pub async fn access_role(
        &self,
        tournament_id: Uuid,
        user_id: UserId,
    ) -> Result<Option<TournamentAccessRole>, TournamentRepositoryError> {
        let row = query::<Postgres>(
            "SELECT role FROM tournament_members
             WHERE tournament_id = $1 AND user_id = $2",
        )
        .bind(tournament_id)
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| parse_access_role(row.try_get("role")?))
            .transpose()
    }

    pub async fn list_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<TournamentSummary>, TournamentRepositoryError> {
        let rows = query::<Postgres>(
            "SELECT tournament.id, tournament.domain_id, tournament.status,
                    tournament.revision, tournament.updated_at, member.role
             FROM tournament_members AS member
             JOIN tournaments AS tournament ON tournament.id = member.tournament_id
             WHERE member.user_id = $1
             ORDER BY tournament.updated_at DESC, tournament.id",
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(TournamentSummary {
                    id: row.try_get("id")?,
                    title: row.try_get("domain_id")?,
                    status: row.try_get("status")?,
                    revision: u64::try_from(row.try_get::<i64, _>("revision")?).map_err(invalid)?,
                    access_role: parse_access_role(row.try_get("role")?)?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn delete(
        &self,
        tournament_id: Uuid,
        expected_revision: u64,
    ) -> Result<(), TournamentRepositoryError> {
        let expected_revision = i64::try_from(expected_revision)
            .map_err(|_| invalid("tournament revision exceeds storage limit"))?;
        let result = query::<Postgres>("DELETE FROM tournaments WHERE id = $1 AND revision = $2")
            .bind(tournament_id)
            .bind(expected_revision)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(TournamentRepositoryError::RevisionConflict);
        }
        Ok(())
    }
}

pub(super) fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub(super) fn parse_access_role(
    value: &str,
) -> Result<TournamentAccessRole, TournamentRepositoryError> {
    match value {
        "owner" => Ok(TournamentAccessRole::Owner),
        "editor" => Ok(TournamentAccessRole::Editor),
        "viewer" => Ok(TournamentAccessRole::Viewer),
        _ => Err(invalid("unknown tournament access role")),
    }
}

pub(super) const fn access_role_name(role: TournamentAccessRole) -> &'static str {
    match role {
        TournamentAccessRole::Owner => "owner",
        TournamentAccessRole::Editor => "editor",
        TournamentAccessRole::Viewer => "viewer",
    }
}

fn invalid(error: impl std::fmt::Display) -> TournamentRepositoryError {
    TournamentRepositoryError::InvalidStoredData(error.to_string())
}
