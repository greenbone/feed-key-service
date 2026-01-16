// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{Router, routing::get};
use utoipa::OpenApi;

use crate::app::AppRouter;

const HEALTH_TAG: &str = "Health";

#[derive(OpenApi)]
#[openapi(
    info(description = "Health check endpoint", title = "Health API"),
    tags((name = HEALTH_TAG, description = "Health check operations")),
    paths(health_check)
)]
pub struct HealthApi;

// Check the health status of the service.
#[utoipa::path(
  get,
  path = "",
  responses(
    (status = 200, description = "Health check OK", body = String)
  ),
  tag = HEALTH_TAG,
)]
async fn health_check() -> &'static str {
    "OK server is healthy"
}

pub fn routes() -> AppRouter {
    Router::new().route("/", get(health_check))
}
