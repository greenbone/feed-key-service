// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{Args, Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Port to listen on
    #[arg(short, long, env = "GREENBONE_FEED_KEY_PORT", default_value_t = 3000)]
    pub port: u16,

    /// Server IP address to bind to
    #[arg(short, long, env = "GREENBONE_FEED_KEY_SERVER", default_value_t = String::from("127.0.0.1"))]
    pub server: String,

    /// Path to the feed key file
    #[arg(short = 'k', long, env = "GREENBONE_FEED_KEY_PATH", default_value_t = String::from("/etc/gvm/greenbone-enterprise-feed-key"))]
    pub feed_key_path: String,

    /// Tracing log level directive
    #[arg(short, long, env = "GREENBONE_FEED_KEY_LOG", default_value_t = format!("{}=info", env!("CARGO_CRATE_NAME")))]
    pub log: String,

    /// Serve OpenAPI documentation
    #[arg(long, env = "GREENBONE_FEED_KEY_API_DOC", default_value_t = false)]
    pub enable_api_doc: bool,

    /// Path to TLS server certificate (.pem) file (enables HTTPS)
    #[arg(
        long,
        env = "GREENBONE_FEED_KEY_TLS_SERVER_CERT",
        requires = "tls_server_key"
    )]
    pub tls_server_cert: Option<String>,

    /// Path to TLS server key file (enables HTTPS)
    #[arg(
        long,
        env = "GREENBONE_FEED_KEY_TLS_SERVER_KEY",
        requires = "tls_server_cert"
    )]
    pub tls_server_key: Option<String>,

    /// Path to TLS client certificates (.pem) file (enables mTLS)
    #[arg(
        long,
        env = "GREENBONE_FEED_KEY_TLS_CLIENT_CERTS",
        requires = "tls_server_cert",
        requires = "tls_server_key"
    )]
    pub tls_client_certs: Option<String>,

    /// Maximum upload size in bytes for feed key uploads
    #[arg(long, env = "GREENBONE_FEED_KEY_UPLOAD_LIMIT")]
    pub upload_limit: Option<usize>,

    #[command(flatten)]
    pub jwt_secret: JwtSecretGroup,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct JwtSecretGroup {
    /// JWT shared secret for securing upload and delete operations
    #[arg(long, env = "GREENBONE_FEED_KEY_JWT_SHARED_SECRET")]
    pub jwt_shared_secret: Option<String>,

    /// JWT RSA key path for securing upload and delete operations
    #[arg(long, env = "GREENBONE_FEED_KEY_JWT_RSA_KEY")]
    pub jwt_rsa_key: Option<String>,

    /// JWT ECDSA key path for securing upload and delete operations
    #[arg(long, env = "GREENBONE_FEED_KEY_JWT_ECDSA_KEY")]
    pub jwt_ecdsa_key: Option<String>,
}

impl Default for Cli {
    fn default() -> Cli {
        Cli::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn try_parse_from(args: Vec<&str>) -> Result<Cli, clap::error::Error> {
        Cli::try_parse_from(vec!["test"].into_iter().chain(args.into_iter()))
    }

    fn try_parse_from_with_required(args: Vec<&str>) -> Result<Cli, clap::error::Error> {
        let mut required_args = vec!["--jwt-shared-secret", "dummy"];
        required_args.extend(args);
        try_parse_from(required_args)
    }

    #[test]
    fn test_default_cli() {
        let cli = try_parse_from_with_required(vec![]).unwrap();

        assert_eq!(cli.port, 3000);
        assert_eq!(cli.server, "127.0.0.1");
        assert_eq!(cli.feed_key_path, "/etc/gvm/greenbone-enterprise-feed-key");
        assert_eq!(cli.log, format!("{}=info", env!("CARGO_CRATE_NAME")));
        assert_eq!(cli.enable_api_doc, false);
        assert_eq!(cli.tls_server_cert, None);
        assert_eq!(cli.tls_server_key, None);
        assert_eq!(cli.tls_client_certs, None);
        assert_eq!(cli.upload_limit, None);
        assert_eq!(
            cli.jwt_secret.jwt_shared_secret,
            Some(String::from("dummy"))
        );
        assert_eq!(cli.jwt_secret.jwt_rsa_key, None);
        assert_eq!(cli.jwt_secret.jwt_ecdsa_key, None);
    }

    #[test]
    fn test_parse_port() {
        let cli = try_parse_from_with_required(vec!["--port", "8080"]).unwrap();
        assert_eq!(cli.port, 8080);
    }

    #[test]
    fn test_parse_server() {
        let cli = try_parse_from_with_required(vec!["--server", "0.0.0.0"]).unwrap();
        assert_eq!(cli.server, "0.0.0.0");
    }

    #[test]
    fn test_parse_feed_key_path() {
        let cli = try_parse_from_with_required(vec!["--feed-key-path", "/tmp/key"]).unwrap();
        assert_eq!(cli.feed_key_path, "/tmp/key");
    }

    #[test]
    fn test_parse_log() {
        let cli = try_parse_from_with_required(vec!["--log", "my_crate=debug"]).unwrap();
        assert_eq!(cli.log, "my_crate=debug");
    }

    #[test]
    fn test_parse_tls_server_cert_without_key() {
        let cli = try_parse_from_with_required(vec!["--tls-server-cert", "/tmp/cert.pem"]);
        assert!(cli.is_err());
        assert_eq!(
            cli.err().unwrap().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn test_parse_tls_server_key_without_cert() {
        let cli = try_parse_from_with_required(vec!["--tls-server-key", "/tmp/key.pem"]);
        assert!(cli.is_err());
        assert_eq!(
            cli.err().unwrap().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn test_parse_tls_server_cert_and_key() {
        let cli = try_parse_from_with_required(vec![
            "--tls-server-cert",
            "/tmp/cert.pem",
            "--tls-server-key",
            "/tmp/key.pem",
        ])
        .unwrap();
        assert_eq!(cli.tls_server_cert, Some(String::from("/tmp/cert.pem")));
        assert_eq!(cli.tls_server_key, Some(String::from("/tmp/key.pem")));
    }

    #[test]
    fn test_parse_tls_client_certs_without_server_cert_and_key() {
        let cli = try_parse_from_with_required(vec!["--tls-client-certs", "/tmp/ca.pem"]);
        assert!(cli.is_err());
        assert_eq!(
            cli.err().unwrap().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn test_parse_tls_client_certs_with_server_cert_and_key() {
        let cli = try_parse_from_with_required(vec![
            "--tls-server-cert",
            "/tmp/cert.pem",
            "--tls-server-key",
            "/tmp/key.pem",
            "--tls-client-certs",
            "/tmp/ca.pem",
        ])
        .unwrap();
        assert_eq!(cli.tls_client_certs, Some(String::from("/tmp/ca.pem")));
        assert_eq!(cli.tls_server_cert, Some(String::from("/tmp/cert.pem")));
        assert_eq!(cli.tls_server_key, Some(String::from("/tmp/key.pem")));
    }

    #[test]
    fn test_parse_upload_limit() {
        let cli = try_parse_from_with_required(vec!["--upload-limit", "1048576"]).unwrap();
        assert_eq!(cli.upload_limit, Some(1048576));
    }

    #[test]
    fn test_parse_jwt_shared_secret() {
        let cli = try_parse_from(vec!["--jwt-shared-secret", "mysecret"]).unwrap();
        assert_eq!(
            cli.jwt_secret.jwt_shared_secret,
            Some(String::from("mysecret"))
        );
        assert_eq!(cli.jwt_secret.jwt_rsa_key, None);
        assert_eq!(cli.jwt_secret.jwt_ecdsa_key, None);
    }

    #[test]
    fn test_parse_jwt_rsa_key() {
        let cli = try_parse_from(vec!["--jwt-rsa-key", "/tmp/rsa_key.pem"]).unwrap();
        assert_eq!(
            cli.jwt_secret.jwt_rsa_key,
            Some(String::from("/tmp/rsa_key.pem"))
        );
        assert_eq!(cli.jwt_secret.jwt_shared_secret, None);
        assert_eq!(cli.jwt_secret.jwt_ecdsa_key, None);
    }

    #[test]
    fn test_parse_jwt_ecdsa_key() {
        let cli = try_parse_from(vec!["--jwt-ecdsa-key", "/tmp/ecdsa_key.pem"]).unwrap();
        assert_eq!(
            cli.jwt_secret.jwt_ecdsa_key,
            Some(String::from("/tmp/ecdsa_key.pem"))
        );
        assert_eq!(cli.jwt_secret.jwt_shared_secret, None);
        assert_eq!(cli.jwt_secret.jwt_rsa_key, None);
    }

    #[test]
    fn test_parse_enable_api_doc() {
        let cli = try_parse_from_with_required(vec!["--enable-api-doc"]).unwrap();
        assert_eq!(cli.enable_api_doc, true);
    }
}
