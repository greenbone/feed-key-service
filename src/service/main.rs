// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use greenbone_feed_key::{
    jwt::JwtSecret,
    service::{app::App, cli::Cli},
};

fn load_key(key_path: &str) -> Result<Vec<u8>, String> {
    match std::fs::read(key_path) {
        Ok(key_data) => Ok(key_data),
        Err(e) => Err(format!("Error reading key from {:?}: {}", key_path, e)),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::default();
    let secret = if let Some(s) = cli.jwt_secret.jwt_shared_secret {
        Ok(JwtSecret::SharedSecret(s))
    } else if let Some(rsa_key_path) = cli.jwt_secret.jwt_rsa_key {
        match load_key(&rsa_key_path) {
            Ok(key) => Ok(JwtSecret::RsaKey(key)),
            Err(e) => Err(e),
        }
    } else if let Some(ecdsa_key_path) = cli.jwt_secret.jwt_ecdsa_key {
        match load_key(&ecdsa_key_path) {
            Ok(key) => Ok(JwtSecret::EcdsaKey(key)),
            Err(e) => Err(e),
        }
    } else {
        Err("Error: No JWT secret provided".to_string())
    };
    let secret = match secret {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    let app = App::new(
        cli.feed_key_path.into(),
        cli.log,
        cli.upload_limit,
        secret,
        cli.enable_api_doc,
    );
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
