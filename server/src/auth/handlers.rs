use std::sync::Arc;

use crate::api_contract::{AuthenticatedUserView, AuthenticationView};
use axum::extract::{Query, State};
use axum::http::header::{CACHE_CONTROL, COOKIE, LOCATION, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use super::google::IdentityProvider;
use super::oauth_attempt::{LOGIN_ATTEMPT_LIFETIME, NewOauthAttempt, safe_return_to};
use super::session::{SESSION_COOKIE_NAME, SessionToken, hash_token};
use super::{AuthRepository, SessionCookie};
use crate::backend::server::error::ApiError;

#[derive(Clone)]
pub struct AuthState {
    repository: AuthRepository,
    provider: Arc<dyn IdentityProvider>,
    session_cookie: SessionCookie,
}

impl AuthState {
    pub fn new(
        repository: AuthRepository,
        provider: Arc<dyn IdentityProvider>,
        session_cookie: SessionCookie,
    ) -> Self {
        Self {
            repository,
            provider,
            session_cookie,
        }
    }

    #[cfg(test)]
    pub fn without_external_provider(repository: AuthRepository) -> Self {
        Self {
            repository,
            provider: Arc::new(RejectingIdentityProvider),
            session_cookie: SessionCookie::new(false),
        }
    }

    pub async fn authenticated_user(
        &self,
        headers: &HeaderMap,
    ) -> Result<Option<super::AuthenticatedUser>, ApiError> {
        let Some(token) = raw_session_cookie(headers) else {
            return Ok(None);
        };
        self.repository
            .resolve_session(&token, Utc::now())
            .await
            .map_err(ApiError::from)
    }
}

#[cfg(test)]
struct RejectingIdentityProvider;

#[cfg(test)]
#[async_trait::async_trait]
impl IdentityProvider for RejectingIdentityProvider {
    fn begin_login(
        &self,
    ) -> Result<super::google::GoogleAuthorization, super::google::GoogleIdentityError> {
        Ok(super::google::GoogleAuthorization {
            authorization_url: url::Url::parse("https://accounts.google.com").unwrap(),
            state: "unused".to_owned(),
            pkce_verifier: "unused".to_owned(),
            nonce: "unused".to_owned(),
        })
    }

    async fn authenticate(
        &self,
        _authorization_code: String,
        _pkce_verifier: String,
        _expected_nonce: String,
    ) -> Result<
        super::authenticated_user::ExternalIdentityProfile,
        super::google::GoogleIdentityError,
    > {
        Err(super::google::GoogleIdentityError::TokenExchange)
    }
}

pub fn routes(state: AuthState) -> Router {
    Router::new()
        .route("/google/login", get(login))
        .route("/google/callback", get(callback))
        .route("/me", get(me))
        .route("/logout", post(logout))
        .with_state(state)
}

#[derive(Deserialize)]
struct LoginQuery {
    return_to: Option<String>,
}

async fn login(
    State(state): State<AuthState>,
    Query(query): Query<LoginQuery>,
) -> Result<Response, ApiError> {
    let authorization = state
        .provider
        .begin_login()
        .map_err(|_| ApiError::Internal)?;
    let now = Utc::now();
    state
        .repository
        .create_oauth_attempt(&NewOauthAttempt {
            id: Uuid::new_v4(),
            state_hash: hash_token(&authorization.state),
            pkce_verifier: authorization.pkce_verifier,
            oidc_nonce: authorization.nonce,
            return_to: safe_return_to(query.return_to.as_deref()),
            created_at: now,
            expires_at: now + LOGIN_ATTEMPT_LIFETIME,
        })
        .await?;
    redirect(authorization.authorization_url.as_str())
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

async fn callback(
    State(state): State<AuthState>,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, ApiError> {
    if query.error.is_some() {
        return Err(ApiError::AuthenticationFailed);
    }
    let state_value = query.state.ok_or(ApiError::AuthenticationFailed)?;
    let code = query.code.ok_or(ApiError::AuthenticationFailed)?;
    let attempt = state
        .repository
        .consume_oauth_attempt(&state_value, Utc::now())
        .await
        .map_err(|_| ApiError::AuthenticationFailed)?;
    let profile = state
        .provider
        .authenticate(code, attempt.pkce_verifier, attempt.oidc_nonce)
        .await
        .map_err(|_| ApiError::AuthenticationFailed)?;
    let session = SessionToken::generate().map_err(|_| ApiError::Internal)?;
    state
        .repository
        .upsert_identity_and_create_session(&profile, &session, Utc::now())
        .await?;
    let cookie = state.session_cookie.create(session.raw().to_owned());
    let mut response = redirect(&attempt.return_to)?;
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|_| ApiError::Internal)?,
    );
    no_store(&mut response);
    Ok(response)
}

async fn me(State(state): State<AuthState>, headers: HeaderMap) -> Result<Response, ApiError> {
    let user = state.authenticated_user(&headers).await?;
    let view = AuthenticationView {
        authenticated: user.is_some(),
        user: user.map(|user| AuthenticatedUserView {
            id: user.user_id.as_uuid().to_string(),
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
        }),
    };
    let mut response = Json(view).into_response();
    no_store(&mut response);
    Ok(response)
}

async fn logout(State(state): State<AuthState>, headers: HeaderMap) -> Result<Response, ApiError> {
    if let Some(token) = raw_session_cookie(&headers) {
        state.repository.invalidate_session(&token).await?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    let cookie = state.session_cookie.remove();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|_| ApiError::Internal)?,
    );
    no_store(&mut response);
    Ok(response)
}

fn raw_session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .filter_map(|value| cookie::Cookie::parse(value.trim()).ok())
        .find(|cookie| cookie.name() == SESSION_COOKIE_NAME)
        .map(|cookie| cookie.value().to_owned())
}

fn redirect(destination: &str) -> Result<Response, ApiError> {
    let location = HeaderValue::from_str(destination).map_err(|_| ApiError::Internal)?;
    let mut response = StatusCode::SEE_OTHER.into_response();
    response.headers_mut().insert(LOCATION, location);
    no_store(&mut response);
    Ok(response)
}

fn no_store(response: &mut Response) {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
}
