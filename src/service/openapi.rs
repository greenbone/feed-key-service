// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{Router, response::Redirect, routing::get};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::service::app::AppRouter;

struct SecuritySchemas;

impl Modify for SecuritySchemas {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        let scheme = SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build(),
        );
        components.add_security_scheme("jwt_auth", scheme);
    }
}

#[derive(OpenApi)]
#[openapi(
    info(description = "Greenbone Feed Key API", title = "Greenbone Feed Key"),
    modifiers(&SecuritySchemas),
    nest(
        (path = "/api/v1/health", api = crate::service::api::health::HealthApi),
        (path = "/api/v1/key", api = crate::service::api::key::KeyApi),
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

pub fn generate_openapi_json() -> Result<String, serde_json::Error> {
    ApiDoc::openapi().to_pretty_json()
}
