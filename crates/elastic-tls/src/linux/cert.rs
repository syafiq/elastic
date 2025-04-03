use std::fs::File;
use std::io::BufReader;
use rustls::{Certificate, PrivateKey, RootCertStore};
use rustls_pemfile::{certs, pkcs8_private_keys};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CertError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid certificate: {0}")]
    InvalidCert(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("No certificates found")]
    NoCerts,
    #[error("No keys found")]
    NoKeys,
}

pub struct CertificateManager {
    server_cert: Option<Certificate>,
    server_key: Option<PrivateKey>,
    cert_store: RootCertStore,
}

impl CertificateManager {
    pub fn new() -> Self {
        Self {
            server_cert: None,
            server_key: None,
            cert_store: RootCertStore::empty(),
        }
    }

    pub fn load_certificate(&mut self, cert_path: &str) -> Result<(), CertError> {
        let cert_file = File::open(cert_path)?;
        let mut reader = BufReader::new(cert_file);
        let certs = certs(&mut reader)
            .map_err(|_| CertError::InvalidCert("Failed to parse certificate".to_string()))?;
        if certs.is_empty() {
            return Err(CertError::NoCerts);
        }
        self.server_cert = Some(Certificate(certs[0].clone()));
        Ok(())
    }

    pub fn load_private_key(&mut self, key_path: &str) -> Result<(), CertError> {
        let key_file = File::open(key_path)?;
        let mut reader = BufReader::new(key_file);
        let keys = pkcs8_private_keys(&mut reader)
            .map_err(|_| CertError::InvalidKey("Failed to parse private key".to_string()))?;
        if keys.is_empty() {
            return Err(CertError::NoKeys);
        }
        self.server_key = Some(PrivateKey(keys[0].clone()));
        Ok(())
    }

    pub fn load_ca_certificates(&mut self, ca_path: &str) -> Result<(), CertError> {
        let ca_file = File::open(ca_path)?;
        let mut reader = BufReader::new(ca_file);
        let certs = certs(&mut reader)
            .map_err(|_| CertError::InvalidCert("Failed to parse CA certificate".to_string()))?;
        for cert in certs {
            self.cert_store.add(&Certificate(cert))
                .map_err(|e| CertError::InvalidCert(e.to_string()))?;
        }
        Ok(())
    }

    pub fn get_server_cert(&self) -> Option<Certificate> {
        self.server_cert.clone()
    }

    pub fn get_server_key(&self) -> Option<PrivateKey> {
        self.server_key.clone()
    }

    pub fn get_cert_store(&self) -> RootCertStore {
        self.cert_store.clone()
    }
} 