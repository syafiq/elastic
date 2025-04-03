mod cert;
mod conn;

use std::sync::{Arc, Mutex};
use tokio_rustls::rustls::{self, client::{ServerCertVerifier, ServerCertVerified}, Error};
use crate::common::{TlsError, TlsVersion, CipherSuite};
use self::cert::CertificateManager;
use self::conn::ConnectionManager;

#[derive(Clone)]
pub struct TlsContext {
    cert_manager: Arc<Mutex<CertificateManager>>,
    conn_manager: Arc<ConnectionManager>,
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

impl TlsContext {
    pub fn new() -> Self {
        Self {
            cert_manager: Arc::new(Mutex::new(CertificateManager::new())),
            conn_manager: Arc::new(ConnectionManager::new()),
        }
    }

    pub fn load_certificate(&self, cert_path: &str) -> Result<(), TlsError> {
        let mut cert_manager = self.cert_manager.lock()
            .map_err(|_| TlsError::InvalidCertificate("Failed to lock certificate manager".to_string()))?;
        cert_manager.load_certificate(cert_path)
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))
    }

    pub fn load_private_key(&self, key_path: &str) -> Result<(), TlsError> {
        let mut cert_manager = self.cert_manager.lock()
            .map_err(|_| TlsError::InvalidKey("Failed to lock certificate manager".to_string()))?;
        cert_manager.load_private_key(key_path)
            .map_err(|e| TlsError::InvalidKey(e.to_string()))
    }

    pub fn load_ca_certificates(&self, ca_path: &str) -> Result<(), TlsError> {
        let mut cert_manager = self.cert_manager.lock()
            .map_err(|_| TlsError::InvalidCertificate("Failed to lock certificate manager".to_string()))?;
        cert_manager.load_ca_certificates(ca_path)
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))
    }

    pub async fn bind(&self, port: u16) -> Result<(), TlsError> {
        let addr = format!("127.0.0.1:{}", port);
        self.conn_manager.bind(&addr).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub async fn accept(&self) -> Result<u32, TlsError> {
        let config = self.create_server_config()?;
        self.conn_manager.accept(config).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub async fn connect(&self, hostname: &str, port: u16) -> Result<u32, TlsError> {
        let config = self.create_client_config()?;
        let addr = format!("{}:{}", hostname, port);
        self.conn_manager.connect(&addr, config).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub async fn close(&self, handle: u32) -> Result<(), TlsError> {
        self.conn_manager.close(handle).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))
    }

    pub async fn write(&self, handle: u32, data: &[u8]) -> Result<(), TlsError> {
        self.conn_manager.write(handle, data).await
            .map_err(|e| TlsError::WriteFailed(e.to_string()))?;
        Ok(())
    }

    pub async fn read(&self, handle: u32, max_size: usize) -> Result<Vec<u8>, TlsError> {
        let mut buf = vec![0; max_size];
        let n = self.conn_manager.read(handle, &mut buf).await
            .map_err(|e| TlsError::ReadFailed(e.to_string()))?;
        buf.truncate(n);
        Ok(buf)
    }

    pub async fn get_peer_certificate(&self, handle: u32) -> Result<Option<Vec<u8>>, TlsError> {
        let cert = self.conn_manager.get_peer_certificate(handle).await
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?;
        Ok(Some(cert))
    }

    pub async fn get_protocol_version(&self, handle: u32) -> Result<TlsVersion, TlsError> {
        let version = self.conn_manager.get_protocol_version(handle).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))?;
        Ok(match version.as_str() {
            "TLSv1.2" => TlsVersion::Tls12,
            "TLSv1.3" => TlsVersion::Tls13,
            _ => return Err(TlsError::UnsupportedProtocol("Unknown protocol version".to_string())),
        })
    }

    pub async fn get_cipher_suite(&self, handle: u32) -> Result<CipherSuite, TlsError> {
        let suite = self.conn_manager.get_cipher_suite(handle).await
            .map_err(|e| TlsError::ConnectionFailed(e.to_string()))?;
        Ok(match suite.as_str() {
            "TLS13_AES_256_GCM_SHA384" => CipherSuite::TlsAes256GcmSha384,
            "TLS13_CHACHA20_POLY1305_SHA256" => CipherSuite::TlsChaCha20Poly1305Sha256,
            _ => return Err(TlsError::UnsupportedCipher("Unsupported cipher suite".to_string())),
        })
    }

    fn create_server_config(&self) -> Result<Arc<rustls::ServerConfig>, TlsError> {
        let cert_manager = self.cert_manager.lock()
            .map_err(|_| TlsError::InvalidCertificate("Failed to lock certificate manager".to_string()))?;
        let certs = cert_manager.get_server_cert()
            .ok_or_else(|| TlsError::InvalidCertificate("No server certificate loaded".to_string()))?;
        let key = cert_manager.get_server_key()
            .ok_or_else(|| TlsError::InvalidKey("No server key loaded".to_string()))?;
        
        let mut config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![certs], key)
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?;
        
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(Arc::new(config))
    }

    fn create_client_config(&self) -> Result<Arc<rustls::ClientConfig>, TlsError> {
        let cert_manager = self.cert_manager.lock()
            .map_err(|_| TlsError::InvalidCertificate("Failed to lock certificate manager".to_string()))?;
        let root_store = cert_manager.get_cert_store();
        let mut config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(Arc::new(config))
    }
}

impl Default for TlsContext {
    fn default() -> Self {
        Self::new()
    }
} 