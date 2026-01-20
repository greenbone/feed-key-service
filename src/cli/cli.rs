// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new JWT token
    Jwt(JwtCommand),
}

#[derive(Args)]
pub struct JwtCommand {
    #[command(flatten)]
    pub secret: JwtSecretGroup,

    /// Subject claim for the generated token
    #[arg(long, short = 'u')]
    pub subject: String,

    /// Duration in seconds for which the token is valid
    #[arg(long, short, default_value_t = 3600)]
    pub duration: u64,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct JwtSecretGroup {
    /// JWT secret for encoding and decoding tokens
    #[arg(
        long,
        short,
        env = "GREENBONE_FEED_KEY_JWT_SHARED_SECRET",
        group = "jwt-secret"
    )]
    pub secret: Option<String>,

    /// Path to RSA private key file for encoding tokens
    #[arg(
        long,
        short = 'r',
        env = "GREENBONE_FEED_KEY_JWT_RSA_KEY",
        group = "jwt-secret"
    )]
    pub rsa_key: Option<PathBuf>,

    /// Path to ECDSA private key file for encoding tokens
    #[arg(
        long,
        short = 'e',
        env = "GREENBONE_FEED_KEY_JWT_ECDSA_KEY",
        group = "jwt-secret"
    )]
    pub ecdsa_key: Option<PathBuf>,
}

impl Default for Cli {
    fn default() -> Cli {
        Cli::parse()
    }
}
