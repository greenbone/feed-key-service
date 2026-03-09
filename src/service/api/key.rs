// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{io, path::Path};

use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Request, State},
    http::header,
    response::IntoResponse,
    routing::{get, put},
};
use futures::TryStreamExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio_util::io::{ReaderStream, StreamReader};
use utoipa::{OpenApi, ToSchema};

use crate::{
    service::{
        app::{AppRouter, AppState},
        error::Error,
        middleware::authorization_middleware,
        response::JsonResponse,
    },
    validation::{Base64FeedKeyValidator, FeedKeyValidator},
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

/// Download the current feed key file.
#[utoipa::path(
  get,
  path = "",
  responses(
    (
        status = 200,
        description = "Key downloaded successfully",
        body = String,
        content_type = "application/octet-stream",
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
        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
        (header::CONTENT_DISPOSITION, content_disposition),
        (header::CONTENT_LENGTH, "".to_string()),
        (header::TRANSFER_ENCODING, "chunked".to_string()),
    ];
    Ok((headers, body))
}

async fn validate_and_read_feed_key<R>(reader: &mut BufReader<R>) -> Result<Vec<String>, Error>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut validator = Base64FeedKeyValidator::new();
    let mut lines: Vec<String> = Vec::new();
    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line).await.map_err(|e| {
            tracing::error!("Failed to read uploaded file: {}", e);
            Error::InternalServerError("Key upload failed. Could not read file.".to_string())
        })?;
        if len == 0 {
            validator.done().map_err(|e| {
                tracing::error!("Key validation failed: {}", e);
                Error::BadRequest(format!("Key upload failed. Failed to validate key. {}", e))
            })?;
            break;
        }
        validator.push(&line).map_err(|e| {
            tracing::error!("Key validation failed on '{}': {}", line.trim(), e);
            Error::BadRequest(format!("Key upload failed. Failed to validate key. {}", e))
        })?;
        lines.push(line);
    }

    Ok(lines)
}

async fn write_feed_key(feed_key_path: &Path, lines: Vec<String>) -> Result<(), Error> {
    let tempfile = tempfile::NamedTempFile::new().map_err(|e| {
        tracing::error!("Failed to create temporary file for key upload: {}", e);
        Error::InternalServerError("Key upload failed. File error.".to_string())
    })?;
    let tempfile_path = tempfile.into_temp_path();
    let file = match tokio::fs::File::create(&tempfile_path).await {
        Ok(file) => file,
        Err(err) => {
            tracing::error!(
                "Failed to create key file {}: {}",
                feed_key_path.display(),
                err
            );
            return Err(Error::InternalServerError(
                "Key upload failed. File error.".to_string(),
            ));
        }
    };
    let mut writer = BufWriter::new(file);
    for line in lines {
        writer.write_all(line.as_bytes()).await.map_err(|e| {
            tracing::error!("Failed to write key data: {}", e);
            Error::InternalServerError("Key upload failed. Could not write to file.".to_string())
        })?;
    }

    writer.flush().await.map_err(|e| {
        tracing::error!("Failed to flush key file writer: {}", e);
        Error::InternalServerError("Key upload failed. Could not write to file.".to_string())
    })?;
    tokio::fs::copy(&tempfile_path, &feed_key_path)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to move temporary key file to destination {}: {}",
                feed_key_path.display(),
                e
            );
            Error::InternalServerError("Key upload failed. Could not write to file.".to_string())
        })?;

    let _ = tempfile_path.close();
    tracing::info!("Successfully wrote key file {}", feed_key_path.display());
    Ok(())
}

/// Upload a new feed key file.
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
        status = 400,
        description = "Bad Request. Failed to validate key.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Bad request: Key upload failed. Failed to validate key. Invalid Key data"})
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
        description = "Key upload failed. Could not read file.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Key upload failed. Could not read file."})
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
  request_body(content(("application/octet-stream")), description = "The key file to upload"),
  security(("jwt_auth" = []))
)]
async fn upload_key(
    State(state): State<AppState>,
    request: Request,
) -> Result<JsonResponse, Error> {
    let stream = request
        .into_body()
        .into_data_stream()
        .map_err(io::Error::other);
    let stream_reader = StreamReader::new(stream);
    let mut reader = BufReader::new(stream_reader);

    let lines = validate_and_read_feed_key(&mut reader).await?;
    write_feed_key(&state.feed_key_path, lines).await?;

    Ok(JsonResponse::from_success("Key uploaded successfully"))
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
        description = "Bad Request. Failed to validate key.",
        body = JsonResponse,
        content_type= "application/json",
        example = json!({"status": "error", "message": "Bad request: Key upload failed. Failed to validate key. Invalid Key data"})
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
    let field = match field {
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
    let stream = StreamReader::new(field.map_err(io::Error::other));
    let mut reader = BufReader::new(stream);

    let lines = validate_and_read_feed_key(&mut reader).await?;
    write_feed_key(&state.feed_key_path, lines).await?;

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
