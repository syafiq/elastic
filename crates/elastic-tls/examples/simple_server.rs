use elastic_tls::{TlsContext, TlsConfig, TlsVersion, CipherSuite};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut context = TlsContext::new();

    // Load certificates
    context.load_certificate("server.crt")?;
    context.load_private_key("server.key")?;
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

    // Bind to port
    let port = 8443;
    context.bind(port).await?;
    println!("Listening on port {}", port);

    // Accept connections
    loop {
        let conn = context.accept(&config).await?;
        println!("Accepted new connection");

        // Get connection info
        let version = context.get_protocol_version(conn).await?;
        println!("Protocol version: {:?}", version);

        let suite = context.get_cipher_suite(conn).await?;
        println!("Cipher suite: {:?}", suite);

        // Get peer certificate
        if let Some(cert) = context.get_peer_certificate(conn).await? {
            println!("Client certificate: {} bytes", cert.len());
        } else {
            println!("No client certificate");
        }

        // Read request
        let request = context.read(conn, 1024).await?;
        println!("Received request: {:?}", String::from_utf8_lossy(&request));

        // Send response
        let response = b"Hello, client!";
        context.write(conn, response).await?;
        println!("Sent response: {:?}", String::from_utf8_lossy(response));

        // Close connection
        context.close(conn).await?;
        println!("Connection closed");
    }
} 