use thiserror::Error;

#[derive(Error, Debug)]
pub enum TlsError {
    #[error("Certificate error: {0}")]
    CertificateError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherSuite {
    TlsAes128GcmSha256,
    TlsAes256GcmSha384,
    TlsChaCha20Poly1305Sha256,
}

pub struct TlsConfig {
    pub verify_peer: bool,
    pub min_version: TlsVersion,
    pub max_version: TlsVersion,
    pub cipher_suites: Vec<CipherSuite>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify_peer: true,
            min_version: TlsVersion::Tls13,
            max_version: TlsVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::TlsAes256GcmSha384,
                CipherSuite::TlsChaCha20Poly1305Sha256,
            ],
        }
    }
}

pub trait TlsOperations {
    fn load_certificate(&mut self, cert_path: &str) -> Result<(), TlsError>;
    fn load_private_key(&mut self, key_path: &str) -> Result<(), TlsError>;
    fn load_ca_certificates(&mut self, ca_path: &str) -> Result<(), TlsError>;
    fn bind(&self, port: u16) -> Result<(), TlsError>;
    fn accept(&self, config: &TlsConfig) -> Result<u32, TlsError>;
    fn connect(&self, hostname: &str, port: u16, config: &TlsConfig) -> Result<u32, TlsError>;
    fn close(&self, handle: u32) -> Result<(), TlsError>;
    fn write(&self, handle: u32, data: &[u8]) -> Result<(), TlsError>;
    fn read(&self, handle: u32, max_size: usize) -> Result<Vec<u8>, TlsError>;
    fn get_peer_certificate(&self, handle: u32) -> Result<Option<Vec<u8>>, TlsError>;
    fn get_protocol_version(&self, handle: u32) -> Result<TlsVersion, TlsError>;
    fn get_cipher_suite(&self, handle: u32) -> Result<CipherSuite, TlsError>;
} 