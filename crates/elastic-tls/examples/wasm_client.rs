use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use elastic_tls::{TlsContext, TlsConfig, TlsVersion, CipherSuite};
use log::{info, error};

#[wasm_bindgen]
pub fn connect_to_server() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    spawn_local(async move {
        let mut context = TlsContext::new();
        
        // Load certificates
        if let Err(e) = context.load_certificate("client.crt") {
            error!("Failed to load certificate: {}", e);
            return;
        }
        if let Err(e) = context.load_private_key("client.key") {
            error!("Failed to load private key: {}", e);
            return;
        }
        if let Err(e) = context.load_ca_certificates("ca.crt") {
            error!("Failed to load CA certificates: {}", e);
            return;
        }

        // Create TLS configuration
        let config = TlsConfig {
            version: TlsVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::TlsAes256GcmSha384,
                CipherSuite::TlsChaCha20Poly1305Sha256,
            ],
            verify_peer: true,
        };

        // Connect to server
        let conn = match context.connect("localhost", 8443, &config).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Connection failed: {}", e);
                return;
            }
        };
        
        info!("Connected to server");

        // Send message
        if let Err(e) = context.write(conn, b"Hello, server!").await {
            error!("Write failed: {}", e);
            return;
        }
        
        info!("Sent message to server");

        // Read response
        let response = match context.read(conn, 1024).await {
            Ok(response) => response,
            Err(e) => {
                error!("Read failed: {}", e);
                return;
            }
        };
        
        info!("Received response: {:?}", String::from_utf8_lossy(&response));

        // Get peer certificate
        match context.get_peer_certificate(conn).await {
            Ok(Some(cert)) => {
                info!("Server certificate: {} bytes", cert.len());
            }
            Ok(None) => {
                info!("No server certificate");
            }
            Err(e) => {
                error!("Failed to get peer certificate: {}", e);
            }
        }

        // Close connection
        if let Err(e) = context.close(conn).await {
            error!("Failed to close connection: {}", e);
            return;
        }
        
        info!("Connection closed");
    });
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();
    Ok(())
} 