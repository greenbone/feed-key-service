# JSON Web Tokens <!-- omit in toc -->

To protect the key API from unauthorized access [JSON Web Tokens][JWT] are used.

At the moment three different mechanism for creating the JWT signature are
supported

- [Shared Secret](#shared-secret)
- [RSA Key](#rsa-key)
- [ECDSA Key](#ecdsa-key)

## Shared Secret

The easiest but least secure solution is to share a secret between the JWT
issuer and the greenbone-feed-key service. Providing a shared secret will use
the `HS256` algorithm for the JWT.

To create a cryptographically secure token the following command can be used.
It generates a secure random token with 32 Byte (256 bit) length (by default).

```sh
./generate-secret.sh
```

The following command creates a secret with 64 Byte (512 bit) length.

```sh
KEY_SIZE=64 ./generate-secrete.sh
```

The shared secret can be used in the `greenbone-feed-key` service at best via a
environment variable.

```sh
export GREENBONE_FEED_KEY_JWT_SHARED_SECRET="your generated shared secret"
greenbone-feed-key
```

## RSA Key

As a better alternative for a shared secret private/public key pairs can be
used. Either RSA or ECDSA keys. The public key is used to validate the JWT
signature and the private key is required to generate the signature.

The RSA key will use the `RS256` algorithm for the JWT.

The following command creates a RSA private/public key pair with 4096 bits.

```sh
./generate-rsa.sh
```

The public key can be used in the `greenbone-feed-key` service via CLI argument
or environment variable.

```sh
greenbone-feed-key --jwt-rsa-key ./jwt/rsa.public.pem
```

```sh
export GREENBONE_FEED_KEY_JWT_RSA_KEY=./jwt/rsa.public.pem
greenbone-feed-key
```

## ECDSA Key

As an alternative for the RSA key an ECDSA key can be used.

The ECDSA key will use the `ES256` algorithm for the JWT. This requires to
create a private/public key pair using the `P-256` curve.

The following command creates a ECDSA private/public key pair with using the
`P-256` curve.

```sh
./generate-ecdsa.sh
```

The public key can be used in the `greenbone-feed-key` service via CLI argument
or environment variable.

```sh
greenbone-feed-key --jwt-ecdsa-key ./jwt/ecdsa.public.pem
```

```sh
export GREENBONE_FEED_KEY_JWT_ECDSA_KEY=./jwt/ecdsa.public.pem
greenbone-feed-key
```

[JWT]: https://en.wikipedia.org/wiki/JSON_Web_Token
