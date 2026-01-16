// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{Router, response::Redirect, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app::AppRouter;

#[derive(OpenApi)]
#[openapi(
    info(description = "Greenbone Feed Key API", title = "Greenbone Feed Key"),
    nest(
        (path = "/api/v1/health", api = crate::api::health::HealthApi),
        (path = "/api/v1/key", api = crate::api::key::KeyApi),
    )
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/api/v1/openapi.json",
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
pub fn routes() -> AppRouter {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/swagger-ui") }))
        .merge(SwaggerUi::new("/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()))
}
