use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Duration;
use cookie::{Cookie, SameSite};
use sha2::{Digest, Sha256};

pub const SESSION_COOKIE_NAME: &str = "ttt_session";
pub const SESSION_LIFETIME: Duration = Duration::days(14);

#[derive(Clone, Debug)]
pub struct SessionToken {
    raw: String,
    hash: Vec<u8>,
}

impl SessionToken {
    pub fn generate() -> Result<Self, getrandom::Error> {
        let mut material = [0_u8; 32];
        getrandom::fill(&mut material)?;
        let raw = URL_SAFE_NO_PAD.encode(material);
        Ok(Self {
            hash: hash_token(&raw),
            raw,
        })
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn hash(&self) -> &[u8] {
        &self.hash
    }
}

pub fn hash_token(raw: &str) -> Vec<u8> {
    Sha256::digest(raw.as_bytes()).to_vec()
}

#[derive(Clone, Debug)]
pub struct SessionCookie {
    secure: bool,
}

impl SessionCookie {
    pub const fn new(secure: bool) -> Self {
        Self { secure }
    }

    pub fn create(&self, raw_token: String) -> Cookie<'static> {
        Cookie::build((SESSION_COOKIE_NAME, raw_token))
            .http_only(true)
            .secure(self.secure)
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(cookie::time::Duration::days(14))
            .build()
    }

    pub fn remove(&self) -> Cookie<'static> {
        Cookie::build((SESSION_COOKIE_NAME, ""))
            .http_only(true)
            .secure(self.secure)
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(cookie::time::Duration::ZERO)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_session_is_stored_as_a_hash() {
        let token = SessionToken::generate().unwrap();
        assert_ne!(token.raw().as_bytes(), token.hash());
        assert_eq!(token.hash(), hash_token(token.raw()));
        assert!(token.raw().len() >= 43);
    }
}
