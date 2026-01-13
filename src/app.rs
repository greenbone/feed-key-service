use axum::{Router, http::StatusCode, response::IntoResponse};
use std::{
    path::{self, PathBuf},
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
    pub fn new(key_path: PathBuf) -> Self {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
            )
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
}

impl Into<Router> for App {
    fn into(self) -> Router {
        self.router()
    }
}

pub type AppState = Arc<GlobalState>;
pub type AppRouter = Router<AppState>;

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here")
}
