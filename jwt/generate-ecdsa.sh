#!/bin/sh
#
# SPDX-FileCopyrightText: 2026 Greenbone AG
#
# SPDX-License-Identifier: AGPL-3.0-or-later

# This script generates an ECDSA private key and corresponding public key
# for use in JWT encoding and decoding.

set -e

PRIVATE_KEY=${PRIVATE_KEY:-ecdsa.private.pem}  # Default private key output file if not set
PUBLIC_KEY=${PUBLIC_KEY:-ecdsa.public.pem}  # Default public key output file if not set
CURVE=${CURVE:-"P-256"}  # Default curve if not set

openssl genpkey \
  -algorithm EC \
  -outform PEM \
  -quiet \
  -out "${PRIVATE_KEY}" \
  -pkeyopt ec_paramgen_curve:"${CURVE}" \
  -pkeyopt ec_param_enc:named_curve

openssl ec \
  -in "${PRIVATE_KEY}" \
  -noout \
  -pubout \
  -outform PEM \
  -out "${PUBLIC_KEY}"
