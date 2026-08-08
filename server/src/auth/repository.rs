use chrono::{DateTime, Utc};
use sqlx::query::query;
use sqlx::row::Row;
use sqlx::transaction::Transaction;
use sqlx_postgres::{PgPool, Postgres};
use uuid::Uuid;

use super::authenticated_user::ExternalIdentityProfile;
use super::oauth_attempt::{
    ConsumedOauthAttempt, NewOauthAttempt, OauthAttemptError, validate_attempt_lifetime,
};
use super::session::{SESSION_LIFETIME, SessionToken, hash_token};
use super::{AuthenticatedUser, UserId};

#[derive(Clone)]
pub struct AuthRepository {
    pool: PgPool,
}

impl AuthRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_oauth_attempt(&self, attempt: &NewOauthAttempt) -> Result<(), sqlx::Error> {
        query::<Postgres>("DELETE FROM oauth_login_attempts WHERE expires_at <= $1")
            .bind(attempt.created_at)
            .execute(&self.pool)
            .await?;
        query::<Postgres>(
            "INSERT INTO oauth_login_attempts (
                id, state_hash, pkce_verifier, oidc_nonce, return_to,
                created_at, expires_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(attempt.id)
        .bind(&attempt.state_hash)
        .bind(&attempt.pkce_verifier)
        .bind(&attempt.oidc_nonce)
        .bind(&attempt.return_to)
        .bind(attempt.created_at)
        .bind(attempt.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn consume_oauth_attempt(
        &self,
        state: &str,
        now: DateTime<Utc>,
    ) -> Result<ConsumedOauthAttempt, ConsumeOauthAttemptError> {
        let state_hash = hash_token(state);
        let mut transaction = self.pool.begin().await?;
        let row = query::<Postgres>(
            "SELECT pkce_verifier, oidc_nonce, return_to, expires_at, consumed_at
             FROM oauth_login_attempts
             WHERE state_hash = $1
             FOR UPDATE",
        )
        .bind(state_hash)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(OauthAttemptError::StateMismatch)?;
        let expires_at: DateTime<Utc> = row.try_get("expires_at")?;
        let consumed_at: Option<DateTime<Utc>> = row.try_get("consumed_at")?;
        validate_attempt_lifetime(expires_at, consumed_at, now)?;
        query::<Postgres>(
            "UPDATE oauth_login_attempts
             SET consumed_at = $2
             WHERE state_hash = $1",
        )
        .bind(hash_token(state))
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(ConsumedOauthAttempt {
            pkce_verifier: row.try_get("pkce_verifier")?,
            oidc_nonce: row.try_get("oidc_nonce")?,
            return_to: row.try_get("return_to")?,
        })
    }

    pub async fn upsert_identity_and_create_session(
        &self,
        profile: &ExternalIdentityProfile,
        session: &SessionToken,
        now: DateTime<Utc>,
    ) -> Result<AuthenticatedUser, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let user_id = find_identity_user(&mut transaction, profile).await?;
        let user_id = match user_id {
            Some(user_id) => {
                update_user(&mut transaction, user_id, profile, now).await?;
                user_id
            }
            None => create_user_and_identity(&mut transaction, profile, now).await?,
        };
        query::<Postgres>(
            "INSERT INTO auth_sessions (
                id, user_id, token_hash, created_at, last_seen_at, expires_at
             ) VALUES ($1, $2, $3, $4, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(user_id.as_uuid())
        .bind(session.hash())
        .bind(now)
        .bind(now + SESSION_LIFETIME)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(AuthenticatedUser {
            user_id,
            email: profile.email.clone(),
            display_name: profile.display_name.clone(),
            avatar_url: profile.avatar_url.clone(),
        })
    }

    pub async fn resolve_session(
        &self,
        raw_token: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<AuthenticatedUser>, sqlx::Error> {
        query::<Postgres>("DELETE FROM auth_sessions WHERE expires_at <= $1")
            .bind(now)
            .execute(&self.pool)
            .await?;
        let row = query::<Postgres>(
            "UPDATE auth_sessions AS session
             SET last_seen_at = $2
             FROM users AS app_user
             WHERE session.token_hash = $1
               AND session.expires_at > $2
               AND app_user.id = session.user_id
             RETURNING app_user.id, app_user.email, app_user.display_name,
                       app_user.avatar_url",
        )
        .bind(hash_token(raw_token))
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;
        row.map(user_from_row).transpose()
    }

    pub async fn invalidate_session(&self, raw_token: &str) -> Result<bool, sqlx::Error> {
        let result = query::<Postgres>("DELETE FROM auth_sessions WHERE token_hash = $1")
            .bind(hash_token(raw_token))
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() != 0)
    }
}

async fn find_identity_user(
    transaction: &mut Transaction<'_, Postgres>,
    profile: &ExternalIdentityProfile,
) -> Result<Option<UserId>, sqlx::Error> {
    let row = query::<Postgres>(
        "SELECT user_id FROM auth_identities
         WHERE provider = $1 AND provider_subject = $2
         FOR UPDATE",
    )
    .bind(profile.provider)
    .bind(&profile.provider_subject)
    .fetch_optional(&mut **transaction)
    .await?;
    row.map(|row| row.try_get::<Uuid, _>("user_id").map(UserId::from_uuid))
        .transpose()
}

async fn update_user(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    profile: &ExternalIdentityProfile,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    query::<Postgres>(
        "UPDATE users
         SET email = $2, display_name = $3, avatar_url = $4,
             updated_at = $5, last_login_at = $5
         WHERE id = $1",
    )
    .bind(user_id.as_uuid())
    .bind(&profile.email)
    .bind(&profile.display_name)
    .bind(&profile.avatar_url)
    .bind(now)
    .execute(&mut **transaction)
    .await?;
    query::<Postgres>(
        "UPDATE auth_identities
         SET provider_email = $3, updated_at = $4
         WHERE provider = $1 AND provider_subject = $2",
    )
    .bind(profile.provider)
    .bind(&profile.provider_subject)
    .bind(&profile.email)
    .bind(now)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn create_user_and_identity(
    transaction: &mut Transaction<'_, Postgres>,
    profile: &ExternalIdentityProfile,
    now: DateTime<Utc>,
) -> Result<UserId, sqlx::Error> {
    let user_id = UserId::new();
    query::<Postgres>(
        "INSERT INTO users (
            id, email, display_name, avatar_url,
            created_at, updated_at, last_login_at
         ) VALUES ($1, $2, $3, $4, $5, $5, $5)",
    )
    .bind(user_id.as_uuid())
    .bind(&profile.email)
    .bind(&profile.display_name)
    .bind(&profile.avatar_url)
    .bind(now)
    .execute(&mut **transaction)
    .await?;
    query::<Postgres>(
        "INSERT INTO auth_identities (
            id, user_id, provider, provider_subject, provider_email,
            created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id.as_uuid())
    .bind(profile.provider)
    .bind(&profile.provider_subject)
    .bind(&profile.email)
    .bind(now)
    .execute(&mut **transaction)
    .await?;
    Ok(user_id)
}

fn user_from_row(row: sqlx_postgres::PgRow) -> Result<AuthenticatedUser, sqlx::Error> {
    Ok(AuthenticatedUser {
        user_id: UserId::from_uuid(row.try_get("id")?),
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
        avatar_url: row.try_get("avatar_url")?,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ConsumeOauthAttemptError {
    #[error(transparent)]
    Attempt(#[from] OauthAttemptError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[cfg(test)]
mod postgres_tests {
    use chrono::Duration;
    use sqlx_postgres::PgPoolOptions;

    use super::*;
    use crate::backend::persistence::migrate_test_database;

    async fn repository() -> AuthRepository {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .unwrap();
        migrate_test_database(&pool).await.unwrap();
        AuthRepository::new(pool)
    }

    #[tokio::test]
    #[ignore = "requires an isolated TEST_DATABASE_URL PostgreSQL database"]
    async fn oauth_attempt_rejects_wrong_state_expiry_and_second_consumption() {
        let repository = repository().await;
        let now = Utc::now();
        let state = format!("state-{}", Uuid::new_v4());
        repository
            .create_oauth_attempt(&NewOauthAttempt {
                id: Uuid::new_v4(),
                state_hash: hash_token(&state),
                pkce_verifier: "verifier".to_owned(),
                oidc_nonce: "nonce".to_owned(),
                return_to: "/dev".to_owned(),
                created_at: now,
                expires_at: now + Duration::minutes(10),
            })
            .await
            .unwrap();
        assert!(matches!(
            repository.consume_oauth_attempt("wrong-state", now).await,
            Err(ConsumeOauthAttemptError::Attempt(
                OauthAttemptError::StateMismatch
            ))
        ));
        repository.consume_oauth_attempt(&state, now).await.unwrap();
        assert!(matches!(
            repository.consume_oauth_attempt(&state, now).await,
            Err(ConsumeOauthAttemptError::Attempt(
                OauthAttemptError::AlreadyConsumed
            ))
        ));

        let expired_state = format!("expired-{}", Uuid::new_v4());
        query::<Postgres>(
            "INSERT INTO oauth_login_attempts (
                id, state_hash, pkce_verifier, oidc_nonce, return_to,
                created_at, expires_at
             ) VALUES ($1, $2, 'verifier', 'nonce', '/', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(hash_token(&expired_state))
        .bind(now - Duration::minutes(20))
        .bind(now - Duration::minutes(10))
        .execute(&repository.pool)
        .await
        .unwrap();
        assert!(matches!(
            repository.consume_oauth_attempt(&expired_state, now).await,
            Err(ConsumeOauthAttemptError::Attempt(
                OauthAttemptError::Expired
            ))
        ));
    }

    #[tokio::test]
    #[ignore = "requires an isolated TEST_DATABASE_URL PostgreSQL database"]
    async fn session_resolves_and_logout_invalidates_it() {
        let repository = repository().await;
        let subject = Uuid::new_v4().to_string();
        let profile = ExternalIdentityProfile {
            provider: "google",
            provider_subject: subject,
            email: format!("{}@test.invalid", Uuid::new_v4()),
            display_name: Some("Test User".to_owned()),
            avatar_url: None,
        };
        let token = SessionToken::generate().unwrap();
        let user = repository
            .upsert_identity_and_create_session(&profile, &token, Utc::now())
            .await
            .unwrap();
        assert_eq!(
            repository
                .resolve_session(token.raw(), Utc::now())
                .await
                .unwrap()
                .unwrap()
                .user_id,
            user.user_id
        );
        assert!(repository.invalidate_session(token.raw()).await.unwrap());
        assert!(
            repository
                .resolve_session(token.raw(), Utc::now())
                .await
                .unwrap()
                .is_none()
        );
        let expired = SessionToken::generate().unwrap();
        let now = Utc::now();
        query::<Postgres>(
            "INSERT INTO auth_sessions (
                id, user_id, token_hash, created_at, last_seen_at, expires_at
             ) VALUES ($1, $2, $3, $4, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(user.user_id.as_uuid())
        .bind(expired.hash())
        .bind(now - Duration::days(2))
        .bind(now - Duration::days(1))
        .execute(&repository.pool)
        .await
        .unwrap();
        assert!(
            repository
                .resolve_session(expired.raw(), now)
                .await
                .unwrap()
                .is_none()
        );
        query::<Postgres>("DELETE FROM users WHERE id = $1")
            .bind(user.user_id.as_uuid())
            .execute(&repository.pool)
            .await
            .unwrap();
    }
}
