use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use openid::{DiscoveredClient, Options, Token};
use url::Url;

use super::authenticated_user::ExternalIdentityProfile;

#[derive(Clone)]
pub struct GoogleIdentityProvider {
    client: DiscoveredClient,
}

impl GoogleIdentityProvider {
    pub async fn discover(
        client_id: String,
        client_secret: String,
        redirect_url: Url,
    ) -> Result<Self, GoogleIdentityError> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| GoogleIdentityError::HttpClient)?;
        let issuer = Url::parse("https://accounts.google.com")
            .map_err(|_| GoogleIdentityError::IssuerConfiguration)?;
        let client = DiscoveredClient::discover_with_client(
            http_client,
            client_id,
            Some(client_secret),
            Some(redirect_url.to_string()),
            issuer,
        )
        .await
        .map_err(|_| GoogleIdentityError::Discovery)?;
        Ok(Self { client })
    }
}

#[async_trait]
pub(crate) trait IdentityProvider: Send + Sync {
    fn begin_login(&self) -> Result<GoogleAuthorization, GoogleIdentityError>;

    async fn authenticate(
        &self,
        authorization_code: String,
        pkce_verifier: String,
        expected_nonce: String,
    ) -> Result<ExternalIdentityProfile, GoogleIdentityError>;
}

#[async_trait]
impl IdentityProvider for GoogleIdentityProvider {
    fn begin_login(&self) -> Result<GoogleAuthorization, GoogleIdentityError> {
        let state = random_protocol_value()?;
        let nonce = random_protocol_value()?;
        let (authorization_url, pkce) = self.client.auth_url_with_new_pkce(&Options {
            scope: Some("openid email profile".to_owned()),
            state: Some(state.clone()),
            nonce: Some(nonce.clone()),
            ..Options::default()
        });
        Ok(GoogleAuthorization {
            authorization_url,
            state,
            pkce_verifier: pkce.code_verifier().to_owned(),
            nonce,
        })
    }

    async fn authenticate(
        &self,
        authorization_code: String,
        pkce_verifier: String,
        expected_nonce: String,
    ) -> Result<ExternalIdentityProfile, GoogleIdentityError> {
        let bearer = self
            .client
            .request_token_pkce(&authorization_code, Some(&pkce_verifier))
            .await
            .map_err(|_| GoogleIdentityError::TokenExchange)?;
        let mut token = Token::from(bearer);
        let id_token = token
            .id_token
            .as_mut()
            .ok_or(GoogleIdentityError::MissingIdToken)?;
        self.client
            .decode_token(id_token)
            .map_err(|_| GoogleIdentityError::InvalidIdToken)?;
        self.client
            .validate_token(id_token, Some(expected_nonce.as_str()), None)
            .map_err(|_| GoogleIdentityError::InvalidIdToken)?;
        let claims = id_token
            .payload()
            .map_err(|_| GoogleIdentityError::InvalidIdToken)?;
        let user = &claims.userinfo;

        if !user.email_verified {
            return Err(GoogleIdentityError::UnverifiedEmail);
        }
        let email = user
            .email
            .clone()
            .ok_or(GoogleIdentityError::MissingEmail)?
            .trim()
            .to_owned();
        if email.is_empty() {
            return Err(GoogleIdentityError::MissingEmail);
        }
        Ok(ExternalIdentityProfile {
            provider: "google",
            provider_subject: user.sub.clone(),
            email,
            display_name: user.name.clone(),
            avatar_url: user.picture.as_ref().map(ToString::to_string),
        })
    }
}

fn random_protocol_value() -> Result<String, GoogleIdentityError> {
    let mut material = [0_u8; 32];
    getrandom::fill(&mut material).map_err(|_| GoogleIdentityError::SecureRandom)?;
    Ok(URL_SAFE_NO_PAD.encode(material))
}

pub(crate) struct GoogleAuthorization {
    pub authorization_url: Url,
    pub state: String,
    pub pkce_verifier: String,
    pub nonce: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum GoogleIdentityError {
    #[error("could not initialize the OIDC HTTP client")]
    HttpClient,
    #[error("Google OIDC issuer configuration is invalid")]
    IssuerConfiguration,
    #[error("Google OIDC discovery failed")]
    Discovery,
    #[error("secure OAuth random material could not be generated")]
    SecureRandom,
    #[error("Google authorization-code exchange failed")]
    TokenExchange,
    #[error("Google did not return an ID token")]
    MissingIdToken,
    #[error("Google returned an invalid ID token")]
    InvalidIdToken,
    #[error("Google did not return an email address")]
    MissingEmail,
    #[error("Google email address is not verified")]
    UnverifiedEmail,
}
