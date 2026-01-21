// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{jwt::validate_token, service::app::AppState, service::error::Error};

pub async fn authorization_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Error> {
    let headers = request.headers();
    let auth_header = headers.get("Authorization").ok_or(Error::Unauthorized)?;
    let auth_str = auth_header.to_str().map_err(|_| Error::Unauthorized)?;
    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or(Error::Unauthorized)?;

    match validate_token(&state.jwt_secret, token) {
        Ok(_) => Ok(next.run(request).await),
        Err(_) => Err(Error::Unauthorized),
    }
}
