use crate::api_contract::ApiErrorView;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("authentication is required")]
    Unauthorized,
    #[error("the request was forbidden")]
    Forbidden,
    #[error("the requested resource was not found")]
    NotFound,
    #[error("the request conflicts with newer server state")]
    RevisionConflict,
    #[error("the match result was changed by another editor")]
    ResultRevisionConflict,
    #[error("invalid request: {message}")]
    InvalidRequest { code: &'static str, message: String },
    #[error("authentication could not be completed")]
    AuthenticationFailed,
    #[error("database operation failed")]
    Database(#[source] sqlx::Error),
    #[error("an internal server error occurred")]
    Internal,
}

impl ApiError {
    pub fn invalid(code: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            code,
            message: message.into(),
        }
    }

    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::RevisionConflict => (StatusCode::CONFLICT, "revision_conflict"),
            Self::ResultRevisionConflict => (StatusCode::CONFLICT, "result_revision_conflict"),
            Self::InvalidRequest { code, .. } => (StatusCode::BAD_REQUEST, code),
            Self::AuthenticationFailed => (StatusCode::BAD_REQUEST, "authentication_failed"),
            Self::Database(_) | Self::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_server_error")
            }
        }
    }

    fn public_message(&self) -> String {
        match self {
            Self::Database(_) | Self::Internal => {
                "The server could not complete the request.".to_owned()
            }
            other => other.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if matches!(self, Self::Database(_) | Self::Internal) {
            tracing::error!(error = ?self, "API request failed");
        }
        let (status, code) = self.status_and_code();
        let body = ApiErrorView {
            code: code.to_owned(),
            message: self.public_message(),
        };
        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}
