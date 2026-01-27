// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{Router, http::StatusCode, response::IntoResponse};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use rustls::{ServerConfig, server::WebPkiClientVerifier};
use std::{
    error::Error,
    net::SocketAddr,
    path::{self, Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use thiserror::Error;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, decompression::RequestDecompressionLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    certs::{create_client_root_cert_store, load_certificate, load_private_key},
    jwt::JwtSecret,
    service::{api, openapi},
};

use tokio::signal::unix::{SignalKind, signal};

fn shutdown(handle: &Handle<SocketAddr>) {
    handle.graceful_shutdown(Some(Duration::from_secs(5)));
}

async fn shutdown_signal(handle: Handle<SocketAddr>) {
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to create SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to create SIGINT handler");

    tokio::select! {
        _ = sigint.recv() => {
            tracing::info!("SIGINT received, shutting down");
            shutdown(&handle);
        },
        _ = sigterm.recv() => {
            tracing::info!("SIGTERM received, shutting down");
            shutdown(&handle);
        },
    }
}

#[derive(Clone)]
pub struct GlobalState {
    pub feed_key_path: PathBuf,
    pub jwt_secret: JwtSecret,
}

impl GlobalState {
    pub fn new(feed_key_path: &Path, jwt_secret: &JwtSecret) -> Self {
        Self {
            feed_key_path: path::absolute(feed_key_path).unwrap(),
            jwt_secret: jwt_secret.clone(),
        }
    }
}

pub struct App {
    state: GlobalState,
    upload_limit: Option<usize>,
    enable_api_doc: bool,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Invalid IP Address: {0}")]
    InvalidAddress(String),

    #[error(
        "Client certificate authentication enabled but no CA certificate chain provided in {0}"
    )]
    EmptyClientCertificateChain(PathBuf),
}

impl App {
    pub fn new(
        feed_key_path: &Path,
        upload_limit: Option<usize>,
        jwt_secret: &JwtSecret,
        enable_api_doc: bool,
    ) -> Self {
        let state = GlobalState::new(feed_key_path, jwt_secret);
        Self {
            state,
            upload_limit,
            enable_api_doc,
        }
    }

    pub fn init_tracing(&self, log: &str) {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(log))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    pub fn router(self) -> Router {
        let state = Arc::new(self.state);
        let mut router =
            Router::new().nest("/api/v1", api::routes(state.clone(), self.upload_limit));

        if self.enable_api_doc {
            tracing::info!("OpenAPI documentation enabled");
            router = router.merge(openapi::routes());
        }

        router = router
            .layer(
                ServiceBuilder::new()
                    .layer(RequestDecompressionLayer::new())
                    .layer(CompressionLayer::new()),
            )
            .layer(TraceLayer::new_for_http())
            .fallback(handler_404);

        router.with_state(state.clone())
    }

    async fn serve_http(
        self,
        handle: Handle<SocketAddr>,
        address: SocketAddr,
    ) -> Result<(), std::io::Error> {
        tracing::info!("Listening on http://{}", address);
        axum_server::bind(address)
            .handle(handle)
            .serve(self.router().into_make_service())
            .await
    }

    async fn serve_tls(
        self,
        handle: Handle<SocketAddr>,
        address: SocketAddr,
        tls_server_cert: &Path,
        tls_server_key: &Path,
    ) -> Result<(), std::io::Error> {
        let config = RustlsConfig::from_pem_file(tls_server_cert, tls_server_key).await?;
        tracing::info!("Listening on https://{}", address);
        axum_server::bind_rustls(address, config)
            .handle(handle)
            .serve(self.router().into_make_service())
            .await
    }

    async fn server_mtls(
        self,
        handle: Handle<SocketAddr>,
        address: SocketAddr,
        tls_server_cert: &Path,
        tls_server_key: &Path,
        tls_client_ca_cert: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let server_cert = load_certificate(tls_server_cert)?;
        let server_key = load_private_key(tls_server_key)?;
        let root_store = create_client_root_cert_store(tls_client_ca_cert)?;
        if root_store.is_empty() {
            return Err(Box::new(AppError::EmptyClientCertificateChain(
                tls_client_ca_cert.into(),
            )));
        }
        let client_cert_verifier = WebPkiClientVerifier::builder(Arc::new(root_store)).build()?;
        let mut server_config = ServerConfig::builder()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(vec![server_cert], server_key)?;

        // Enable ALPN protocols to support both HTTP/2 and HTTP/1.1
        server_config.alpn_protocols = vec![
            b"h2".to_vec(),       // HTTP/2
            b"http/1.1".to_vec(), // HTTP/1.1
        ];
        let config = RustlsConfig::from_config(Arc::new(server_config));

        tracing::info!("Client certificate authentication enabled");
        tracing::info!("Listening on https://{}", address);

        axum_server::bind_rustls(address, config)
            .handle(handle)
            .serve(self.router().into_make_service())
            .await
            .map_err(|e| e.into())
    }

    pub async fn serve(
        self,
        server: &str,
        port: u16,
        tls_server_cert: Option<&Path>,
        tls_server_key: Option<&Path>,
        tls_client_certs: Option<&Path>,
    ) -> Result<(), Box<dyn Error>> {
        let address = format!("{}:{}", server, port);
        tracing::debug!(server = ?server, port = ?port, "parsing server address {}", address);
        let socket_address =
            SocketAddr::from_str(&address).map_err(|_| AppError::InvalidAddress(address))?;
        let handle = Handle::new();
        tokio::spawn(shutdown_signal(handle.clone()));

        if tls_server_cert.is_some() && tls_server_key.is_some() {
            let tls_server_cert = tls_server_cert.unwrap();
            let tls_server_key = tls_server_key.unwrap();
            match tls_client_certs {
                Some(client_cert) => self
                    .server_mtls(
                        handle,
                        socket_address,
                        &tls_server_cert,
                        &tls_server_key,
                        &client_cert,
                    )
                    .await
                    .map_err(|e| e.into()),
                None => self
                    .serve_tls(handle, socket_address, &tls_server_cert, &tls_server_key)
                    .await
                    .map_err(|e| e.into()),
            }
        } else {
            self.serve_http(handle, socket_address)
                .await
                .map_err(|e| e.into())
        }
    }
}

pub type AppState = Arc<GlobalState>;
pub type AppRouter = Router<AppState>;

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here")
}
