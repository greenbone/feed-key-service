![Greenbone Logo](https://www.greenbone.net/wp-content/uploads/gb_new-logo_horizontal_rgb_small.png)

# feed-key-service <!-- omit in toc -->

Service for Managing a Greenbone Feed Key

- [Installation](#installation)
- [Usage](#usage)
- [Settings](#settings)
- [TLS](#tls)
- [JWT](#jwt)
- [CLI](#cli)
- [Maintainer](#maintainer)
- [Contributing](#contributing)
- [License](#license)

## Installation

The project contains the `greenbone-feed-key` application which implements a
HTTP service providing a REST based API. It is implemented in [Rust] and
requires [cargo] for building and installing.

```sh
make DESTDIR=path/to/install install
```

The binary can be found at `path/to/install/usr/local/bin` afterwards.

## Usage

After installation the service is available as `greenbone-feed-key`. By default
it listens for http on `127.0.0.1` on port `3000`. Running the service requires
setting a [JWT] key. See [settings](#settings) the [JWT README](./jwt/README.md)
for all possible options.

Example using a shared secret

```sh
greenbone-feed-key --jwt-secret-key some-secret-key
```

## Settings

The following settings can be adjusted for the `greenbone-feed-key` service.

| CLI                   | Env                                    | Type   | Default                                  | Description                                                                                                               |
| --------------------- | -------------------------------------- | ------ | ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `-p, --port`          | `GREENBONE_FEED_KEY_PORT`              | int    | `3000`                                   | Port to listen on                                                                                                         |
| `-s, --server`        | `GREENBONE_FEED_KEY_SERVER`            | string | `127.0.0.1`                              | IP address to listen on                                                                                                   |
| `-k, --feed-key-path` | `GREENBONE_FEED_KEY_PATH`              | path   | `/etc/gvm/greenbone-enterprise-feed-key` | Path to the enterprise feed key location                                                                                  |
| `-l, --log`           | `GREENBONE_FEED_KEY_LOG`               | string | `greenbone_feed_key=info`                | [Logging directive](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives) |
| `--enable-api-doc`    | `GREENBONE_FEED_KEY_API_DOC`           | bool   | false                                    | Enable OpenAPI documentation and Swagger UI                                                                               |
| `--tls-server-cert`   | `GREENBONE_FEED_KEY_TLS_SERVER_CERT`   | string |                                          | Path to a TLS certificate (`.pem`) file                                                                                   |
| `--tls-server-key`    | `GREENBONE_FEED_KEY_TLS_SERVER_KEY`    | string |                                          | Path to a TLS private key file                                                                                            |
| `--tls-client-certs`  | `GREENBONE_FEED_KEY_TLS_CLIENT_CERTS`  | string |                                          | Path to a `.pem` file containing one ore more root certificates (aka. CA certs)                                           |
| `--upload-limit`      | `GREENBONE_FEED_KEY_UPLOAD_LIMIT`      | int    | 2 MiB                                    | File size limit for the feed key in bytes                                                                                 |
| `--jwt-shared-secret` | `GREENBONE_FEED_KEY_JWT_SHARED_SECRET` | string |                                          | A shared secret for validating [JSON Web Tokens](https://en.wikipedia.org/wiki/JSON_Web_Token)                            |
| `--jwt-rsa-key`       | `GREENBONE_FEED_KEY_JWT_RSA_KEY`       | path   |                                          | Path to a `.pem` file containing a RSA public key for [JWT] signature validation                                          |
| `--jwt-ecdsa-key`     | `GREENBONE_FEED_KEY_JWT_ECDSA_KEY`     | path   |                                          | Path to a `.pem` file containing an ECDSA public key (ECDSA using P-256 and SHA-256) for [JWT] signature validation       |

## TLS

[TLS](https://en.wikipedia.org/wiki/Transport_Layer_Security) can be enabled for
secure communication with the greenbone-feed-key service.

See [TLS documentation](./certs/README.md) for more details

## JWT

[JSON Web Tokens][JWT] are used to secure the key API.

See [JWT documentation](./jwt/README.md) for more details.

## CLI

Additionally to the service the project provides a CLI helper tool
`greenbone-feed-service-cli`. Currently it implements the following features:

- `jwt` - Generating JSON Web tokens for testing purposes
- `openapi` - Generating the [OpenAPI] spec file

Run `greenbone-feed-service-cli --help` for more details.

## Maintainer

This project is maintained by [Greenbone AG][Greenbone].

## Contributing

Your contributions are highly appreciated. Please [create a pull
request](https://github.com/greenbone/feed-key-service/pulls) on GitHub. Bigger changes need
to be discussed with the development team via the [issues section at
github](https://github.com/greenbone/feed-key-service/issues) first.

## License

Copyright (C) 2026 [Greenbone AG][Greenbone]

Licensed under the [GNU Affero General Public License v3.0 or later](LICENSE).

[Greenbone]: https://www.greenbone.net/
[JWT]: https://en.wikipedia.org/wiki/JSON_Web_Token
[Rust]: https://rust-lang.org/
[cargo]: https://doc.rust-lang.org/stable/cargo/
[OpenAPI]: https://www.openapis.org/
