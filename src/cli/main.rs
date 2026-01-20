// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::Duration;
use greenbone_feed_key::{
    cli::{Cli, cli::Commands},
    jwt::{Claims, JwtSecret, generate_token},
};

fn main() {
    let cli = Cli::default();
    match cli.command {
        Commands::Jwt(cmd) => {
            let claims = Claims::new(cmd.subject, Duration::seconds(cmd.duration as i64));
            let secret = if let Some(secret) = cmd.secret.secret {
                Ok(JwtSecret::SharedSecret(secret))
            } else if let Some(rsa_key_path) = cmd.secret.rsa_key {
                match std::fs::read(&rsa_key_path) {
                    Ok(key_data) => Ok(JwtSecret::RsaKey(key_data)),
                    Err(e) => Err(format!(
                        "Error reading RSA key from {:?}: {}",
                        rsa_key_path, e,
                    )),
                }
            } else if let Some(ecdsa_key_path) = cmd.secret.ecdsa_key {
                match std::fs::read(&ecdsa_key_path) {
                    Ok(key_data) => Ok(JwtSecret::EcdsaKey(key_data)),
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
    };
}
