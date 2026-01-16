// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{app::App, cli::Cli};

mod api;
mod app;
mod certs;
mod cli;
mod openapi;

#[tokio::main]
async fn main() {
    let cli = Cli::default();
    let app = App::new(cli.feed_key_path.into(), cli.log, cli.upload_limit);
    let result = app
        .serve(
            cli.server,
            cli.port,
            cli.tls_server_cert,
            cli.tls_server_key,
            cli.tls_client_certs,
        )
        .await;
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
