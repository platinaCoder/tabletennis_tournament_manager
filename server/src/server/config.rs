use std::env;
use std::num::NonZeroU32;

use url::Url;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub database_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub app_base_url: Url,
    pub database_max_connections: NonZeroU32,
}

impl ServerConfig {
    pub fn from_environment() -> Result<Self, ConfigError> {
        let app_base_url = required("APP_BASE_URL")?;
        let app_base_url =
            Url::parse(&app_base_url).map_err(|source| ConfigError::InvalidBaseUrl { source })?;
        validate_base_url(&app_base_url)?;

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "4".to_owned())
            .parse::<u32>()
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or(ConfigError::InvalidPoolSize)?;

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            google_client_id: required("GOOGLE_CLIENT_ID")?,
            google_client_secret: required("GOOGLE_CLIENT_SECRET")?,
            app_base_url,
            database_max_connections,
        })
    }

    pub fn oauth_callback_url(&self) -> Result<Url, ConfigError> {
        self.app_base_url
            .join("api/auth/google/callback")
            .map_err(|source| ConfigError::InvalidBaseUrl { source })
    }

    pub fn secure_cookie(&self) -> bool {
        self.app_base_url.scheme() == "https"
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingVariable { name })
}

fn validate_base_url(url: &Url) -> Result<(), ConfigError> {
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ConfigError::UnsupportedBaseUrl);
    }
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err(ConfigError::UnsupportedBaseUrl);
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("required environment variable {name} is missing")]
    MissingVariable { name: &'static str },
    #[error("APP_BASE_URL is invalid")]
    InvalidBaseUrl { source: url::ParseError },
    #[error("APP_BASE_URL must be an HTTP(S) origin without a path, query or fragment")]
    UnsupportedBaseUrl,
    #[error("DATABASE_MAX_CONNECTIONS must be a positive integer")]
    InvalidPoolSize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_base_url_must_be_an_origin() {
        assert!(validate_base_url(&Url::parse("https://example.test").unwrap()).is_ok());
        assert!(validate_base_url(&Url::parse("https://example.test/app").unwrap()).is_err());
        assert!(validate_base_url(&Url::parse("https://example.test?next=/dev").unwrap()).is_err());
    }
}
