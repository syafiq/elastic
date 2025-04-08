use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{self, ClientConfig, ServerConfig};
use thiserror::Error;

mod cert;
mod conn;

use cert::CertificateManager;
use conn::ConnectionManager;

#[derive(Debug, Error)]
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
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TLS error: {0}")]
    TlsError(#[from] rustls::Error),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CipherSuite {
    TlsAes128GcmSha256,
    TlsAes256GcmSha384,
    TlsChaCha20Poly1305Sha256,
}

#[derive(Debug, Clone)]
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

impl rustls::client::ServerCertVerifier for AcceptAllVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

pub struct TlsContext {
    cert_manager: CertificateManager,
    conn_manager: ConnectionManager,
    inner: Arc<Mutex<TlsContextInner>>,
}

struct TlsContextInner {
    server_config: Option<Arc<ServerConfig>>,
    client_config: Option<Arc<ClientConfig>>,
    listener: Option<TcpListener>,
}

impl TlsContext {
    pub fn new() -> Self {
        Self {
            cert_manager: CertificateManager::new(),
            conn_manager: ConnectionManager::new(),
            inner: Arc::new(Mutex::new(TlsContextInner {
                server_config: None,
                client_config: None,
                listener: None,
            })),
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

    pub async fn bind(&mut self, port: u16) -> Result<(), TlsError> {
        let mut inner = self.inner.lock().await;
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        inner.listener = Some(listener);
        Ok(())
    }

    pub async fn get_port(&self) -> Result<u16, TlsError> {
        let inner = self.inner.lock().await;
        if let Some(listener) = &inner.listener {
            Ok(listener.local_addr()?.port())
        } else {
            Err(TlsError::ConnectionFailed("No listener bound".to_string()))
        }
    }

    pub async fn accept(&self, config: &TlsConfig) -> Result<u32, TlsError> {
        let server_cert = self.cert_manager.get_server_cert()
            .ok_or_else(|| TlsError::InvalidCertificate("Server certificate not loaded".to_string()))?;
        let server_key = self.cert_manager.get_server_key()
            .ok_or_else(|| TlsError::InvalidKey("Server key not loaded".to_string()))?;

        let mut server_config = if config.verify_peer {
            ServerConfig::builder()
                .with_safe_defaults()
                .with_client_cert_verifier(Arc::new(rustls::server::AllowAnyAuthenticatedClient::new(
                    self.cert_manager.get_cert_store().clone()
                )))
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
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_trust_anchors(
            webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
                rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                    ta.subject,
                    ta.spki,
                    ta.name_constraints,
                )
            }),
        );

        let mut client_config = if config.verify_peer {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.add_trust_anchors(
                webpki_roots::TLS_SERVER_ROOTS.0
                    .iter()
                    .map(|ta| {
                        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                            ta.subject,
                            ta.spki,
                            ta.name_constraints,
                        )
                    }),
            );

            ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        } else {
            ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(AcceptAllVerifier))
                .with_no_client_auth()
        };

        client_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        
        self.conn_manager.connect(Arc::new(client_config), hostname, port).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub async fn close(&self, handle: u32) -> Result<(), TlsError> {
        self.conn_manager.close(handle).await
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

    pub async fn get_peer_certificate(&self, handle: u32) -> Result<Option<Vec<u8>>, TlsError> {
        self.conn_manager.get_peer_certificate(handle).await
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))
    }

    pub async fn get_protocol_version(&self, handle: u32) -> Result<TlsVersion, TlsError> {
        let version = self.conn_manager.get_protocol_version(handle).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))?;
        
        match version {
            Some(rustls::ProtocolVersion::TLSv1_2) => Ok(TlsVersion::Tls12),
            Some(rustls::ProtocolVersion::TLSv1_3) => Ok(TlsVersion::Tls13),
            _ => Err(TlsError::UnsupportedProtocol(format!("{:?}", version))),
        }
    }

    pub async fn get_cipher_suite(&self, handle: u32) -> Result<CipherSuite, TlsError> {
        let suite = self.conn_manager.get_cipher_suite(handle).await
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

    pub async fn set_listener(&self, listener: TcpListener) {
        self.conn_manager.set_listener(listener).await;
    }
}

impl Clone for TlsContext {
    fn clone(&self) -> Self {
        Self {
            cert_manager: self.cert_manager.clone(),
            conn_manager: self.conn_manager.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl Drop for TlsContext {
    fn drop(&mut self) {
        // Clean up any remaining connections
        // The ConnectionManager's Drop implementation will handle this
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_tls_context_creation() {
        let client_context = TlsContext::new();
        assert!(client_context.inner.lock().await.client_config.is_none());
        assert!(client_context.inner.lock().await.server_config.is_none());
    }

    #[tokio::test]
    async fn test_certificate_loading() {
        let mut context = TlsContext::new();
        context.load_certificate("test_data/server.crt").unwrap();
        context.load_private_key("test_data/server.key").unwrap();
        context.load_ca_certificates("test_data/ca.crt").unwrap();
    }

    #[tokio::test]
    async fn test_tls_connection() {
        let mut server_context = TlsContext::new();
        let mut client_context = TlsContext::new();

        // Load test certificates
        server_context.load_certificate("test_data/server.crt").unwrap();
        server_context.load_private_key("test_data/server.key").unwrap();
        // Skip loading CA certificates since we're not verifying peers
        // client_context.load_ca_certificates("test_data/ca.crt").unwrap();

        // Create TLS config with peer verification disabled
        let config = TlsConfig {
            version: TlsVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::TlsAes256GcmSha384,
                CipherSuite::TlsChaCha20Poly1305Sha256,
            ],
            verify_peer: false, // Disable peer verification for testing
        };

        // Start server
        let server_port = {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            server_context.conn_manager.set_listener(listener).await;
            port
        };

        // Server task
        let server_config = config.clone();
        let server_handle = tokio::spawn(async move {
            let conn = server_context.accept(&server_config).await.unwrap();
            let request = server_context.read(conn, 1024).await.unwrap();
            assert_eq!(request, b"Hello, server!");
            server_context.write(conn, b"Hello, client!").await.unwrap();
            server_context.close(conn).await.unwrap();
        });

        // Client task
        let client_config = config;
        let client_handle = tokio::spawn(async move {
            let conn = client_context.connect("127.0.0.1", server_port, &client_config).await.unwrap();
            client_context.write(conn, b"Hello, server!").await.unwrap();
            let response = client_context.read(conn, 1024).await.unwrap();
            assert_eq!(response, b"Hello, client!");
            client_context.close(conn).await.unwrap();
        });

        // Wait for both tasks to complete and handle any errors
        match tokio::try_join!(server_handle, client_handle) {
            Ok(_) => (),
            Err(e) => panic!("Test failed: {}", e),
        }
    }
} 