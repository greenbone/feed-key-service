// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::Duration;
use greenbone_feed_key::{
    cli::{Cli, cli::Commands},
    jwt::{Claims, JwtEncodeSecret, generate_token},
};

fn main() {
    let cli = Cli::default();
    match cli.command {
        Commands::Jwt(cmd) => {
            let claims = Claims::new(cmd.subject, Duration::seconds(cmd.duration as i64));
            let secret = if let Some(secret) = cmd.secret.secret {
                Ok(JwtEncodeSecret::from_shared_secret(&secret))
            } else if let Some(rsa_key_path) = cmd.secret.rsa_key {
                match std::fs::read(&rsa_key_path) {
                    Ok(key_data) => match JwtEncodeSecret::from_rsa_pem(&key_data) {
                        Ok(secret) => Ok(secret),
                        Err(e) => Err(format!(
                            "Error reading RSA key from {:?}: {}",
                            rsa_key_path, e,
                        )),
                    },
                    Err(e) => Err(format!(
                        "Error reading RSA key from {:?}: {}",
                        rsa_key_path, e,
                    )),
                }
            } else if let Some(ecdsa_key_path) = cmd.secret.ecdsa_key {
                match std::fs::read(&ecdsa_key_path) {
                    Ok(key_data) => match JwtEncodeSecret::from_ec_pem(&key_data) {
                        Ok(secret) => Ok(secret),
                        Err(e) => Err(format!(
                            "Error reading ECDSA key from {:?}: {}",
                            ecdsa_key_path, e,
                        )),
                    },
                    Err(e) => Err(format!(
                        "Error reading ECDSA key from {:?}: {}",
                        ecdsa_key_path, e,
                    )),
                }
            } else {
                Err("No JWT secret provided".to_string())
            };
            let secret = match secret {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };
            match generate_token(&secret, &claims) {
                Ok(token) => {
                    println!("{}", token)
                }
                Err(e) => {
                    eprintln!("Error generating token: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::OpenAPI(cmd) => {
            match greenbone_feed_key::service::openapi::generate_openapi_json() {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&cmd.output, json) {
                        eprintln!("Error writing OpenAPI JSON to {:?}: {}", cmd.output, e);
                        std::process::exit(1);
                    } else {
                        println!("OpenAPI documentation written to {:?}", cmd.output);
                    }
                }
                Err(e) => {
                    eprintln!("Error generating OpenAPI JSON: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };
}
