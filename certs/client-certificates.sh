#!/bin/sh
#
# SPDX-FileCopyrightText: 2026 Greenbone AG
#
# SPDX-License-Identifier: AGPL-3.0-or-later

set -e

if [ -f .env ]; then
  # shellcheck source=/dev/null
  . ./.env
fi

CA_CERT=${CA_CERT:-ca.cert.pem}  # Default CA cert file if not set
CA_KEY=${CA_KEY:-ca.key}  # Default CA key file if not set
CLIENT_CERT=${CLIENT_CERT:-client.cert.pem}  # Default client cert output file if not set
CLIENT_KEY=${CLIENT_KEY:-client.key}  # Default client key output file if not set
CLIENT_CSR=${CLIENT_CSR:-client.csr}  # Default client CSR output file if not set
CLIENT_DAYS=${CLIENT_DAYS:-825}  # Default to ~2.25 years if not set
CLIENT_NAME=${CLIENT_NAME:-localhost}  # Default client CN name if not set
CLIENT_ENCRYPT_KEY=${CLIENT_ENCRYPT_KEY:-0} # Default to not encrypt the client key if not set
KEY_SIZE=${KEY_SIZE:-4096}  # Default key size if not set
CLIENT_KEY_ALGORITHM=${CLIENT_KEY_ALGORITHM:-rsa:${KEY_SIZE}}  # Default key algorithm if not set

CLIENT_ENCRYPTION_OPTION=-noenc
if [ "${CLIENT_ENCRYPT_KEY}" = "1" ]; then
  CLIENT_ENCRYPTION_OPTION=
fi

openssl req \
  -newkey "${CLIENT_KEY_ALGORITHM}" \
  ${CLIENT_ENCRYPTION_OPTION:+"${CLIENT_ENCRYPTION_OPTION}"} \
  -keyout "${CLIENT_KEY}" \
  -out "${CLIENT_CSR}" \
  -subj "/CN=${CLIENT_NAME}"

openssl x509 \
  -req \
  -in "${CLIENT_CSR}" \
  -CA "${CA_CERT}" \
  -CAkey "${CA_KEY}" \
  -CAcreateserial \
  -days "${CLIENT_DAYS}" \
  -out "${CLIENT_CERT}" \
  -extfile ./openssl.cnf \
  -extensions v3_client

rm -f "${CLIENT_CSR}"
