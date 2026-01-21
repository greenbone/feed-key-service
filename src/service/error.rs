// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unauthorized")]
    Unauthorized,
}

#[derive(Serialize)]
struct JsonErrorResponse {
    error: String,
}

impl JsonErrorResponse {
    fn new(msg: String) -> Self {
        JsonErrorResponse { error: msg }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message): (StatusCode, String) = match self {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, Error::Unauthorized.to_string()),
        };
        (status, Json(JsonErrorResponse::new(error_message))).into_response()
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

        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let expected_json = json!({
            "error": "Unauthorized"
        });

        assert_eq!(body_json, expected_json);
    }
}
