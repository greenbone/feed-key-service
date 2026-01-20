// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::{jwt::validate_token, service::app::AppState};

pub async fn authorization_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let headers = request.headers();
    let auth_header = headers
        .get("Authorization")
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized"))?;
    let auth_str = auth_header
        .to_str()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Unauthorized"))?;
    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized"))?;

    match validate_token(&state.jwt_secret, token) {
        Ok(_) => Ok(next.run(request).await),
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Unauthorized")),
    }
}
