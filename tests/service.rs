// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use chrono::Duration;
use cucumber::{World, given, then, when};
use greenbone_feed_key::service::app::App;
use gvm_auth::jwt::{Claims, JwtDecodeSecret, JwtEncodeSecret, generate_token};
use rust_multipart_rfc7578_2::client::multipart::Body as MultipartBody;
use rust_multipart_rfc7578_2::client::multipart::Form as MultipartForm;
use tower::ServiceExt;

const HEALTH_API: &str = "/api/v1/health";
const KEY_API: &str = "/api/v1/key";
const OPENAPI_DOCS_API: &str = "/api/v1/openapi.json";
const SWAGGER_UI_URL: &str = "/swagger-ui/";
const SHARED_SECRET: &str = "some-secret";
const KEY_CONTENT: &str = "SOME-ENTERPRISE-FEED-KEY";

#[derive(Debug, Default, World)]
struct ServiceWorld {
    app: Option<App>,
    authenticated: bool,
    response: Option<Response>,
    response_json: Option<serde_json::Value>,
    encode_secret: Option<JwtEncodeSecret>,
    decode_secret: Option<JwtDecodeSecret>,
    tempfile_path: Option<PathBuf>,
}

#[given("the service is running")]
fn service_is_running(world: &mut ServiceWorld) {
    let encode_secret = JwtEncodeSecret::from_shared_secret(SHARED_SECRET);
    let decode_secret = JwtDecodeSecret::from_shared_secret(SHARED_SECRET);
    let tempfile = tempfile::Builder::new()
        .prefix("feed_key_")
        .tempfile()
        .expect("Could not create temporary feed key file")
        .into_temp_path();
    let app = App::new(&tempfile, None, &decode_secret, false);
    world.tempfile_path = Some(tempfile.to_path_buf());
    world.app = Some(app);
    world.encode_secret = Some(encode_secret);
    world.decode_secret = Some(decode_secret);
    world.authenticated = false;
}

#[given("a feed key exists in the system")]
fn a_feed_key_exists_in_the_system(world: &mut ServiceWorld) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    std::fs::write(path, KEY_CONTENT).expect("Could not write feed key to temporary file");
}

#[given("no feed key exists in the system")]
fn no_feed_key_exists_in_the_system(world: &mut ServiceWorld) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    let _ = std::fs::remove_file(path);
}

#[given("the API documentation is enabled")]
fn the_api_documentation_feature_is_enabled(world: &mut ServiceWorld) {
    let app = world.app.as_ref().expect("service not available");
    world.app = Some(app.enable_api_documentation());
}

#[given("the user is authenticated")]
fn the_user_is_authenticated(world: &mut ServiceWorld) {
    world.authenticated = true;
}

#[given("the feed key file is not writable")]
fn the_key_file_is_not_writable(world: &mut ServiceWorld) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    let mut permissions = std::fs::metadata(path)
        .expect("Could not get file metadata")
        .permissions();
    permissions.set_readonly(true);
    tracing::info!("Setting feed key file {:?} to read-only", path);
    std::fs::set_permissions(path, permissions).expect("Could not set file permissions");
}

#[when(
    regex = r"^I send a (GET|DELETE|POST|PUT) request to the (key endpoint|health endpoint|API documentation|swagger UI)$"
)]
async fn i_send_a_request(world: &mut ServiceWorld, method: String, endpoint: String) {
    let builder = Request::builder();
    let builder = match endpoint.as_str() {
        "key endpoint" => builder.uri(KEY_API),
        "health endpoint" => builder.uri(HEALTH_API),
        "API documentation" => builder.uri(OPENAPI_DOCS_API),
        "swagger UI" => builder.uri(SWAGGER_UI_URL),
        _ => panic!("Unsupported endpoint"),
    };
    let builder = match method.as_str() {
        "GET" => builder.method("GET"),
        "DELETE" => builder.method("DELETE"),
        "POST" => builder.method("POST"),
        "PUT" => builder.method("PUT"),
        _ => panic!("Unsupported method"),
    };
    let builder = if world.authenticated {
        let jwt = generate_token(
            world.encode_secret.as_ref().expect("secret not available"),
            &Claims::new("test_user".to_string(), Duration::minutes(10)),
        )
        .unwrap();
        builder.header("Authorization", format!("Bearer {}", jwt))
    } else {
        builder
    };
    let request = builder
        .body(Body::empty())
        .expect("Could not build request body");
    let router = world.app.as_ref().expect("service not available").router();
    let result = router.oneshot(request).await.expect("Request failed");
    world.response = Some(result);
}

#[when(regex = r"^I upload the feed key '(.+)' via a (POST|PUT) request to the key endpoint$")]
async fn i_send_an_upload_request(world: &mut ServiceWorld, body: String, method: String) {
    let builder = Request::builder().uri(KEY_API);
    let builder = match method.as_str() {
        "POST" => builder.method("POST"),
        "PUT" => builder.method("PUT"),
        _ => panic!("Unsupported method"),
    };
    let builder = if world.authenticated {
        let jwt = generate_token(
            world.encode_secret.as_ref().expect("secret not available"),
            &Claims::new("test_user".to_string(), Duration::minutes(10)),
        )
        .unwrap();
        builder.header("Authorization", format!("Bearer {}", jwt))
    } else {
        builder
    };
    let request = builder
        .body(Body::from(body))
        .expect("Could not build request body");
    let router = world.app.as_ref().expect("service not available").router();
    let result = router.oneshot(request).await.expect("Request failed");
    world.response = Some(result);
}

#[when(regex = r"^I post the field '(.+)' with content '(.+)' to the key endpoint$")]
async fn i_post_the_field_with_content_to_the_key_endpoint(
    world: &mut ServiceWorld,
    field_name: String,
    field_content: String,
) {
    let builder = Request::builder().uri(KEY_API).method("POST");
    let builder = if world.authenticated {
        let jwt = generate_token(
            world.encode_secret.as_ref().expect("secret not available"),
            &Claims::new("test_user".to_string(), Duration::minutes(10)),
        )
        .unwrap();
        builder.header("Authorization", format!("Bearer {}", jwt))
    } else {
        builder
    };
    let mut form = MultipartForm::default();
    form.add_text(field_name, field_content);
    let content_type = form.content_type();
    let multipart_body = MultipartBody::from(form);
    let request = builder
        .header("Content-Type", content_type)
        .body(Body::from_stream(multipart_body))
        .expect("Could not build request body");
    let router = world.app.as_ref().expect("service not available").router();
    let result = router.oneshot(request).await.expect("Request failed");
    world.response = Some(result);
}

#[then(expr = "the response status code should be {int}")]
fn the_response_status_code_should_be(world: &mut ServiceWorld, status_code: u16) {
    let response = world.response.as_ref().expect("No response stored");
    assert_eq!(
        response.status(),
        StatusCode::from_u16(status_code).unwrap()
    );
}

#[then(expr = "the response body should be {string}")]
async fn the_response_body_should_be_ok(world: &mut ServiceWorld, expected_body: String) {
    let response = world.response.take().expect("No response stored");
    let (req_parts, req_body) = response.into_parts();
    let body_bytes = axum::body::to_bytes(req_body, usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();

    assert_eq!(body_str, expected_body);

    let req = Response::from_parts(req_parts, Body::from(body_bytes));
    world.response = Some(req);
}

#[then("the response body should be valid JSON")]
async fn the_response_body_should_be_valid_json(world: &mut ServiceWorld) {
    let response = world.response.take().expect("No response stored");
    let (req_parts, req_body) = response.into_parts();
    let body_bytes = axum::body::to_bytes(req_body, usize::MAX).await.unwrap();

    let json: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("Response body is not valid JSON");
    world.response_json = Some(json);

    let req = Response::from_parts(req_parts, Body::from(body_bytes));
    world.response = Some(req);
}

#[then(expr = "the response body should contain the OpenAPI version {string}")]
async fn the_response_body_should_contain_the_openapi_version(
    world: &mut ServiceWorld,
    expected_version: String,
) {
    let json = world
        .response_json
        .as_ref()
        .expect("No JSON response stored");
    let openapi_version = json
        .get("openapi")
        .expect("No 'openapi' field in JSON response")
        .as_str()
        .expect("'openapi' field is not a string");

    assert_eq!(openapi_version, expected_version);
}

#[then("no feed key exists in the system")]
fn then_no_feed_key_exists_in_the_system(world: &mut ServiceWorld) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    assert!(!path.exists(), "Feed key file should not exist");
}

#[then("a feed key exists in the system")]
fn then_a_feed_key_exists_in_the_system(world: &mut ServiceWorld) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    assert!(path.exists(), "Feed key file should exist");
}

#[then(regex = r"^the feed key in the system should be '(.+)'$")]
async fn the_feed_key_in_the_system_should_be(world: &mut ServiceWorld, expected_key: String) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    let key = tokio::fs::read_to_string(path)
        .await
        .expect("Could not read feed key file");
    assert_eq!(key, expected_key);
}

#[then(expr = "the JSON message should be {string}")]
fn the_json_message_should_be(world: &mut ServiceWorld, expected_message: String) {
    let json = world
        .response_json
        .as_ref()
        .expect("No JSON response stored");
    let message = json
        .get("message")
        .expect("No 'message' field in JSON response")
        .as_str()
        .expect("'message' field is not a string");

    assert_eq!(message, expected_message);
}

#[tokio::main]
async fn main() {
    ServiceWorld::cucumber()
        .fail_on_skipped()
        // .init_tracing()
        .run("tests/features")
        .await;
}
