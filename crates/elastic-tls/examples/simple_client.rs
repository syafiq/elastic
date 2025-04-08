use elastic_tls::{TlsContext, TlsConfig, TlsVersion, CipherSuite};
use std::env;
use std::time::Duration;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Enable logging
    env_logger::init();

    // Check if SEV-SNP should be enabled
    if env::var("ELASTIC_SEV_SNP").is_ok() {
        println!("SEV-SNP hardware acceleration enabled");
    }

    let mut context = TlsContext::new();

    // Load CA certificates
    context.load_ca_certificates("ca.crt")?;

    // Create TLS configuration
    let config = TlsConfig {
        version: TlsVersion::Tls13,
        cipher_suites: vec![
            CipherSuite::TlsAes256GcmSha384,
            CipherSuite::TlsChaCha20Poly1305Sha256,
        ],
        verify_peer: true,
    };

    println!("Connecting to server...");

    // Connect to server
    let conn = context.connect("localhost", 8443, &config).await?;
    println!("Connected to server");

    // Send data
    let data = b"Hello, server!";
    context.write(conn, data).await?;
    println!("Sent data: {:?}", String::from_utf8_lossy(data));

    // Read response
    let response = context.read(conn, 1024).await?;
    println!("Received response: {:?}", String::from_utf8_lossy(&response));

    // Get connection info
    let version = context.get_protocol_version(conn).await?;
    println!("Protocol version: {:?}", version);

    let suite = context.get_cipher_suite(conn).await?;
    println!("Cipher suite: {:?}", suite);

    // Get peer certificate
    if let Some(cert) = context.get_peer_certificate(conn).await? {
        println!("Peer certificate: {} bytes", cert.len());
    } else {
        println!("No peer certificate");
    }

    // Close connection
    context.close(conn).await?;
    println!("Connection closed");

    Ok(())
} 