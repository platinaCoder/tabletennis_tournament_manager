use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

pub const LOGIN_ATTEMPT_LIFETIME: Duration = Duration::minutes(10);

#[derive(Clone, Debug)]
pub struct NewOauthAttempt {
    pub id: Uuid,
    pub state_hash: Vec<u8>,
    pub pkce_verifier: String,
    pub oidc_nonce: String,
    pub return_to: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct ConsumedOauthAttempt {
    pub pkce_verifier: String,
    pub oidc_nonce: String,
    pub return_to: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum OauthAttemptError {
    #[error("OAuth state is invalid")]
    StateMismatch,
    #[error("OAuth login attempt has expired")]
    Expired,
    #[error("OAuth login attempt was already consumed")]
    AlreadyConsumed,
}

pub fn safe_return_to(value: Option<&str>) -> String {
    value
        .filter(|candidate| is_safe_relative_path(candidate))
        .unwrap_or("/")
        .to_owned()
}

pub fn validate_attempt_lifetime(
    expires_at: DateTime<Utc>,
    consumed_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Result<(), OauthAttemptError> {
    if consumed_at.is_some() {
        Err(OauthAttemptError::AlreadyConsumed)
    } else if expires_at <= now {
        Err(OauthAttemptError::Expired)
    } else {
        Ok(())
    }
}

fn is_safe_relative_path(value: &str) -> bool {
    value.starts_with('/')
        && !value.starts_with("//")
        && !value.contains('\r')
        && !value.contains('\n')
        && url::Url::parse(value).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_and_protocol_relative_redirects_are_rejected() {
        assert_eq!(safe_return_to(Some("https://attacker.example")), "/");
        assert_eq!(safe_return_to(Some("//attacker.example/path")), "/");
        assert_eq!(safe_return_to(Some("/dev")), "/dev");
    }

    #[test]
    fn expired_attempt_is_rejected() {
        let now = Utc::now();
        assert_eq!(
            validate_attempt_lifetime(now, None, now),
            Err(OauthAttemptError::Expired)
        );
    }

    #[test]
    fn consumed_attempt_cannot_be_consumed_twice() {
        let now = Utc::now();
        assert_eq!(
            validate_attempt_lifetime(now + LOGIN_ATTEMPT_LIFETIME, Some(now), now),
            Err(OauthAttemptError::AlreadyConsumed)
        );
    }
}
