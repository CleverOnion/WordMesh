//! JWT token utilities for access/refresh issuance and validation.

use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub scope: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Clone)]
pub struct TokenConfig {
    pub algorithm: Algorithm,
    pub access_ttl_secs: u64,
    pub refresh_ttl_secs: Option<u64>,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub issuer: Option<String>,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("token generation failed: {0}")]
    Encode(jsonwebtoken::errors::Error),
    #[error("token validation failed: {0}")]
    Decode(jsonwebtoken::errors::Error),
    #[error("refresh token not enabled")]
    RefreshDisabled,
}

fn header_for(config: &TokenConfig) -> Header {
    let mut header = Header::new(config.algorithm);
    if let Some(iss) = &config.issuer {
        header.kid = Some(iss.clone());
    }
    header
}

pub fn generate_access_token(
    config: &TokenConfig,
    subject: &str,
    scope: Option<String>,
    request_id: Option<String>,
) -> Result<String, TokenError> {
    let issued_at = Utc::now().timestamp();
    let exp = issued_at + config.access_ttl_secs as i64;
    let claims = Claims {
        sub: subject.to_string(),
        exp,
        iat: issued_at,
        scope,
        request_id,
    };
    jsonwebtoken::encode(&header_for(config), &claims, &config.encoding_key)
        .map_err(TokenError::Encode)
}

pub fn generate_refresh_token(
    config: &TokenConfig,
    subject: &str,
    request_id: Option<String>,
) -> Result<String, TokenError> {
    let ttl = config.refresh_ttl_secs.ok_or(TokenError::RefreshDisabled)?;
    let issued_at = Utc::now().timestamp();
    let exp = issued_at + ttl as i64;
    let claims = Claims {
        sub: subject.to_string(),
        exp,
        iat: issued_at,
        scope: None,
        request_id,
    };
    jsonwebtoken::encode(&header_for(config), &claims, &config.encoding_key)
        .map_err(TokenError::Encode)
}

pub fn validate_token(config: &TokenConfig, token: &str) -> Result<Claims, TokenError> {
    let mut validation = Validation::new(config.algorithm);
    validation.validate_exp = true;
    if let Some(iss) = &config.issuer {
        validation.set_issuer(&[iss.as_str()]);
    }
    jsonwebtoken::decode::<Claims>(token, &config.decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(TokenError::Decode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::Algorithm;

    fn test_config() -> TokenConfig {
        let secret = b"0123456789abcdef0123456789abcdef";
        TokenConfig {
            algorithm: Algorithm::HS256,
            access_ttl_secs: 60,
            refresh_ttl_secs: Some(120),
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            issuer: Some("wordmesh".into()),
        }
    }

    #[test]
    fn access_token_round_trip() {
        let config = test_config();
        let token = generate_access_token(
            &config,
            "user-1",
            Some("scope".into()),
            Some("req-1".into()),
        )
        .unwrap();
        let claims = validate_token(&config, &token).unwrap();
        assert_eq!(claims.sub, "user-1");
        assert!(claims.exp > claims.iat);
        assert_eq!(claims.scope.as_deref(), Some("scope"));
        assert_eq!(claims.request_id.as_deref(), Some("req-1"));
    }

    #[test]
    fn refresh_token_round_trip() {
        let config = test_config();
        let token = generate_refresh_token(&config, "user-1", None).unwrap();
        let claims = validate_token(&config, &token).unwrap();
        assert_eq!(claims.sub, "user-1");
        assert!(claims.exp > claims.iat);
        assert!(claims.scope.is_none());
    }

    #[test]
    fn refresh_disabled_error() {
        let mut config = test_config();
        config.refresh_ttl_secs = None;
        let err = generate_refresh_token(&config, "user", None).unwrap_err();
        assert!(matches!(err, TokenError::RefreshDisabled));
    }
}
