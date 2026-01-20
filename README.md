![Greenbone Logo](https://www.greenbone.net/wp-content/uploads/gb_new-logo_horizontal_rgb_small.png)

# greenbone-feed-key <!-- omit in toc -->

Service for Uploading a Greenbone Feed Key

- [Settings](#settings)
- [TLS](#tls)
- [Maintainer](#maintainer)
- [Contributing](#contributing)
- [License](#license)

## Settings

| CLI                   | Env                                    | Type   | Default                                  | Description                                                                                                               |
| --------------------- | -------------------------------------- | ------ | ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `-p, --port`          | `GREENBONE_FEED_KEY_PORT`              | int    | `3000`                                   | Port to listen on                                                                                                         |
| `-s, --server`        | `GREENBONE_FEED_KEY_SERVER`            | string | `127.0.0.1`                              | IP address to listen on                                                                                                   |
| `-k, --feed-key-path` | `GREENBONE_FEED_KEY_PATH`              | string | `/etc/gvm/greenbone-enterprise-feed-key` | Path to the enterprise feed key location                                                                                  |
| `-l, --log`           | `GREENBONE_FEED_KEY_LOG`               | string | `greenbone_feed_key=info`                | [Logging directive](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives) |
| `--tls-server-cert`   | `GREENBONE_FEED_KEY_TLS_SERVER_CERT`   | string |                                          | Path to a TLS certificate (`.pem`) file                                                                                   |
| `--tls-server-key`    | `GREENBONE_FEED_KEY_TLS_SERVER_KEY`    | string |                                          | Path to a TLS private key file                                                                                            |
| `--tls-client-certs`  | `GREENBONE_FEED_KEY_TLS_CLIENT_CERTS`  | string |                                          | Path to a `.pem` file containing one ore more root certificates (aka. CA certs)                                           |
| `--upload-limit`      | `GREENBONE_FEED_KEY_UPLOAD_LIMIT`      | int    | 2 MiB                                    | File size limit for the feed key in bytes                                                                                 |
| `--jwt-shared-secret` | `GREENBONE_FEED_KEY_JWT_SHARED_SECRET` | string |                                          | A shared secret for validating [JSON Web Tokens](https://en.wikipedia.org/wiki/JSON_Web_Token)                            |

## TLS

See [TLS documentation](./certs/README.md) for more details

## Maintainer

This project is maintained by [Greenbone AG][Greenbone].

## Contributing

Your contributions are highly appreciated. Please [create a pull
request](https://github.com/greenbone/greenbone-feed-key/pulls) on GitHub. Bigger changes need
to be discussed with the development team via the [issues section at
github](https://github.com/greenbone/greenbone-feed-key/issues) first.

## License

Copyright (C) 2026 [Greenbone AG][Greenbone]

Licensed under the [GNU Affero General Public License v3.0 or later](LICENSE).

[Greenbone]: https://www.greenbone.net/
