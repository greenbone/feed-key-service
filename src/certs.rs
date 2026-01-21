// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use rustls::{
    RootCertStore,
    pki_types::{
        CertificateDer, PrivateKeyDer,
        pem::{self, PemObject},
    },
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CertificateError {
    #[error("Could not load certificate from {0}. {1}")]
    CertificateLoadingError(PathBuf, pem::Error),

    #[error("Could not load key from {0}. {1}")]
    KeyLoadingError(PathBuf, pem::Error),

    #[error("Could not load certificate chain from {0}. {1}")]
    CertificateChainLoadingError(PathBuf, pem::Error),

    #[error("Could not add certificate to root store. {0}")]
    RootCertStoreError(rustls::Error),
}

pub fn load_certificate(cert_path: &Path) -> Result<CertificateDer<'static>, CertificateError> {
    tracing::debug!(cert_path = ?cert_path, "loading certificate");
    let cert = CertificateDer::from_pem_file(cert_path)
        .map_err(|e| CertificateError::CertificateLoadingError(cert_path.into(), e))?;
    tracing::debug!(cert = ?cert, "certificate loaded successfully");
    Ok(cert)
}

pub fn load_private_key(key_path: &Path) -> Result<PrivateKeyDer<'static>, CertificateError> {
    tracing::debug!(key_path = ?key_path, "loading private key");
    let key = PrivateKeyDer::from_pem_file(key_path)
        .map_err(|e| CertificateError::KeyLoadingError(key_path.into(), e))?;
    tracing::debug!(key = ?key, "private key loaded successfully");
    Ok(key)
}

pub fn load_certificates(
    cert_path: &Path,
) -> Result<Vec<CertificateDer<'static>>, CertificateError> {
    tracing::debug!(cert_path = ?cert_path, "loading certificates");
    let certs = CertificateDer::pem_file_iter(cert_path)
        .map_err(|e| CertificateError::CertificateChainLoadingError(cert_path.into(), e))?
        .collect::<Result<Vec<CertificateDer<'static>>, pem::Error>>()
        .map_err(|e| CertificateError::CertificateChainLoadingError(cert_path.into(), e))?;
    tracing::debug!(certs_len = certs.len(), "certificates loaded successfully");
    Ok(certs)
}

pub fn create_client_root_cert_store(
    client_cert_path: &Path,
) -> Result<RootCertStore, CertificateError> {
    tracing::debug!(client_cert_path = ?client_cert_path, "creating client cert store");
    let client_certs = load_certificates(client_cert_path)?;
    let mut root_store = RootCertStore::empty();
    for cert in client_certs {
        root_store
            .add(cert)
            .map_err(|e| CertificateError::RootCertStoreError(e))?;
    }
    tracing::debug!(client_certs_len = ?root_store.len(), "client cert store created successfully");
    if root_store.is_empty() {
        tracing::warn!("client cert store is empty");
    }
    Ok(root_store)
}
