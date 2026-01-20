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
    #[arg(long, short, env = "GREENBONE_FEED_KEY_JWT_SHARED_SECRET")]
    pub secret: Option<String>,

    /// Path to RSA private key file for encoding tokens
    #[arg(long, short = 'r', env = "GREENBONE_FEED_KEY_JWT_RSA_KEY")]
    pub rsa_key: Option<PathBuf>,

    /// Path to ECDSA private key file for encoding tokens
    #[arg(long, short = 'e', env = "GREENBONE_FEED_KEY_JWT_ECDSA_KEY")]
    pub ecdsa_key: Option<PathBuf>,
}

impl Default for Cli {
    fn default() -> Cli {
        Cli::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn try_parse_from(args: Vec<&str>) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(vec!["test", "jwt"].into_iter().chain(args.into_iter()))
    }

    fn parse_from(args: Vec<&str>) -> Cli {
        try_parse_from(args).expect("Failed to parse CLI arguments")
    }

    #[test]
    fn test_should_use_defaults() {
        let cli = parse_from(vec!["--subject", "some-user", "--secret", "dummy"]);
        match cli.command {
            Commands::Jwt(cmd) => {
                assert_eq!(cmd.duration, 3600);
                assert_eq!(cmd.subject, "some-user");
                assert_eq!(cmd.secret.secret, Some(String::from("dummy")));
                assert_eq!(cmd.secret.rsa_key, None);
                assert_eq!(cmd.secret.ecdsa_key, None);
            }
        }
    }

    #[test]
    fn test_parse_duration() {
        let cli = parse_from(vec![
            "--subject",
            "some-user",
            "--secret",
            "dummy",
            "--duration",
            "7200",
        ]);
        match cli.command {
            Commands::Jwt(cmd) => {
                assert_eq!(cmd.duration, 7200);
            }
        }
    }

    #[test]
    fn test_parse_subject() {
        let cli = parse_from(vec!["--subject", "test-user", "--secret", "dummy"]);
        match cli.command {
            Commands::Jwt(cmd) => {
                assert_eq!(cmd.subject, "test-user");
            }
        }
    }

    #[test]
    fn test_parse_jwt_shared_secret() {
        let cli = parse_from(vec!["--secret", "mysecret", "--subject", "some-user"]);
        match cli.command {
            Commands::Jwt(cmd) => {
                assert_eq!(cmd.secret.secret, Some(String::from("mysecret")));
            }
        }
    }

    #[test]
    fn test_parse_jwt_rsa_key() {
        let cli = parse_from(vec![
            "--rsa-key",
            "/path/to/rsa_key",
            "--subject",
            "some-user",
        ]);
        match cli.command {
            Commands::Jwt(cmd) => {
                assert_eq!(cmd.secret.rsa_key, Some(PathBuf::from("/path/to/rsa_key")));
            }
        }
    }

    #[test]
    fn test_parse_jwt_ecdsa_key() {
        let cli = parse_from(vec![
            "--ecdsa-key",
            "/path/to/ecdsa_key",
            "--subject",
            "some-user",
        ]);
        match cli.command {
            Commands::Jwt(cmd) => {
                assert_eq!(
                    cmd.secret.ecdsa_key,
                    Some(PathBuf::from("/path/to/ecdsa_key"))
                );
            }
        }
    }

    #[test]
    fn test_should_fail_without_required_secret() {
        let result = try_parse_from(vec!["--subject", "some-user"]);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn test_should_fail_without_required_subject() {
        let result = try_parse_from(vec!["--secret", "dummy"]);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }
}
