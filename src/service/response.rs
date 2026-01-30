// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{Json, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct JsonResponse {
    status: String,
    message: String,
}

impl JsonResponse {
    pub fn from_error(msg: &str) -> Self {
        JsonResponse {
            message: msg.to_string(),
            status: "error".to_string(),
        }
    }

    pub fn from_success(msg: &str) -> Self {
        JsonResponse {
            message: msg.to_string(),
            status: "success".to_string(),
        }
    }
}

impl IntoResponse for JsonResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_response_from_error() {
        let error_message = "An error occurred";
        let response = JsonResponse::from_error(error_message);
        assert_eq!(response.status, "error");
        assert_eq!(response.message, error_message);
    }

    #[test]
    fn test_json_response_from_success() {
        let success_message = "Operation successful";
        let response = JsonResponse::from_success(success_message);
        assert_eq!(response.status, "success");
        assert_eq!(response.message, success_message);
    }

    #[test]
    fn test_json_response_serialization_error() {
        let message = "Test message";
        let response = JsonResponse::from_error(message);
        let serialized = serde_json::to_string(&response).unwrap();
        let expected = format!(r#"{{"status":"error","message":"{}"}}"#, message);
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_json_response_serialization_success() {
        let message = "Test success message";
        let response = JsonResponse::from_success(message);
        let serialized = serde_json::to_string(&response).unwrap();
        let expected = format!(r#"{{"status":"success","message":"{}"}}"#, message);
        assert_eq!(serialized, expected);
    }

    #[tokio::test]
    async fn test_json_response_into_response_success() {
        let message = "Test into response";
        let response = JsonResponse::from_success(message);
        let axum_response = response.into_response();
        let body = axum::body::to_bytes(axum_response.into_body(), 4096)
            .await
            .unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let expected_json = json!({
            "status": "success",
            "message": "Test into response"
        });

        assert_eq!(body_json, expected_json);
    }

    #[tokio::test]
    async fn test_json_response_into_response_error() {
        let message = "Test into response error";
        let response = JsonResponse::from_error(message);
        let axum_response = response.into_response();
        let body = axum::body::to_bytes(axum_response.into_body(), 4096)
            .await
            .unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let expected_json = json!({
            "status": "error",
            "message": "Test into response error"
        });
        assert_eq!(body_json, expected_json);
    }
}
