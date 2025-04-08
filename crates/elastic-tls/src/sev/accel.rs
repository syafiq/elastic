use std::sync::Arc;
use tokio_rustls::rustls::{self, ClientConfig, ServerConfig};
use crate::{TlsError, TlsConfig};

pub struct SevAccelerator {
    use_hardware_acceleration: bool,
}

impl SevAccelerator {
    pub fn new() -> Self {
        Self {
            use_hardware_acceleration: Self::is_sevsnp_available(),
        }
    }

    fn is_sevsnp_available() -> bool {
        std::env::var("ELASTIC_SEV_SNP").is_ok()
    }

    pub fn use_hardware_acceleration(&self) -> bool {
        self.use_hardware_acceleration
    }

    pub fn configure_cipher_suites(&self, config: &mut rustls::ConfigBuilder<rustls::ClientConfig, rustls::WantsVerifier>) {
        if self.use_hardware_acceleration {
            // Prefer AES-GCM for hardware acceleration
            config.cipher_suites = vec![
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256,
            ];
        } else {
            config.cipher_suites = vec![
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
            ];
        }
    }

    pub fn configure_server_cipher_suites(&self, config: &mut rustls::ConfigBuilder<rustls::ServerConfig, rustls::WantsVerifier>) {
        if self.use_hardware_acceleration {
            // Prefer AES-GCM for hardware acceleration
            config.cipher_suites = vec![
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256,
            ];
        } else {
            config.cipher_suites = vec![
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
            ];
        }
    }
}

#[cfg(feature = "sevsnp")]
pub mod sev_bindings {
    use super::*;
    use std::sync::Arc;

    pub struct SevTlsSocket {
        accelerator: Arc<SevAccelerator>,
        // Add SEV-SNP specific fields here
    }

    impl SevTlsSocket {
        pub fn new(accelerator: Arc<SevAccelerator>) -> Self {
            Self {
                accelerator,
                // Initialize SEV-SNP specific fields
            }
        }

        pub fn use_hardware_acceleration(&self) -> bool {
            self.accelerator.use_hardware_acceleration()
        }
    }
} 