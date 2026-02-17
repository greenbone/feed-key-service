// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Request, State},
    http::header,
    response::IntoResponse,
    routing::{get, put},
};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_stream::StreamExt;
use tokio_util::io::{ReaderStream, StreamReader};
use utoipa::{OpenApi, ToSchema};

use crate::service::{
    app::{AppRouter, AppState},
    error::Error,
    middleware::authorization_middleware,
    response::JsonResponse,
};

const KEY_TAG: &str = "Key";
const DEFAULT_UPLOAD_LIMIT: usize = 2 * 1024 * 1024; // 2 MB

#[derive(OpenApi)]
#[openapi(
    info(description = "Key management endpoint", title = "Key API"),
    tags((name = KEY_TAG, description = "Key management operations")),
    paths(download_key, upload_key, upload_key_multipart, delete_key)
)]
pub struct KeyApi;

/// Download the current feed key as a PEM file.
#[utoipa::path(
  get,
  path = "",
  responses(
    (
        status = 200,
        description = "Key downloaded successfully",
        body = String,
        content_type = "application/x-pem-file"
    ),
    (
        status = 401,
        description = "Unauthorized",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Unauthorized"})
    ),
    (
        status = 404,
        description = "Key not available",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key not available"})
    ),
  ),
  tag = KEY_TAG,
  security(("jwt_auth" = []))
)]
async fn download_key(State(state): State<AppState>) -> Result<impl IntoResponse, Error> {
    let file = match tokio::fs::File::open(&state.feed_key_path).await {
        Ok(file) => file,
        Err(_) => {
            return Err(Error::KeyNotFound);
        }
    };
    let filename = match state.feed_key_path.file_name() {
        Some(name) => name,
        None => {
            tracing::error!(
                "Could not determine file name for key file {}",
                state.feed_key_path.display()
            );
            // should not happen, but handle it gracefully
            return Err(Error::InternalServerError(
                "Could not determine file name for key file".to_string(),
            ));
        }
    };
    let content_disposition = format!("attachment; filename=\"{}\"", filename.to_string_lossy());
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let headers = [
        (header::CONTENT_TYPE, "application/x-pem-file".to_string()),
        (header::CONTENT_DISPOSITION, content_disposition),
        (header::CONTENT_LENGTH, "".to_string()),
        (header::TRANSFER_ENCODING, "chunked".to_string()),
    ];
    Ok((headers, body))
}

/// Upload a new feed key as a PEM file.
#[utoipa::path(
  put,
  path = "",
  responses(
    (
        status = 200,
        description = "Key uploaded successfully",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "success", "message": "Key uploaded successfully"})
    ),
    (
        status = 401,
        description = "Unauthorized",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Unauthorized"})
    ),
    (
        status = 500,
        description = "Key upload failed. File error.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. File error."})
    ),
    (
        status = 500,
        description = "Key upload failed. Stream error.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. Stream error."})
    ),
  ),
  tag = KEY_TAG,
  request_body(content(("application/x-pem-file"),("application/octet-stream")), description = "The key file to upload"),
  security(("jwt_auth" = []))
)]
async fn upload_key(
    State(state): State<AppState>,
    request: Request,
) -> Result<JsonResponse, Error> {
    let file = match tokio::fs::File::create(&state.feed_key_path).await {
        Ok(file) => file,
        Err(err) => {
            tracing::error!(
                "Failed to create key file {}: {}",
                state.feed_key_path.display(),
                err
            );
            return Err(Error::InternalServerError(
                "Key upload failed. File error.".to_string(),
            ));
        }
    };
    let stream = request.into_body().into_data_stream();
    let stream = stream.map(|result| result.map_err(std::io::Error::other));
    let mut reader = StreamReader::new(stream);
    let mut writer = BufWriter::new(file);
    match tokio::io::copy(&mut reader, &mut writer).await {
        Ok(_) => {
            tracing::info!(
                "Successfully wrote key file {}",
                state.feed_key_path.display()
            );
            Ok(JsonResponse::from_success("Key uploaded successfully"))
        }
        Err(err) => {
            tracing::error!(
                "Failed to write key file {}: {}",
                state.feed_key_path.display(),
                err
            );
            Err(Error::InternalServerError(
                "Key upload failed. Stream error.".to_string(),
            ))
        }
    }
}

// only for utoipa documentation
#[derive(ToSchema)]
#[allow(unused)]
struct UploadedForm {
    #[schema(content_media_type = "application/octet-stream", format = "binary")]
    file: String,
}

/// Upload a new feed key as a multipart/form-data file upload.
#[utoipa::path(
  post,
  path = "",
  request_body(content_type = "multipart/form-data", content = inline(UploadedForm), description = "File to upload"),
  responses(
    (
        status = 200,
        description = "Key upload successful",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "success", "message": "Key uploaded successfully"})
    ),
    (
        status = 400,
        description = "Key upload failed. Invalid multipart data.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. Invalid multipart data."})
    ),
    (
        status = 400,
        description = "Key upload failed. No file provided.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. No file provided."})
    ),
    (
        status = 400,
        description = "Key upload failed. Could not read file field.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. Could not read file field."})
    ),
    (
        status = 401,
        description = "Unauthorized",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Unauthorized"})
    ),
    (
        status = 500,
        description = "Key upload failed. File error.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. File error."})
    ),
    (
        status = 500,
        description = "Key upload failed. Stream error.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. Stream error."})
    ),
    (
        status = 500,
        description = "Key upload failed. Could not write to file.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. Could not write to file."})
    ),
  ),
  tag = KEY_TAG,
  security(("jwt_auth" = []))
)]
async fn upload_key_multipart(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<JsonResponse, Error> {
    let field = multipart.next_field().await;
    let field = match field {
        Ok(f) => f,
        Err(err) => {
            tracing::error!("Failed to process multipart upload: {}", err);
            return Err(Error::BadRequest(
                "Key upload failed. Invalid multipart data.".to_string(),
            ));
        }
    };
    let mut field = match field {
        Some(f) => f,
        None => {
            tracing::error!("No file provided in multipart upload");
            return Err(Error::BadRequest(
                "Key upload failed. No file provided.".to_string(),
            ));
        }
    };
    match field.name() {
        Some("file") => {}
        _ => {
            tracing::error!("No file provided in multipart upload");
            return Err(Error::BadRequest(
                "Key upload failed. No file provided.".to_string(),
            ));
        }
    };
    let file = match tokio::fs::File::create(&state.feed_key_path).await {
        Ok(file) => file,
        Err(err) => {
            tracing::error!(
                "Failed to create key file {}: {}",
                state.feed_key_path.display(),
                err
            );
            return Err(Error::InternalServerError(
                "Key upload failed. File error.".to_string(),
            ));
        }
    };

    let mut writer = BufWriter::new(file);
    while let Some(chunk) = field.chunk().await.map_err(|_| {
        Error::BadRequest("Key upload failed. Could not read file field".to_string())
    })? {
        if let Err(e) = writer.write(&chunk).await {
            tracing::error!(
                "Failed to write key file {}: {}",
                state.feed_key_path.display(),
                e
            );
            return Err(Error::InternalServerError(
                "Key upload failed. Stream error.".to_string(),
            ));
        }
    }
    writer.flush().await.map_err(|err| {
        tracing::error!(
            "Failed to flush key file {}: {}",
            state.feed_key_path.display(),
            err
        );
        Error::InternalServerError("Key upload failed. Could not write to file.".to_string())
    })?;
    tracing::info!(
        "Successfully wrote key file {}",
        state.feed_key_path.display()
    );
    Ok(JsonResponse::from_success("Key uploaded successfully"))
}

/// Delete the current feed key.
#[utoipa::path(
  delete,
  path = "",
  responses(
    (
        status = 200,
        description = "Key deleted successfully",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "success", "message": "Key deleted successfully"})
    ),
    (
        status = 401,
        description = "Unauthorized",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Unauthorized"})
    ),
    (
        status = 500,
        description = "Key deletion failed",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key deletion failed"})
    ),
  ),
  tag = KEY_TAG,
  security(("jwt_auth" = []))
)]
async fn delete_key(State(state): State<AppState>) -> Result<JsonResponse, Error> {
    match state.feed_key_path.try_exists() {
        Ok(exists) => {
            if !exists {
                tracing::info!(
                    "Key file {} does not exist, nothing to delete",
                    state.feed_key_path.display()
                );
                return Ok(JsonResponse::from_success("Key deleted successfully"));
            }
        }
        Err(_) => {
            tracing::error!(
                "Failed to check existence of key file {}",
                state.feed_key_path.display()
            );
            return Err(Error::KeyDeletionFailed);
        }
    }

    match tokio::fs::remove_file(&state.feed_key_path).await {
        Ok(_) => {
            tracing::info!(
                "Successfully deleted key file {}",
                state.feed_key_path.display()
            );
            Ok(JsonResponse::from_success("Key deleted successfully"))
        }
        Err(err) => {
            tracing::error!(
                "Failed to delete key file {}: {}",
                state.feed_key_path.display(),
                err
            );
            Err(Error::KeyDeletionFailed)
        }
    }
}

pub fn routes(state: AppState, upload_limit: Option<usize>) -> AppRouter {
    Router::new()
        .route("/", get(download_key).delete(delete_key))
        .route(
            "/",
            put(upload_key)
                .post(upload_key_multipart)
                // by default the upload limit is 2MB
                // see https://docs.rs/axum/latest/axum/extract/struct.Multipart.html#large-files
                // and https://docs.rs/axum/latest/axum/extract/struct.DefaultBodyLimit.html
                .layer(DefaultBodyLimit::max(
                    upload_limit.unwrap_or(DEFAULT_UPLOAD_LIMIT),
                )),
        )
        // require authorization for all key routes
        .layer(axum::middleware::from_fn_with_state(
            state,
            authorization_middleware,
        ))
}
