use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::HeaderValue;
use axum::http::Method;
use axum::http::header::CACHE_CONTROL;
use axum::http::header::ORIGIN;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::backend::application::{TournamentApiState, TournamentService};
use crate::backend::auth::{AuthRepository, AuthState, GoogleIdentityProvider, SessionCookie};
use crate::backend::persistence::{TournamentRepository, connect};
use crate::backend::server::config::ServerConfig;
use crate::backend::server::error::ApiError;

#[derive(Clone)]
struct OriginPolicy {
    trusted_origin: String,
}

pub async fn application_router() -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let config = ServerConfig::from_environment()?;
    let pool = connect(&config).await?;
    let provider = GoogleIdentityProvider::discover(
        config.google_client_id.clone(),
        config.google_client_secret.clone(),
        config.oauth_callback_url()?,
    )
    .await?;
    let auth_state = AuthState::new(
        AuthRepository::new(pool.clone()),
        Arc::new(provider),
        SessionCookie::new(config.secure_cookie()),
    );
    let tournament_state = TournamentApiState::new(
        auth_state.clone(),
        TournamentService::new(TournamentRepository::new(pool)),
    );
    let origin_policy = OriginPolicy {
        trusted_origin: config.app_base_url.origin().ascii_serialization(),
    };

    Ok(Router::new()
        .route(
            "/api/health",
            get(|| async { Json(json!({ "status": "ok" })) }),
        )
        .nest("/api/auth", crate::backend::auth::routes(auth_state))
        .merge(crate::backend::application::routes(tournament_state))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn(disable_api_caching))
        .layer(middleware::from_fn_with_state(
            origin_policy,
            enforce_mutation_origin,
        )))
}

async fn enforce_mutation_origin(
    State(policy): State<OriginPolicy>,
    request: Request,
    next: Next,
) -> Response {
    if !matches!(
        *request.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    ) {
        return next.run(request).await;
    }
    let valid_origin = request
        .headers()
        .get(ORIGIN)
        .and_then(|origin| origin.to_str().ok())
        .is_some_and(|origin| origin == policy.trusted_origin);
    if valid_origin {
        next.run(request).await
    } else {
        ApiError::Forbidden.into_response()
    }
}

async fn disable_api_caching(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
    response
}

#[cfg(test)]
mod tests {
    #[test]
    fn localhost_origin_is_exact() {
        let url = url::Url::parse("http://localhost:3000").unwrap();
        assert_eq!(url.origin().ascii_serialization(), "http://localhost:3000");
    }
}
