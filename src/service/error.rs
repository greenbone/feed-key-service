// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::service::response::JsonResponse;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Key deletion failed")]
    KeyDeletionFailed,
    #[error("Key not available")]
    KeyNotFound,
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message): (StatusCode, String) = match self {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, Error::Unauthorized.to_string()),
            Error::KeyDeletionFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Error::KeyDeletionFailed.to_string(),
            ),
            Error::KeyNotFound => (StatusCode::NOT_FOUND, Error::KeyNotFound.to_string()),
            Error::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Error::InternalServerError(msg).to_string(),
            ),
            Error::BadRequest(msg) => (StatusCode::BAD_REQUEST, Error::BadRequest(msg).to_string()),
        };
        (status, JsonResponse::from_error(&error_message)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::Response;
    use serde_json::json;

    #[tokio::test]
    async fn test_unauthorized_error_response() {
        let error = Error::Unauthorized;
        let response: Response = error.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = response.into_body();
        let body = axum::body::to_bytes(body, 4096).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let expected_json = json!({
            "status": "error",
            "message": "Unauthorized"
        });

        assert_eq!(body_json, expected_json);
    }
}
