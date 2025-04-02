use std::sync::Arc;
use tokio_rustls::rustls::{self, ClientConfig, ServerConfig};
use thiserror::Error;
use wit_bindgen::rt::vec::Vec;
use rustls::client::ServerCertVerifier;
use rustls::client::ServerCertVerified;
use rustls::Error;
use rustls::RootCertStore;

mod cert;
mod conn;

use cert::CertificateManager;
use conn::ConnectionManager;

#[derive(Error, Debug)]
pub enum TlsError {
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Write failed: {0}")]
    WriteFailed(String),
    #[error("Read failed: {0}")]
    ReadFailed(String),
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    #[error("Unsupported cipher: {0}")]
    UnsupportedCipher(String),
}

#[derive(Debug, Clone, Copy)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

#[derive(Debug, Clone, Copy)]
pub enum CipherSuite {
    TlsAes128GcmSha256,
    TlsAes256GcmSha384,
    TlsChaCha20Poly1305Sha256,
}

#[derive(Clone)]
pub struct TlsConfig {
    pub version: TlsVersion,
    pub cipher_suites: Vec<CipherSuite>,
    pub verify_peer: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            version: TlsVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::TlsAes256GcmSha384,
                CipherSuite::TlsChaCha20Poly1305Sha256,
            ],
            verify_peer: true,
        }
    }
}

struct AcceptAllVerifier;

impl ServerCertVerifier for AcceptAllVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}

#[derive(Clone)]
pub struct TlsContext {
    cert_manager: CertificateManager,
    conn_manager: ConnectionManager,
}

impl TlsContext {
    pub fn new() -> Self {
        Self {
            cert_manager: CertificateManager::new(),
            conn_manager: ConnectionManager::new(),
        }
    }

    pub fn load_certificate(&mut self, cert_path: &str) -> Result<(), TlsError> {
        self.cert_manager.load_certificate(cert_path)
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))
    }

    pub fn load_private_key(&mut self, key_path: &str) -> Result<(), TlsError> {
        self.cert_manager.load_private_key(key_path)
            .map_err(|e| TlsError::InvalidKey(e.to_string()))
    }

    pub fn load_ca_certificates(&mut self, ca_path: &str) -> Result<(), TlsError> {
        self.cert_manager.load_ca_certificates(ca_path)
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))
    }

    pub async fn bind(&self, port: u16) -> Result<(), TlsError> {
        self.conn_manager.bind(port).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub async fn accept(&self, config: &TlsConfig) -> Result<u32, TlsError> {
        let server_cert = self.cert_manager.get_server_cert()
            .ok_or_else(|| TlsError::InvalidCertificate("Server certificate not loaded".to_string()))?;
        let server_key = self.cert_manager.get_server_key()
            .ok_or_else(|| TlsError::InvalidKey("Server key not loaded".to_string()))?;

        let mut server_config = if config.verify_peer {
            let verifier = rustls::server::AllowAnyAuthenticatedClient::new(
                self.cert_manager.get_cert_store().clone()
            );
            ServerConfig::builder()
                .with_safe_defaults()
                .with_client_cert_verifier(Arc::new(verifier))
                .with_single_cert(vec![server_cert.clone()], server_key.clone())
                .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?
        } else {
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(vec![server_cert.clone()], server_key.clone())
                .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?
        };

        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        
        self.conn_manager.accept(Arc::new(server_config)).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub async fn connect(&self, hostname: &str, port: u16, config: &TlsConfig) -> Result<u32, TlsError> {
        let client_config = if config.verify_peer {
            let mut root_store = RootCertStore::empty();
            root_store.add_trust_anchors(
                webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
                    rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                })
            );

            let mut config = ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            config
        } else {
            let mut config = ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(AcceptAllVerifier))
                .with_no_client_auth();

            config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            config
        };

        self.conn_manager.connect(Arc::new(client_config), hostname, port).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub fn close(&self, handle: u32) -> Result<(), TlsError> {
        self.conn_manager.close(handle)
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub async fn write(&self, handle: u32, data: &[u8]) -> Result<(), TlsError> {
        self.conn_manager.write(handle, data).await
            .map_err(|e| TlsError::WriteFailed(e.to_string()))
    }

    pub async fn read(&self, handle: u32, max_size: usize) -> Result<Vec<u8>, TlsError> {
        self.conn_manager.read(handle, max_size).await
            .map_err(|e| TlsError::ReadFailed(e.to_string()))
    }

    pub fn get_peer_certificate(&self, handle: u32) -> Result<Option<Vec<u8>>, TlsError> {
        self.conn_manager.get_peer_certificate(handle)
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))
    }

    pub fn get_protocol_version(&self, handle: u32) -> Result<TlsVersion, TlsError> {
        let version = self.conn_manager.get_protocol_version(handle)
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))?;
        
        match version {
            Some(rustls::ProtocolVersion::TLSv1_2) => Ok(TlsVersion::Tls12),
            Some(rustls::ProtocolVersion::TLSv1_3) => Ok(TlsVersion::Tls13),
            _ => Err(TlsError::UnsupportedProtocol(format!("{:?}", version))),
        }
    }

    pub fn get_cipher_suite(&self, handle: u32) -> Result<CipherSuite, TlsError> {
        let suite = self.conn_manager.get_cipher_suite(handle)
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))?;
        
        match suite {
            Some(suite) => {
                let suite_id = suite.suite();
                match suite_id {
                    rustls::CipherSuite::TLS13_AES_128_GCM_SHA256 => Ok(CipherSuite::TlsAes128GcmSha256),
                    rustls::CipherSuite::TLS13_AES_256_GCM_SHA384 => Ok(CipherSuite::TlsAes256GcmSha384),
                    rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256 => Ok(CipherSuite::TlsChaCha20Poly1305Sha256),
                    _ => Err(TlsError::UnsupportedCipher(format!("{:?}", suite_id))),
                }
            }
            None => Err(TlsError::UnsupportedCipher("No cipher suite negotiated".to_string())),
        }
    }
}

impl Drop for TlsContext {
    fn drop(&mut self) {
        // Clean up any remaining connections
        // The ConnectionManager's Drop implementation will handle this
    }
} 