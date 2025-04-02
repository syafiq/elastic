use rustls::{Certificate, PrivateKey, RootCertStore};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[derive(Clone)]
pub struct CertificateManager {
    cert_store: RootCertStore,
    server_cert: Option<Arc<Certificate>>,
    server_key: Option<Arc<PrivateKey>>,
}

impl CertificateManager {
    pub fn new() -> Self {
        Self {
            cert_store: RootCertStore::empty(),
            server_cert: None,
            server_key: None,
        }
    }

    pub fn load_certificate(&mut self, cert_path: &str) -> Result<(), String> {
        let cert_file = File::open(cert_path)
            .map_err(|e| format!("Failed to open certificate file: {}", e))?;
        let mut reader = BufReader::new(cert_file);
        let certs = certs(&mut reader)
            .map_err(|e| format!("Failed to parse certificate: {}", e))?;
        
        if certs.is_empty() {
            return Err("No certificates found in file".to_string());
        }

        self.server_cert = Some(Arc::new(Certificate(certs[0].clone())));
        Ok(())
    }

    pub fn load_private_key(&mut self, key_path: &str) -> Result<(), String> {
        let key_file = File::open(key_path)
            .map_err(|e| format!("Failed to open key file: {}", e))?;
        let mut reader = BufReader::new(key_file);
        let keys = pkcs8_private_keys(&mut reader)
            .map_err(|e| format!("Failed to parse private key: {}", e))?;
        
        if keys.is_empty() {
            return Err("No private keys found in file".to_string());
        }

        self.server_key = Some(Arc::new(PrivateKey(keys[0].clone())));
        Ok(())
    }

    pub fn load_ca_certificates(&mut self, ca_path: &str) -> Result<(), String> {
        let ca_file = File::open(ca_path)
            .map_err(|e| format!("Failed to open CA certificate file: {}", e))?;
        let mut reader = BufReader::new(ca_file);
        let certs = certs(&mut reader)
            .map_err(|e| format!("Failed to parse CA certificates: {}", e))?;
        
        for cert_data in certs {
            self.cert_store.add(&Certificate(cert_data))
                .map_err(|e| format!("Failed to add CA certificate: {}", e))?;
        }

        Ok(())
    }

    pub fn get_server_cert(&self) -> Option<Certificate> {
        self.server_cert.as_ref().map(|cert| Certificate(cert.0.clone()))
    }

    pub fn get_server_key(&self) -> Option<PrivateKey> {
        self.server_key.as_ref().map(|key| PrivateKey(key.0.clone()))
    }

    pub fn get_cert_store(&self) -> &RootCertStore {
        &self.cert_store
    }
} 