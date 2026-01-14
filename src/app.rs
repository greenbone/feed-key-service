// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{Router, http::StatusCode, response::IntoResponse};
use axum_server::tls_rustls::RustlsConfig;
use std::{
    net::SocketAddr,
    path::{self, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, decompression::RequestDecompressionLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct GlobalState {
    pub key_path: PathBuf,
}

impl GlobalState {
    pub fn new(key_path: PathBuf) -> Self {
        Self {
            key_path: path::absolute(key_path).unwrap(),
        }
    }
}

pub struct App {
    state: GlobalState,
}

impl App {
    pub fn new(key_path: PathBuf, log: String) -> Self {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(log))
            .with(tracing_subscriber::fmt::layer())
            .init();

        let state = GlobalState::new(key_path);
        Self { state }
    }

    pub fn router(self) -> Router {
        Router::new()
            .nest("/key", crate::key::routes())
            .nest("/health", crate::health::routes())
            .merge(crate::openapi::routes())
            .layer(
                ServiceBuilder::new()
                    .layer(RequestDecompressionLayer::new())
                    .layer(CompressionLayer::new()),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(Arc::new(self.state))
            .fallback(handler_404)
    }

    async fn serve_http(self, server: &str, port: u16) -> Result<(), std::io::Error> {
        let address = format!("{}:{}", server, port);
        let listener = tokio::net::TcpListener::bind(address).await.unwrap();
        tracing::info!("Listening on http://{}", listener.local_addr().unwrap());
        axum::serve(listener, self.router()).await
    }

    async fn serve_tls(
        self,
        server: &str,
        port: u16,
        tls_cert: String,
        tls_key: String,
    ) -> Result<(), std::io::Error> {
        let address = format!("{}:{}", server, port);
        let config = RustlsConfig::from_pem_file(tls_cert, tls_key).await?;
        tracing::info!("Listening on https://{}", address);
        axum_server::bind_rustls(SocketAddr::from_str(&address).unwrap(), config)
            .serve(self.router().into_make_service())
            .await
    }

    pub async fn serve(
        self,
        server: &str,
        port: u16,
        tls_cert: Option<String>,
        tls_key: Option<String>,
    ) -> Result<(), std::io::Error> {
        if tls_cert.is_none() || tls_key.is_none() {
            self.serve_http(server, port).await
        } else {
            self.serve_tls(server, port, tls_cert.unwrap(), tls_key.unwrap())
                .await
        }
    }
}

pub type AppState = Arc<GlobalState>;
pub type AppRouter = Router<AppState>;

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here")
}
