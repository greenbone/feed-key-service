// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    extract::{Request, State},
    http,
    middleware::Next,
    response::Response,
};
use gvm_auth::jwt::validate_token;

use crate::{service::app::AppState, service::error::Error};

pub async fn authorization_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Error> {
    let headers = request.headers();
    let auth_header = match headers.get(http::header::AUTHORIZATION) {
        Some(header) => header,
        None => {
            tracing::debug!("Missing Authorization header {:?}", request);
            return Err(Error::Unauthorized);
        }
    };
    let auth_str = auth_header.to_str().map_err(|e| {
        tracing::debug!("Unable to convert Authorization header into string: {}", e);
        Error::Unauthorized
    })?;
    let token = match auth_str.strip_prefix("Bearer ") {
        Some(token) => token,
        None => {
            tracing::debug!("Could not extract JWT token from Authorization header");
            return Err(Error::Unauthorized);
        }
    };

    match validate_token(&state.jwt_decode_secret, token) {
        Ok(_) => Ok(next.run(request).await),
        Err(e) => {
            tracing::debug!("Invalid JWT token {}", e);
            Err(Error::Unauthorized)
        }
    }
}
