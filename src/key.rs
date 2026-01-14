// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use tokio::io::BufWriter;
use tokio_stream::StreamExt;
use tokio_util::io::{ReaderStream, StreamReader};
use utoipa::OpenApi;

use crate::app::{AppRouter, AppState};

const KEY_TAG: &str = "Key";

#[derive(OpenApi)]
#[openapi(
    info(description = "Key management endpoint", title = "Key API"),
    tags((name = KEY_TAG, description = "Key management operations")),
    paths(download_key, upload_key, delete_key)
)]
pub struct KeyApi;

#[utoipa::path(
  get,
  path = "",
  responses(
    (status = 200, description = "Key download OK", body = String, content_type = "application/x-pem-file"),
    (status = 404, description = "Key not available", body = String),
  ),
  tag = KEY_TAG,
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

#[utoipa::path(
  post,
  path = "",
  responses(
    (status = 200, description = "Key upload OK", body = String),
    (status = 500, description = "Key upload failed. File error.", body = String),
    (status = 500, description = "Key upload failed. Stream error.", body = String),
  ),
  tag = KEY_TAG,
  request_body(content_type = "application/x-pem-file", description = "The key file to upload")
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

#[utoipa::path(
  delete,
  path = "",
  responses(
    (status = 200, description = "Key deleted successfully", body = String),
    (status = 500, description = "Key deletion failed", body = String),
  ),
  tag = KEY_TAG,
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

pub fn routes() -> AppRouter {
    Router::new().route("/", get(download_key).post(upload_key).delete(delete_key))
}
