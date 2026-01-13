use crate::{app::App, cli::Cli};

mod app;
mod cli;
mod health;
mod key;
mod openapi;

#[tokio::main]
async fn main() {
    let cli = Cli::default();
    let app = App::new(cli.key_path.into());

    let address = format!("{}:{}", cli.server, cli.port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.router()).await.unwrap();
}
