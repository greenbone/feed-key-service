#!/bin/sh
#
# SPDX-FileCopyrightText: 2026 Greenbone AG
#
# SPDX-License-Identifier: AGPL-3.0-or-later


set -e

CA_DAYS=${CA_DAYS:-3650}  # Default to 10 years if not set
CA_NAME=${CA_NAME:-ACME Test CA}  # Default CN name if not set
SERVER_DAYS=${SERVER_DAYS:-825}  # Default to ~2.25 years if not set
SERVER_NAME=${SERVER_NAME:-localhost}  # Default server CN name if not set

openssl req \
  -x509 \
  -newkey rsa:4096 \
  -days ${CA_DAYS} \
  -noenc \
  -keyout ca.key.pem \
  -out ca.cert.pem \
  -subj "/CN=${CA_NAME}" \
  -config ./openssl.cnf \
  -extensions v3_ca

openssl req \
  -newkey rsa:4096 \
  -noenc \
  -keyout server.key.pem \
  -out server.csr.pem \
  -subj "/CN=${SERVER_NAME}"

openssl x509 \
  -req \
  -in server.csr.pem \
  -CA ca.cert.pem \
  -CAkey ca.key.pem \
  -CAcreateserial \
  -days ${SERVER_DAYS} \
  -out server.cert.pem \
  -extfile ./openssl.cnf \
  -extensions v3_server

