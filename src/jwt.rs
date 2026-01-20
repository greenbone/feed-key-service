// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user name)
    pub exp: u64,    // Expiration Time as Unix timestamp
    pub iat: u64,    // Issued at as Unix timestamp
}

#[allow(unused)]
impl Claims {
    pub fn new(sub: String, expiration: Duration) -> Self {
        let now = Utc::now();
        let exp = (now + expiration).timestamp() as u64;
        Claims {
            sub,
            exp,
            iat: now.timestamp() as u64,
        }
    }
}

#[derive(Clone)]
pub enum JwtSecret {
    /// A shared secret for HMAC algorithms.
    SharedSecret(String),
    // An RSA public key (decoding) or private key (encoding) of in PEM format.
    RsaKey(Vec<u8>),
    // An ECDSA public key (decoding) or private key (encoding) in PEM format.
    EcdsaKey(Vec<u8>),
}

pub fn validate_token(
    secret: &JwtSecret,
    token: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let (decoding_key, validation) = match secret {
        JwtSecret::SharedSecret(key) => (
            jsonwebtoken::DecodingKey::from_secret(key.as_bytes()),
            jsonwebtoken::Validation::default(),
        ),
        JwtSecret::RsaKey(pem) => (
            jsonwebtoken::DecodingKey::from_rsa_pem(pem.as_slice())?,
            jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256),
        ),
        JwtSecret::EcdsaKey(pem) => (
            jsonwebtoken::DecodingKey::from_ec_pem(pem.as_slice())?,
            jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::ES256),
        ),
    };
    let token_data = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

#[allow(unused)]
pub fn generate_token(
    secret: &JwtSecret,
    claims: &Claims,
) -> Result<String, jsonwebtoken::errors::Error> {
    let (encoding_key, header) = match secret {
        JwtSecret::SharedSecret(key) => (
            jsonwebtoken::EncodingKey::from_secret(key.as_bytes()),
            jsonwebtoken::Header::default(),
        ),
        JwtSecret::RsaKey(pem) => (
            jsonwebtoken::EncodingKey::from_rsa_pem(pem.as_slice())?,
            jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        ),
        JwtSecret::EcdsaKey(pem) => (
            jsonwebtoken::EncodingKey::from_ec_pem(pem.as_slice())?,
            jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        ),
    };
    jsonwebtoken::encode(&header, &claims, &encoding_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_secret() {
        let secret = JwtSecret::SharedSecret("my_secret".to_string());
        let claims = Claims::new("test_user".to_string(), Duration::minutes(10));
        let token = generate_token(&secret, &claims).expect("Failed to generate token");
        let decoded_claims = validate_token(&secret, &token).expect("Failed to validate token");
        assert_eq!(decoded_claims.sub, claims.sub);
        assert_eq!(decoded_claims.exp, claims.exp);
        assert_eq!(decoded_claims.iat, claims.iat);
    }

    #[test]
    fn test_rsa_secret() {
        let encode_secret = JwtSecret::RsaKey(include_bytes!("../tests/rsa-private.pem").to_vec());
        let decode_secret = JwtSecret::RsaKey(include_bytes!("../tests/rsa-public.pem").to_vec());
        let claims = Claims::new("test_user".to_string(), Duration::minutes(10));
        let token = generate_token(&encode_secret, &claims).expect("Failed to generate token");
        let decoded_claims =
            validate_token(&decode_secret, &token).expect("Failed to validate token");
        assert_eq!(decoded_claims.sub, claims.sub);
        assert_eq!(decoded_claims.exp, claims.exp);
        assert_eq!(decoded_claims.iat, claims.iat);
    }

    #[test]
    fn test_ecdsa_secret() {
        let encode_secret =
            JwtSecret::EcdsaKey(include_bytes!("../tests/ecdsa-private.pem").to_vec());
        let decode_secret =
            JwtSecret::EcdsaKey(include_bytes!("../tests/ecdsa-public.pem").to_vec());
        let claims = Claims::new("test_user".to_string(), Duration::minutes(10));
        let token = generate_token(&encode_secret, &claims).expect("Failed to generate token");
        let decoded_claims =
            validate_token(&decode_secret, &token).expect("Failed to validate token");
        assert_eq!(decoded_claims.sub, claims.sub);
        assert_eq!(decoded_claims.exp, claims.exp);
        assert_eq!(decoded_claims.iat, claims.iat);
    }

    #[test]
    fn test_expired_token() {
        let secret = JwtSecret::SharedSecret("my_secret".to_string());
        let claims = Claims::new("test_user".to_string(), Duration::seconds(-100));
        let token = generate_token(&secret, &claims).expect("Failed to generate token");
        let result = validate_token(&secret, &token);
        assert!(result.is_err());
    }
}
