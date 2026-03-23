// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{os::unix::fs::PermissionsExt, path::PathBuf};

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
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

const HEALTH_API: &str = "/api/v1/health";
const KEY_API: &str = "/api/v1/key";
const KEY_STATUS_API: &str = "/api/v1/key/status";
const OPENAPI_DOCS_API: &str = "/api/v1/openapi.json";
const SWAGGER_UI_URL: &str = "/swagger-ui/";
const SHARED_SECRET: &str = "some-secret";
const VALID_FEED_KEY: &str =
    "ZG93bmxvYWRAc3RhZ2luZy5vcGVyYXRpb24ub3MuZ3JlZW5ib25lLm5ldDovZmVlZC8KLS0tLS1C
RUdJTiBSU0EgUFJJVkFURSBLRVktLS0tLQpNSUlFb1FJQkFBS0NBUUVBdSsvSXVMcjNaOXlyK1o5
d3dXcUtZVDN4RlZxTnFtUVo1QzVhVG9iZ3czNFdITThjClMzQXJGQm9McU5LMFlXUlNyS05XOGNY
R1BiVEV2Y0NKaFBWbWFVc0QzMHZBRVlzWExONklJYm5oT2dpdGVCbm0KekREVDVRZmZSTUdMUzYx
WU5RSnpoMjdEUWRrSjRBVkI3enBBVnRVdTNwaWkzZHQyZzBPR1pmd214L2hPS3kwUAoxRzhDOGFI
WXYvbUJpZ0NWdm9ScUhjWXFKZHJ6R3A4Ym8wVHkrK09XMjRtaTZvUUcxb2ZvdEh1dk02U1JUajky
Ckc2ZlFlZGF0OUNCQnhwbEdqc3JKUUZwOU9wUGVWRWVDNnNveGdUc1ZsZUtldXpRaUU3Y05iTG1j
TUEyYVBLRk4KTnpUUmpHbUc4dlp4TStpempBSFdNeG95b01JRVBkQjkzNGdOOHdJQkl3S0NBUUFR
Rzk0Qk5KamRCRWxCU0M0OApkdGlwUDlMVjhkR2c1QUk0SVRyd3lidCtlSVdOY05hS0g0NnA4NXFa
Y1NWbmJ2L0ZwNW04ODdIb0NDNGU1SjRTCnRmTFYwenJZb0JmRy9Vc2hHbU53bXVkclg5UmF3R1E5
WTBWY3hpa1VoWjQ1cjhXN1pwd3dMaEM4Z0ZGTnQwN0wKWE1PdnE5OXlLbGNhVktQQ0c3c1FEa3gz
aWlvRkUxYnhuanVFN1J1REdwWk1hVUNsWXl4V1ZtTjJYc281L1J3Kwo0c1lXUlhxZzZDSk12MHB1
ZzVDekVEWFpCalhBNmNWWnBydE1iLzFNQmhjTDhrOEZ5TUpyQ0dSV0w1d1BDcjVOCmtjMzFoTS8v
MDJnTk5SQ21pMHFhby9lbGhIMlBsU000S1hlY1NuRk5ER2JjL2IvNUNLREgwVkxldzRBMXZKb0YK
WDljakFvR0JBUFI0dmlXZ0FEZzFUdnp6WEsvUWgyWUZCQ2dyNWQvamlUTWV6Y0lVT0tpRENQeFZk
L0JwRFcyQgo1czFCMkpVVkNIbGtKVm03VUcyemdSOEhCcWYrVHNRZWE1QUN6WU9iZ1dpK0VMMWIw
UFdkQUxsVldtZXNrbllVCnFpZ3JlbnVEejVHOFFCL1IzdjZPMi8xSGg3ZzBWUTFPQkkyVncwemFQ
QW9xbEtESWdjcWxBb0dCQU1UTWpVa3gKamhSWUwwVjl1L2crQXBMYXBlZFg4R3lILzk0Qjh4QUd0
cGpLV3E2R0tDSlhhWXBMVE5pZTVmczZxUG9mY3NpZgp2aTcxT3dQTGZUZVNQNU1zNDdKYzFieGNQ
RmptdVc2d3FZT3oxNHhDa0hrZk5wUm9DU21LY05kMmlhVDFQK2ZXCkhMVGVLUXA5M2dqdFdKRTU3
ZlRYZDNSS1I4ejFvNExZVXNxM0FvR0FXczNDK0FnNm1JalVNaEUvcndSUGpFc0IKaTBPQlJJZTJu
ZlYvbjl1Z0JCTnB2TTlKemxMbnV2VzhJRnBKSVc0MldQbFBzNTFZWTBLc1RWcGhqdHJNeklCVAox
bWR3NTcxcWxKNWQvVEM3VStuYnNvMXF1TXNSMUJaTjFHZnJxamhGeUdwK043UmFKQWt0STQrWXoy
UGxFNUlCCnNPNTd2WFdoUlpySmZZeE5keWNDZ1lFQWh2S3FCazNkeE03QlZEajE3QTFEbCtaaklr
cnVBVWRRWGJneHBKYmoKbS9oNHNqQWJoVU5CRGwrRkp0TmIzM2l1WWw2MUc5dEFsVHB4bkRQRGs5
SC90V0NjSW9qTkUzSnloaHFOeUQ2ZwpIOHZIQVJlemhrRkovMTFJSEh3d0hyZXUxNE9aaVFmWks1
RUdNeFI2L3M2akljRlN0b1VldGlST2ZlcEVQSGNVCk16c0NnWUJ3L1NXRmVGZzZGeGtEbU1uUzNx
bEJ5MlMyRG1NQ24xSkJvTHRCc1VvcmR2S3lnbHhPTEpGOWlvSDUKOHpGYTdKUFF4NlpVbm83cTJl
dkMxcU9IMm8wRUhFQklHb1dGNTdlM2pFNDJseHlscEVwNlhrQ2Q4Y1BOTDV6MQpreHJ6ZlltUjdS
bm45M2Myd2lrZ0VuUmpNK09IbkVpRHVmdTZJQW16eDE4ODRXcktDZz09Ci0tLS0tRU5EIFJTQSBQ
UklWQVRFIEtFWS0tLS0tCi0tLS0tQkVHSU4gUEdQIFBVQkxJQyBLRVkgQkxPQ0stLS0tLQpWZXJz
aW9uOiBHbnVQRyB2Mi4wLjEzIChHTlUvTGludXgpCgptUUdpQkV2N29Zc1JCQUMrcjE2WGFHWXBJ
Qm1JR09GSDEyMlhOUVk5UVU1clBmQW9EekFydG5JTGhpbVJ1RlFwCkhCc2FyczhHOTdROU1QMU9C
cmx5WG9DTk4rWDJYbEdtU3loUjNnd3RTbVZUaEtudFROWks4K1pIUFl1Z3dHcVMKTjNROU1nTGdS
MnEvWW4vODdrVW5QeWh4bDdtRnhRbmUvMFZKcDFLMzJ4Y0Fld0pyYlduZ0h1QVh6d0NncXhtaQpp
ZncxdlhRQzZXS3k4NEtKUWRrOStzY0QvMmlxa0EwZE8xY2VCUDFtcXdXYVVMMUg1VndZbEhUaytY
NUozQmFPCnREdnR3QUpvMG1GVzQyZnJDYXRWT1FibTdid3NMeEZpZzZ5d1d6NGk4dDR0clh2Rm51
QS8veGhoZGsrRjVEWlIKQko3VVhmcDRWaGhFUGh1dmR0SEN6OC9INlRKTWU0ZHZocGRWblVyVjdm
R1NGdVJuaE5BeFl1U1AvWHJzSVRNMgo2QmFrQS8wV216c1JrUWUyZ0x1blFNdXJCT3NYNkVlUDln
eXoyUHIxZmRTcUxJL3VlRC9hbC9ocTdYd1pwcFNUCkN4Vi9SdUhSdzR5WFczR2RqTnBRWENlczYx
M0UvdG5OamxYMEV4d3FycFRlNFEyelh6d1lLU0RxKzljQ2oxRisKY1ZCVzRSUmxJY0ltc3kzYm9C
c3hlWjBMMEladCt4c252VTh2SGxSTllhQnRQTFFSaXJRc1IzSmxaVzVpYjI1bApJRk5sWTNWeWFY
UjVJRVpsWldRZ1BHWmxaV1JBWjNKbFpXNWliMjVsTG01bGRENklaZ1FURVFJQUpnVUNTL3VoCml3
SWJBd1VKQjRUT0FBWUxDUWdIQXdJRUZRSUlBd1FXQWdNQkFoNEJBaGVBQUFvSkVFQmhPekF3aVB2
R1lHb0EKb0k4cWczcG5CVlh2UGpHVU5lb0tGbWlzV29OQ0FKOS9pMDBQbldGSTVYcUM2a3p5aGpl
dWtNdWZ1Zz09Cj1tVm9iCi0tLS0tRU5EIFBHUCBQVUJMSUMgS0VZIEJMT0NLLS0tLS0K";
const INVALID_FEED_KEY: &str = "SOME INVALID FEED KEY";

#[derive(Debug, Default, World)]
struct ServiceWorld {
    app: Option<App>,
    authenticated: bool,
    response: Option<Response>,
    response_json: Option<serde_json::Value>,
    encode_secret: Option<JwtEncodeSecret>,
    decode_secret: Option<JwtDecodeSecret>,
    tempdir: Option<TempDir>,
    tempfile_path: Option<PathBuf>,
}

#[given("the service is running")]
fn given_the_service_is_running(world: &mut ServiceWorld) {
    let encode_secret = JwtEncodeSecret::from_shared_secret(SHARED_SECRET);
    let decode_secret = JwtDecodeSecret::from_shared_secret(SHARED_SECRET);
    let tempdir = tempfile::Builder::new()
        .prefix("feed_key_")
        .tempdir()
        .expect("Could not create temporary feed key directory");
    let tempfile = tempdir.path().join("feed-key-file");
    let app = App::new(&tempfile, None, &decode_secret, false);
    world.tempdir = Some(tempdir);
    world.tempfile_path = Some(tempfile.to_path_buf());
    world.app = Some(app);
    world.encode_secret = Some(encode_secret);
    world.decode_secret = Some(decode_secret);
    world.authenticated = false;
}

#[given("a valid feed key exists in the system")]
fn given_a_valid_feed_key_exists_in_the_system(world: &mut ServiceWorld) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    std::fs::write(path, VALID_FEED_KEY)
        .unwrap_or_else(|_| panic!("Could not write feed key to temporary file {:?}", path));
}

#[given("no feed key exists in the system")]
fn given_no_feed_key_exists_in_the_system(world: &mut ServiceWorld) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    let _ = std::fs::remove_file(path);
}

#[given("the API documentation is enabled")]
fn given_the_api_documentation_feature_is_enabled(world: &mut ServiceWorld) {
    let app = world.app.as_ref().expect("service not available");
    world.app = Some(app.enable_api_documentation());
}

#[given("the user is authenticated")]
fn given_the_user_is_authenticated(world: &mut ServiceWorld) {
    world.authenticated = true;
}

#[given("the feed key file is not writable")]
fn given_the_feed_key_file_is_not_writable(world: &mut ServiceWorld) {
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

#[given("the feed key file is not readable")]
fn given_the_feed_key_file_is_not_readable(world: &mut ServiceWorld) {
    let path = world
        .tempdir
        .as_ref()
        .expect("No tempdir path available")
        .path();
    let mut permissions = std::fs::metadata(path)
        .expect("Could not get file metadata")
        .permissions();
    permissions.set_mode(0o000);
    tracing::info!("Setting tempdir permissions {:?} to not readable", path);
    std::fs::set_permissions(path, permissions).expect("Could not set file permissions");
}

#[when(
    regex = r"^I send a (GET|DELETE|POST|PUT) request to the (key endpoint|key status endpoint|health endpoint|API documentation|swagger UI)$"
)]
async fn when_i_send_a_request(world: &mut ServiceWorld, method: String, endpoint: String) {
    let builder = Request::builder();
    let builder = match endpoint.as_str() {
        "key endpoint" => builder.uri(KEY_API),
        "key status endpoint" => builder.uri(KEY_STATUS_API),
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

#[when(
    regex = r"^I upload (a valid|an invalid) feed key via a (POST|PUT) request to the key endpoint$"
)]
async fn when_i_upload_a_feed_key_via_a_request_to_the_key_endpoint(
    world: &mut ServiceWorld,
    feed_key: String,
    method: String,
) {
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
        .body(Body::from(match feed_key.as_str() {
            "a valid" => VALID_FEED_KEY,
            "an invalid" => INVALID_FEED_KEY,
            _ => panic!("Unsupported feed key type"),
        }))
        .expect("Could not build request body");
    let router = world.app.as_ref().expect("service not available").router();
    let result = router.oneshot(request).await.expect("Request failed");
    world.response = Some(result);
}

#[when(regex = r"^I post the field '(.+)' with (a valid|an invalid) feed key to the key endpoint$")]
async fn when_i_post_the_field_with_a_feed_key_to_the_key_endpoint(
    world: &mut ServiceWorld,
    field_name: String,
    feed_key: String,
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
    form.add_text(
        field_name,
        match feed_key.as_str() {
            "a valid" => VALID_FEED_KEY,
            "an invalid" => INVALID_FEED_KEY,
            _ => panic!("Unsupported feed key type"),
        },
    );
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
fn then_the_response_status_code_should_be(world: &mut ServiceWorld, status_code: u16) {
    let response = world.response.as_ref().expect("No response stored");
    assert_eq!(
        response.status(),
        StatusCode::from_u16(status_code).unwrap()
    );
}

#[then(expr = "the response body should contain the valid feed key")]
async fn then_the_response_body_should_contain_the_valid_feed_key(world: &mut ServiceWorld) {
    let response = world.response.take().expect("No response stored");
    let (req_parts, req_body) = response.into_parts();
    let body_bytes = axum::body::to_bytes(req_body, usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();

    assert_eq!(body_str, VALID_FEED_KEY);

    let req = Response::from_parts(req_parts, Body::from(body_bytes));
    world.response = Some(req);
}

#[then("the response body should be valid JSON")]
async fn then_the_response_body_should_be_valid_json(world: &mut ServiceWorld) {
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
async fn then_the_response_body_should_contain_the_openapi_version(
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

#[then("the feed key file should have the same content as the valid feed key")]
async fn then_the_feed_key_file_should_have_the_same_content_as_the_valid_feed_key(
    world: &mut ServiceWorld,
) {
    let path = world
        .tempfile_path
        .as_ref()
        .expect("No tempfile path available");
    let key = tokio::fs::read_to_string(path)
        .await
        .expect("Could not read feed key file");
    assert_eq!(key, VALID_FEED_KEY);
}

#[then(expr = "the JSON message should be {string}")]
fn then_the_json_message_should_be(world: &mut ServiceWorld, expected_message: String) {
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

#[then(expr = "the response JSON property {string} should be {string}")]
fn then_the_response_json_property_should_be(
    world: &mut ServiceWorld,
    property: String,
    expected_value: String,
) {
    let json = world
        .response_json
        .as_ref()
        .expect("No JSON response stored");

    let value = match json
        .get(&property)
        .unwrap_or_else(|| panic!("No {} field in JSON response", &property))
    {
        Value::String(value) => value,
        Value::Bool(value) => &value.to_string(),
        _ => panic!("Invalid value"),
    };

    assert_eq!(value, &expected_value);
}

#[tokio::main]
async fn main() {
    ServiceWorld::cucumber()
        .fail_on_skipped()
        // .init_tracing()
        .run("tests/features")
        .await;
}
