#!/bin/sh
#
# SPDX-FileCopyrightText: 2026 Greenbone AG
#
# SPDX-License-Identifier: AGPL-3.0-or-later

# This script generates a shared secret for use in JWT encoding and decoding.

set -e
KEY_SIZE=${KEY_SIZE:-32}  # Default key size in bytes if not set. Default is 32 bytes (256 bits).

openssl rand \
  -base64 \
  "${KEY_SIZE}" \
  | tr -d '\n'
echo
