use std::sync::Arc;
use tokio_rustls::rustls::{self, ClientConfig, ServerConfig};
use crate::{TlsError, TlsConfig};

mod accel;
pub use accel::{SevAccelerator, sev_bindings};

pub struct SevTlsContext {
    accelerator: Arc<SevAccelerator>,
}

impl SevTlsContext {
    pub fn new() -> Self {
        Self {
            accelerator: Arc::new(SevAccelerator::new()),
        }
    }

    pub fn create_client_config(&self, config: &TlsConfig) -> Result<Arc<ClientConfig>, TlsError> {
        let mut client_config = if config.verify_peer {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.add_trust_anchors(
                webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
                    rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                })
            );

            ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        } else {
            ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(crate::AcceptAllVerifier))
                .with_no_client_auth()
        };

        self.accelerator.configure_cipher_suites(&mut client_config);
        client_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(Arc::new(client_config))
    }

    pub fn create_server_config(
        &self,
        config: &TlsConfig,
        cert_chain: Vec<rustls::Certificate>,
        key: rustls::PrivateKey,
    ) -> Result<Arc<ServerConfig>, TlsError> {
        let mut server_config = if config.verify_peer {
            ServerConfig::builder()
                .with_safe_defaults()
                .with_client_cert_verifier(Arc::new(rustls::server::AllowAnyAuthenticatedClient::new(
                    rustls::RootCertStore::empty(),
                )))
                .with_single_cert(cert_chain, key)
                .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?
        } else {
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(cert_chain, key)
                .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?
        };

        self.accelerator.configure_server_cipher_suites(&mut server_config);
        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(Arc::new(server_config))
    }
}

#[cfg(feature = "sevsnp")]
mod sev_bindings {
    use std::sync::Arc;
    use tokio_rustls::rustls;
    use super::*;

    pub struct SevTlsSocket {
        context: Arc<SevTlsContext>,
        // Add SEV-SNP specific fields here
    }

    impl SevTlsSocket {
        pub fn new(context: Arc<SevTlsContext>) -> Self {
            Self {
                context,
                // Initialize SEV-SNP specific fields
            }
        }

        pub fn use_hardware_acceleration(&self) -> bool {
            self.context.accelerator.use_hardware_acceleration
        }
    }
} 