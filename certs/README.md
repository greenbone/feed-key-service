# TLS and Certificates <!-- omit in toc -->

This document explains how to handle TLS certificates for the feed key service.

The service uses [rustls] internally to handle the TLS connections.

- [Certificate Generation](#certificate-generation)
  - [Server Certificates Quick Start](#server-certificates-quick-start)
  - [Server Certificates with CA](#server-certificates-with-ca)
  - [Client Certificates with CA](#client-certificates-with-ca)
- [Setup TLS](#setup-tls)
- [Setup mTLS](#setup-mtls)
- [View Content of Certificates](#view-content-of-certificates)
  - [Server Private Key](#server-private-key)
  - [Server Certificate](#server-certificate)
  - [Server Certificate Signing Request (CSR)](#server-certificate-signing-request-csr)
  - [Client Private Key](#client-private-key)
  - [Client Certificate](#client-certificate)
  - [Client Certificate Signing Request (CSR)](#client-certificate-signing-request-csr)

## Certificate Generation

For a local setup it is possible to setup a self-signed certificate chain which
requires the `openssl` command to be installed.

### Server Certificates Quick Start

A self-signed TLS server private key and server certificate for testing purposes
can be generated with the following command easily

```sh
openssl req -newkey rsa:4096 -noenc -keyout server.key -x509 -days 365 -out server.cert.pem -subj "/CN=ACME" -batch
```

### Server Certificates with CA

Create CA private key and certificate (if not already created).

```sh
./ca-certificates.sh
```

Create server private key and certificate that is signed by the CA

```sh
./server-certificates.sh
```

### Client Certificates with CA

Create CA private key and certificate (if not already created).

```sh
./ca-certificates.sh
```

Create client private key and certificate that is signed by the CA

```sh
./client-certificates.sh
```

## Setup TLS

Via CLI

```sh
greenbone-feed-key --tls-server-key ./certs/server.key --tls-server-cert ./certs/server.cert.pem
```

Via Environment Variables

```sh
export GREENBONE_FEED_KEY_TLS_SERVER_KEY=./certs/server.key
export GREENBONE_FEED_KEY_TLS_SERVER_CERT=./certs/server.cert.pem
greenbone-feed-key
```

## Setup mTLS

Setting up [mTLS](https://www.cloudflare.com/learning/access-management/what-is-mutual-tls/)
requires providing a root certificate that has signed the actual client certificates.

Via CLI

```sh
greenbone-feed-key --tls-server-key ./certs/server.key --tls-server-cert ./certs/server.cert.pem --tls-client-certs ./certs/ca.cert.pem
```

Via Environment Variables

```sh
export GREENBONE_FEED_KEY_TLS_SERVER_KEY=./certs/server.key
export GREENBONE_FEED_KEY_TLS_SERVER_CERT=./certs/server.cert.pem
export GREENBONE_FEED_KEY_TLS_CLIENT_CERTS=./certs/ca.cert.pem
greenbone-feed-key
```

## View Content of Certificates

### Server Private Key

```sh
openssl rsa -noout -text -in ./server.key
```

### Server Certificate

```sh
openssl x509 -noout -text -in ./server.cert.pem
```

### Server Certificate Signing Request (CSR)

```sh
openssl req -noout -text -in ./server.csr
```

### Client Private Key

```sh
openssl rsa -noout -text -in ./client.key
```

### Client Certificate

```sh
openssl x509 -noout -text -in ./client.cert.pem
```

### Client Certificate Signing Request (CSR)

```sh
openssl req -noout -text -in ./client.csr
```

[rustls]: https://rustls.dev/
