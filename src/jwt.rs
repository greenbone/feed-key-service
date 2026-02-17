// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
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

#[derive(Clone, Debug)]
pub enum JwtEncodeSecret {
    /// A shared secret for HMAC algorithms.
    SharedSecret(EncodingKey, Algorithm),
    // An RSA private key in PEM format.
    RsaKey(EncodingKey, Algorithm),
    // An ECDSA private key in PEM format.
    EcdsaKey(EncodingKey, Algorithm),
}

impl JwtEncodeSecret {
    pub fn from_shared_secret(secret: &str) -> Self {
        JwtEncodeSecret::SharedSecret(
            EncodingKey::from_secret(secret.as_bytes()),
            Algorithm::HS256,
        )
    }

    pub fn from_rsa_pem(pem: &[u8]) -> Result<Self, jsonwebtoken::errors::Error> {
        Ok(JwtEncodeSecret::RsaKey(
            EncodingKey::from_rsa_pem(pem)?,
            Algorithm::RS256,
        ))
    }

    pub fn from_ec_pem(pem: &[u8]) -> Result<Self, jsonwebtoken::errors::Error> {
        Ok(JwtEncodeSecret::EcdsaKey(
            EncodingKey::from_ec_pem(pem)?,
            Algorithm::ES256,
        ))
    }
}

#[derive(Clone, Debug)]
pub enum JwtDecodeSecret {
    /// A shared secret for HMAC algorithms.
    SharedSecret(DecodingKey, Algorithm),
    // An RSA public key in PEM format.
    RsaKey(DecodingKey, Algorithm),
    // An ECDSA public key in PEM format.
    EcdsaKey(DecodingKey, Algorithm),
}

impl JwtDecodeSecret {
    pub fn from_shared_secret(secret: &str) -> Self {
        JwtDecodeSecret::SharedSecret(
            DecodingKey::from_secret(secret.as_bytes()),
            Algorithm::HS256,
        )
    }

    pub fn from_rsa_pem(pem: &[u8]) -> Result<Self, jsonwebtoken::errors::Error> {
        Ok(JwtDecodeSecret::RsaKey(
            DecodingKey::from_rsa_pem(pem)?,
            Algorithm::RS256,
        ))
    }

    pub fn from_ec_pem(pem: &[u8]) -> Result<Self, jsonwebtoken::errors::Error> {
        Ok(JwtDecodeSecret::EcdsaKey(
            DecodingKey::from_ec_pem(pem)?,
            Algorithm::ES256,
        ))
    }
}

pub fn validate_token(
    secret: &JwtDecodeSecret,
    token: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let (decoding_key, validation) = match secret {
        JwtDecodeSecret::SharedSecret(key, alg) => (key, Validation::new(*alg)),
        JwtDecodeSecret::RsaKey(key, alg) => (key, Validation::new(*alg)),
        JwtDecodeSecret::EcdsaKey(key, alg) => (key, Validation::new(*alg)),
    };
    let token_data = jsonwebtoken::decode::<Claims>(token, decoding_key, &validation)?;
    Ok(token_data.claims)
}

#[allow(unused)]
pub fn generate_token(
    secret: &JwtEncodeSecret,
    claims: &Claims,
) -> Result<String, jsonwebtoken::errors::Error> {
    let (encoding_key, header) = match secret {
        JwtEncodeSecret::SharedSecret(key, alg) => (key, Header::new(*alg)),
        JwtEncodeSecret::RsaKey(key, alg) => (key, Header::new(*alg)),
        JwtEncodeSecret::EcdsaKey(key, alg) => (key, Header::new(*alg)),
    };
    jsonwebtoken::encode(&header, &claims, encoding_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_secret() {
        let secret = "my_secret".to_string();
        let decode_secret = JwtDecodeSecret::from_shared_secret(&secret);
        let encode_secret = JwtEncodeSecret::from_shared_secret(&secret);
        let claims = Claims::new("test_user".to_string(), Duration::minutes(10));
        let token = generate_token(&encode_secret, &claims).expect("Failed to generate token");
        let decoded_claims =
            validate_token(&decode_secret, &token).expect("Failed to validate token");
        assert_eq!(decoded_claims.sub, claims.sub);
        assert_eq!(decoded_claims.exp, claims.exp);
        assert_eq!(decoded_claims.iat, claims.iat);
    }

    #[test]
    fn test_rsa_secret() {
        let encode_secret =
            JwtEncodeSecret::from_rsa_pem(include_bytes!("../tests/rsa-private.pem"))
                .expect("Failed to create encode secret");
        let decode_secret =
            JwtDecodeSecret::from_rsa_pem(include_bytes!("../tests/rsa-public.pem"))
                .expect("Failed to create decode secret");
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
            JwtEncodeSecret::from_ec_pem(include_bytes!("../tests/ecdsa-private.pem"))
                .expect("Failed to create encode secret");
        let decode_secret =
            JwtDecodeSecret::from_ec_pem(include_bytes!("../tests/ecdsa-public.pem"))
                .expect("Failed to create decode secret");
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
        let secret = "my_secret".to_string();
        let decode_secret = JwtDecodeSecret::from_shared_secret(&secret);
        let encode_secret = JwtEncodeSecret::from_shared_secret(&secret);
        let claims = Claims::new("test_user".to_string(), Duration::seconds(-100));
        let token = generate_token(&encode_secret, &claims).expect("Failed to generate token");
        let result = validate_token(&decode_secret, &token);
        assert!(result.is_err());
    }
}
