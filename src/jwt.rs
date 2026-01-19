// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,                  // Subject (user name)
    pub exp: u64,                     // Expiration Time as Unix timestamp
    pub iat: u64,                     // Issued at as Unix timestamp
    pub roles: HashSet<String>,       // User roles
    pub permissions: HashSet<String>, // User permissions
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
            roles: HashSet::new(),
            permissions: HashSet::new(),
        }
    }
}

#[derive(Clone)]
#[allow(unused)]
pub enum JwtDecodeSecret {
    /// A shared secret for HMAC algorithms.
    SharedSecret(String),
    // An RSA public key in PEM format.
    RsaKey(Vec<u8>),
    // An ECDSA public key in PEM format.
    EcdsaKey(Vec<u8>),
}

pub fn validate_token(
    secret: &JwtDecodeSecret,
    token: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = match secret {
        JwtDecodeSecret::SharedSecret(key) => {
            jsonwebtoken::DecodingKey::from_secret(key.as_bytes())
        }
        JwtDecodeSecret::RsaKey(pem) => jsonwebtoken::DecodingKey::from_rsa_pem(pem.as_slice())?,
        JwtDecodeSecret::EcdsaKey(pem) => jsonwebtoken::DecodingKey::from_ec_pem(pem.as_slice())?,
    };
    let validation = jsonwebtoken::Validation::default();
    let token_data = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

#[derive(Clone)]
#[allow(unused)]
pub enum JwtEncodeSecret {
    /// A shared secret for HMAC algorithms.
    SharedSecret(String),
    // An RSA private key in PEM format.
    RsaKey(Vec<u8>),
    // An ECDSA private key in PEM format.
    EcdsaKey(Vec<u8>),
}

#[allow(unused)]
pub fn generate_token(
    secret: &JwtEncodeSecret,
    claims: &Claims,
) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = match secret {
        JwtEncodeSecret::SharedSecret(key) => {
            jsonwebtoken::EncodingKey::from_secret(key.as_bytes())
        }
        JwtEncodeSecret::RsaKey(pem) => jsonwebtoken::EncodingKey::from_rsa_pem(pem.as_slice())?,
        JwtEncodeSecret::EcdsaKey(pem) => jsonwebtoken::EncodingKey::from_ec_pem(pem.as_slice())?,
    };
    jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &encoding_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_hmac() {
        let secret = JwtEncodeSecret::SharedSecret("my_secret".to_string());
        let claims = Claims::new("test_user".to_string(), Duration::minutes(10));
        let token = generate_token(&secret, &claims).expect("Failed to generate token");
        let decoded_claims = validate_token(
            &JwtDecodeSecret::SharedSecret("my_secret".to_string()),
            &token,
        )
        .expect("Failed to validate token");
        assert_eq!(decoded_claims.sub, claims.sub);
        assert_eq!(decoded_claims.exp, claims.exp);
        assert_eq!(decoded_claims.iat, claims.iat);
    }
}
