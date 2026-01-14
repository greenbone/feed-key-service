// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Port to listen on
    #[arg(short, long, env = "GREENBONE_FEED_KEY_PORT", default_value_t = 3000)]
    pub port: u16,

    /// Server address to bind to
    #[arg(short, long, env = "GREENBONE_FEED_KEY_SERVER", default_value_t = String::from("127.0.0.1"))]
    pub server: String,

    /// Path to the key directory
    #[arg(short, long, env = "GREENBONE_FEED_KEY_PATH", default_value_t = String::from("/etc/gvm/greenbone-enterprise-feed-key"))]
    pub key_path: String,

    /// Tracing log level directive
    #[arg(short, long, env = "GREENBONE_FEED_KEY_LOG", default_value_t = format!("{}=info", env!("CARGO_CRATE_NAME")))]
    pub log: String,

    /// Path to TLS certificate file (enables HTTPS)
    #[arg(long, env = "GREENBONE_FEED_KEY_TLS_CERT", requires = "tls_key")]
    pub tls_cert: Option<String>,

    /// Path to TLS key file (enables HTTPS)
    #[arg(long, env = "GREENBONE_FEED_KEY_TLS_KEY", requires = "tls_cert")]
    pub tls_key: Option<String>,
}

impl Default for Cli {
    fn default() -> Cli {
        Cli::parse()
    }
}
