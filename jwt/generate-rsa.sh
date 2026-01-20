#!/bin/sh
#
# SPDX-FileCopyrightText: 2026 Greenbone AG
#
# SPDX-License-Identifier: AGPL-3.0-or-later

# This script generates an RSA private key and corresponding public key
# for use in JWT encoding and decoding.

set -e

KEY_SIZE=${KEY_SIZE:-4096}  # Default key size if not set
PRIVATE_KEY=${PRIVATE_KEY:-rsa.private.pem}  # Default private key output file if not set
PUBLIC_KEY=${PUBLIC_KEY:-rsa.public.pem}  # Default public key output file if

openssl genpkey \
  -algorithm RSA \
  -outform PEM \
  -quiet \
  -out "${PRIVATE_KEY}" \
  -pkeyopt rsa_keygen_bits:"${KEY_SIZE}"

openssl rsa \
  -in "${PRIVATE_KEY}" \
  -noout \
  -pubout \
  -out "${PUBLIC_KEY}"
