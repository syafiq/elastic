use std::fs::File;
use std::io::BufReader;
use tokio_rustls::rustls::{Certificate, PrivateKey, RootCertStore};
use std::io;

#[derive(Clone)]
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

    pub fn load_certificate(&mut self, cert_path: &str) -> io::Result<()> {
        let cert_file = File::open(cert_path)?;
        let mut reader = BufReader::new(cert_file);
        let certs = rustls_pemfile::certs(&mut reader)?;
        if let Some(cert) = certs.into_iter().next() {
            self.server_cert = Some(Certificate(cert));
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "No certificate found"))
        }
    }

    pub fn load_private_key(&mut self, key_path: &str) -> io::Result<()> {
        let key_file = File::open(key_path)?;
        let mut reader = BufReader::new(key_file);
        let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)?;
        if let Some(key) = keys.into_iter().next() {
            self.server_key = Some(PrivateKey(key));
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "No private key found"))
        }
    }

    pub fn load_ca_certificates(&mut self, ca_path: &str) -> io::Result<()> {
        let ca_file = File::open(ca_path)?;
        let mut reader = BufReader::new(ca_file);
        let certs = rustls_pemfile::certs(&mut reader)?;
        for cert in certs {
            self.cert_store.add(&Certificate(cert))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        }
        Ok(())
    }

    pub fn get_server_cert(&self) -> Option<Certificate> {
        self.server_cert.clone()
    }

    pub fn get_server_key(&self) -> Option<PrivateKey> {
        self.server_key.clone()
    }

    pub fn get_cert_store(&self) -> &RootCertStore {
        &self.cert_store
    }
} 