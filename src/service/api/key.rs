// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Request, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, put},
};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_stream::StreamExt;
use tokio_util::io::{ReaderStream, StreamReader};
use utoipa::{OpenApi, ToSchema};

use crate::{
    service::app::{AppRouter, AppState},
    service::middleware::authorization_middleware,
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
    (status = 200, description = "Key download OK", body = String, content_type = "application/x-pem-file"),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "Key not available", body = String),
  ),
  tag = KEY_TAG,
  security(("jwt_auth" = []))
)]
async fn download_key(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let file = match tokio::fs::File::open(&state.feed_key_path).await {
        Ok(file) => file,
        Err(_) => {
            return Err((StatusCode::NOT_FOUND, "Key not available"));
        }
    };
    let filename = match state.feed_key_path.file_name() {
        Some(name) => name,
        None => {
            return Err((StatusCode::BAD_REQUEST, "File name couldn't be determined"));
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
    (status = 200, description = "Key upload successful", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Key upload failed. File error.", body = String),
    (status = 500, description = "Key upload failed. Stream error.", body = String),
  ),
  tag = KEY_TAG,
  request_body(content(("application/x-pem-file"),("application/octet-stream")), description = "The key file to upload"),
  security(("jwt_auth" = []))
)]
async fn upload_key(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    let file = match tokio::fs::File::create(&state.feed_key_path).await {
        Ok(file) => file,
        Err(err) => {
            tracing::error!(
                "Failed to create key file {}: {}",
                state.feed_key_path.display(),
                err
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Key upload failed. File error.",
            ));
        }
    };
    let stream = request.into_body().into_data_stream();
    let stream = stream
        .map(|result| result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)));
    let mut reader = StreamReader::new(stream);
    let mut writer = BufWriter::new(file);
    match tokio::io::copy(&mut reader, &mut writer).await {
        Ok(_) => {
            tracing::info!(
                "Successfully wrote key file {}",
                state.feed_key_path.display()
            );
            return Ok("Key upload successful");
        }
        Err(err) => {
            tracing::error!(
                "Failed to write key file {}: {}",
                state.feed_key_path.display(),
                err
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Key upload failed. Stream error.",
            ));
        }
    }
}

// only for utoipa documentation
#[derive(ToSchema)]
#[allow(unused)]
struct UploadedForm {
    #[schema(content_media_type = "application/x-pem-file", format = "binary")]
    file: String,
}

/// Upload a new feed key as a multipart/form-data file upload.
#[utoipa::path(
  post,
  path = "",
  request_body(content_type = "multipart/formdata", content = inline(UploadedForm), description = "File to upload"),
  responses(
    (status = 200, description = "Key upload successful", body = String),
    (status = 400, description = "Key upload failed. Invalid multipart data.", body = String),
    (status = 400, description = "Key upload failed. No file provided.", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Key upload failed. File error.", body = String),
    (status = 500, description = "Key upload failed. Stream error.", body = String),
    (status = 500, description = "Key upload failed. Could not write to file.", body = String),
  ),
  tag = KEY_TAG,
  security(("jwt_auth" = []))
)]
async fn upload_key_multipart(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<&'static str, (StatusCode, &'static str)> {
    let field = multipart.next_field().await;
    let field = match field {
        Ok(f) => f,
        Err(err) => {
            tracing::error!("Failed to process multipart upload: {}", err);
            return Err((
                StatusCode::BAD_REQUEST,
                "Key upload failed. Invalid multipart data.",
            ));
        }
    };
    let mut field = match field {
        Some(f) => f,
        None => {
            tracing::error!("No file provided in multipart upload");
            return Err((
                StatusCode::BAD_REQUEST,
                "Key upload failed. No file provided.",
            ));
        }
    };
    match field.name() {
        Some(name) if name == "file" => {}
        _ => {
            tracing::error!("No file provided in multipart upload");
            return Err((
                StatusCode::BAD_REQUEST,
                "Key upload failed. No file provided.",
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
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Key upload failed. File error.",
            ));
        }
    };

    let mut writer = BufWriter::new(file);
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "foo"))?
    {
        if let Err(e) = writer.write(&chunk).await {
            tracing::error!(
                "Failed to write key file {}: {}",
                state.feed_key_path.display(),
                e
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Key upload failed. Stream error.",
            ));
        }
    }
    writer.flush().await.map_err(|err| {
        tracing::error!(
            "Failed to flush key file {}: {}",
            state.feed_key_path.display(),
            err
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Key upload failed. Could not write to file.",
        )
    })?;
    tracing::info!(
        "Successfully wrote key file {}",
        state.feed_key_path.display()
    );
    Ok("Key upload successful")
}

/// Delete the current feed key.
#[utoipa::path(
  delete,
  path = "",
  responses(
    (status = 200, description = "Key deleted successfully", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Key deletion failed", body = String),
  ),
  tag = KEY_TAG,
  security(("jwt_auth" = []))
)]
async fn delete_key(State(state): State<AppState>) -> impl IntoResponse {
    match state.feed_key_path.try_exists() {
        Ok(exists) => {
            if !exists {
                tracing::info!(
                    "Key file {} does not exist, nothing to delete",
                    state.feed_key_path.display()
                );
                return Ok("Key deleted successfully");
            }
        }
        Err(_) => {
            tracing::error!(
                "Failed to check existence of key file {}",
                state.feed_key_path.display()
            );
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Key deletion failed"));
        }
    }

    match tokio::fs::remove_file(&state.feed_key_path).await {
        Ok(_) => {
            tracing::info!(
                "Successfully deleted key file {}",
                state.feed_key_path.display()
            );
            return Ok("Key deleted successfully");
        }
        Err(err) => {
            tracing::error!(
                "Failed to delete key file {}: {}",
                state.feed_key_path.display(),
                err
            );
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Key deletion failed"));
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
