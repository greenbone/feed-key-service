// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{app::App, cli::Cli};

mod app;
mod cli;
mod health;
mod key;
mod openapi;

#[tokio::main]
async fn main() {
    let cli = Cli::default();
    let app = App::new(cli.key_path.into(), cli.log);
    app.serve(&cli.server, cli.port, cli.tls_cert, cli.tls_key)
        .await
        .unwrap();
}
