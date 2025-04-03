use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use elastic_tls::{TlsContext, TlsVersion, CipherSuite};

#[tokio::test]
async fn test_tls_handshake() {
    // Create temporary directory for test certificates
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let cert_path = temp_dir.path().join("cert.pem");
    let key_path = temp_dir.path().join("key.pem");
    let ca_path = temp_dir.path().join("ca.pem");

    // Write test certificates
    File::create(&cert_path)
        .expect("Failed to create cert file")
        .write_all(include_bytes!("../certs/cert.pem"))
        .expect("Failed to write cert file");
    File::create(&key_path)
        .expect("Failed to create key file")
        .write_all(include_bytes!("../certs/key.pem"))
        .expect("Failed to write key file");
    File::create(&ca_path)
        .expect("Failed to create CA file")
        .write_all(include_bytes!("../certs/ca.pem"))
        .expect("Failed to write CA file");

    // Initialize server context
    let server_ctx = TlsContext::new();
    server_ctx.load_certificate(cert_path.to_str().unwrap())
        .expect("Failed to load server certificate");
    server_ctx.load_private_key(key_path.to_str().unwrap())
        .expect("Failed to load server private key");

    // Initialize client context
    let client_ctx = TlsContext::new();
    client_ctx.load_ca_certificates(ca_path.to_str().unwrap())
        .expect("Failed to load CA certificates");

    // Start server
    let server_ctx_clone = server_ctx.clone();
    tokio::spawn(async move {
        server_ctx_clone.bind(8080).await.expect("Failed to bind server");
        let _handle = server_ctx_clone.accept().await.expect("Failed to accept connection");
    });

    // Connect client
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let client_handle = client_ctx.connect("127.0.0.1", 8080).await
        .expect("Failed to connect client");

    // Verify connection
    assert!(client_ctx.get_peer_certificate(client_handle).unwrap().is_some());
}

#[tokio::test]
async fn test_tls_versions() {
    let ctx = TlsContext::new();
    let handle = ctx.connect("example.com", 443).await.expect("Failed to connect");
    let version = ctx.get_protocol_version(handle).expect("Failed to get protocol version");
    assert!(matches!(version, TlsVersion::Tls12 | TlsVersion::Tls13));
}

#[tokio::test]
async fn test_cipher_suites() {
    let ctx = TlsContext::new();
    let handle = ctx.connect("example.com", 443).await.expect("Failed to connect");
    let suite = ctx.get_cipher_suite(handle).expect("Failed to get cipher suite");
    assert!(matches!(suite, 
        CipherSuite::TlsAes256GcmSha384 | 
        CipherSuite::TlsChaCha20Poly1305Sha256
    ));
}

#[tokio::test]
async fn test_peer_certificate() {
    let ctx = TlsContext::new();
    let handle = ctx.connect("example.com", 443).await.expect("Failed to connect");
    let cert = ctx.get_peer_certificate(handle).expect("Failed to get peer certificate");
    assert!(cert.is_some());
}

#[tokio::test]
async fn test_invalid_handle() {
    let ctx = TlsContext::new();
    let invalid_handle = 12345;
    assert!(ctx.close(invalid_handle).is_err());
    assert!(ctx.get_peer_certificate(invalid_handle).is_err());
    assert!(ctx.get_protocol_version(invalid_handle).is_err());
    assert!(ctx.get_cipher_suite(invalid_handle).is_err());
}