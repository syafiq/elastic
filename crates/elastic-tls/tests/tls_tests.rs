use std::net::SocketAddr;
use tokio::net::TcpListener;
use elastic_tls::{TlsContext, TlsConfig, TlsVersion, CipherSuite};
use std::path::PathBuf;

async fn ensure_test_certs() -> (PathBuf, PathBuf, PathBuf) {
    let cert_path = PathBuf::from("test_data/server.crt");
    let key_path = PathBuf::from("test_data/server.key");
    let ca_path = PathBuf::from("test_data/ca.crt");
    
    if !cert_path.exists() || !key_path.exists() || !ca_path.exists() {
        let status = std::process::Command::new("./scripts/generate_test_certs.sh")
            .current_dir("crates/elastic-tls")
            .status()
            .expect("Failed to run certificate generation script");
        assert!(status.success(), "Certificate generation failed");
    }
    
    (cert_path, key_path, ca_path)
}

#[tokio::test]
async fn test_tls_server_client() {
    let (cert_path, key_path, ca_path) = ensure_test_certs().await;

    // Create a TCP listener
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Create TLS contexts
    let mut server_context = TlsContext::new();
    let mut client_context = TlsContext::new();

    // Load certificates
    server_context.load_certificate(&format!("crates/elastic-tls/{}", cert_path.display())).unwrap();
    server_context.load_private_key(&format!("crates/elastic-tls/{}", key_path.display())).unwrap();
    
    // Load CA certificate for client verification
    client_context.load_ca_certificates(&format!("crates/elastic-tls/{}", ca_path.display())).unwrap();

    // Create TLS config
    let config = TlsConfig {
        version: TlsVersion::Tls13,
        cipher_suites: vec![
            CipherSuite::TlsAes256GcmSha384,
            CipherSuite::TlsChaCha20Poly1305Sha256,
        ],
        verify_peer: true,
    };

    // Start server
    let server_config = config.clone();
    let server_handle = tokio::spawn(async move {
        server_context.bind(addr.port()).await.unwrap();
        let conn = server_context.accept(&server_config).await.unwrap();
        
        // Read client message
        let received = server_context.read(conn, 1024).await.unwrap();
        assert_eq!(received, b"Hello, server!");

        // Send response
        server_context.write(conn, b"Hello, client!").await.unwrap();
        server_context.close(conn).await.unwrap();
    });

    // Start client
    let client_config = config.clone();
    let client_handle = tokio::spawn(async move {
        let conn = client_context.connect("127.0.0.1", addr.port(), &client_config).await.unwrap();
        
        // Send message
        client_context.write(conn, b"Hello, server!").await.unwrap();

        // Read response
        let received = client_context.read(conn, 1024).await.unwrap();
        assert_eq!(received, b"Hello, client!");

        client_context.close(conn).await.unwrap();
    });

    // Wait for both tasks to complete
    tokio::try_join!(server_handle, client_handle).unwrap();
}

#[tokio::test]
async fn test_tls_versions() {
    let (cert_path, key_path, ca_path) = ensure_test_certs().await;

    let mut context = TlsContext::new();
    context.load_certificate(&format!("crates/elastic-tls/{}", cert_path.display())).unwrap();
    context.load_private_key(&format!("crates/elastic-tls/{}", key_path.display())).unwrap();
    context.load_ca_certificates(&format!("crates/elastic-tls/{}", ca_path.display())).unwrap();

    let config = TlsConfig {
        version: TlsVersion::Tls12,
        cipher_suites: vec![CipherSuite::TlsAes256GcmSha384],
        verify_peer: true,
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut server_context = context.clone();
    let server_config = config.clone();
    let server_handle = tokio::spawn(async move {
        server_context.bind(addr.port()).await.unwrap();
        let conn = server_context.accept(&server_config).await.unwrap();
        let version = server_context.get_protocol_version(conn).await.unwrap();
        assert_eq!(version, TlsVersion::Tls12);
        server_context.close(conn).await.unwrap();
    });

    let client_context = context.clone();
    let client_config = config.clone();
    let client_handle = tokio::spawn(async move {
        let conn = client_context.connect("127.0.0.1", addr.port(), &client_config).await.unwrap();
        let version = client_context.get_protocol_version(conn).await.unwrap();
        assert_eq!(version, TlsVersion::Tls12);
        client_context.close(conn).await.unwrap();
    });

    let (server_result, client_result) = tokio::join!(server_handle, client_handle);
    server_result.unwrap();
    client_result.unwrap();
}

#[tokio::test]
async fn test_cipher_suites() {
    let (cert_path, key_path, ca_path) = ensure_test_certs().await;

    let mut context = TlsContext::new();
    context.load_certificate(&format!("crates/elastic-tls/{}", cert_path.display())).unwrap();
    context.load_private_key(&format!("crates/elastic-tls/{}", key_path.display())).unwrap();
    context.load_ca_certificates(&format!("crates/elastic-tls/{}", ca_path.display())).unwrap();

    let config = TlsConfig {
        version: TlsVersion::Tls13,
        cipher_suites: vec![CipherSuite::TlsChaCha20Poly1305Sha256],
        verify_peer: true,
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut server_context = context.clone();
    let server_config = config.clone();
    let server_handle = tokio::spawn(async move {
        server_context.bind(addr.port()).await.unwrap();
        let conn = server_context.accept(&server_config).await.unwrap();
        let suite = server_context.get_cipher_suite(conn).await.unwrap();
        assert_eq!(suite, CipherSuite::TlsChaCha20Poly1305Sha256);
        server_context.close(conn).await.unwrap();
    });

    let client_context = context.clone();
    let client_config = config.clone();
    let client_handle = tokio::spawn(async move {
        let conn = client_context.connect("127.0.0.1", addr.port(), &client_config).await.unwrap();
        let suite = client_context.get_cipher_suite(conn).await.unwrap();
        assert_eq!(suite, CipherSuite::TlsChaCha20Poly1305Sha256);
        client_context.close(conn).await.unwrap();
    });

    let (server_result, client_result) = tokio::join!(server_handle, client_handle);
    server_result.unwrap();
    client_result.unwrap();
}

#[cfg(feature = "sevsnp")]
#[tokio::test]
async fn test_sevsnp_acceleration() {
    let (cert_path, key_path) = ensure_test_certs().await;

    let mut context = TlsContext::new();
    context.load_certificate(cert_path.to_str().unwrap()).unwrap();
    context.load_private_key(key_path.to_str().unwrap()).unwrap();

    let config = TlsConfig {
        version: TlsVersion::Tls13,
        cipher_suites: vec![CipherSuite::TlsAes256GcmSha384],
        verify_peer: false,
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut server_context = context.clone();
    let server_config = config.clone();
    let server_handle = tokio::spawn(async move {
        server_context.bind(addr.port()).await.unwrap();
        let conn = server_context.accept(&server_config).await.unwrap();
        let suite = server_context.get_cipher_suite(conn).await.unwrap();
        assert_eq!(suite, CipherSuite::TlsAes256GcmSha384);
        server_context.close(conn).await.unwrap();
    });

    let client_context = context.clone();
    let client_config = config.clone();
    let client_handle = tokio::spawn(async move {
        let conn = client_context.connect("127.0.0.1", addr.port(), &client_config).await.unwrap();
        let suite = client_context.get_cipher_suite(conn).await.unwrap();
        assert_eq!(suite, CipherSuite::TlsAes256GcmSha384);
        client_context.close(conn).await.unwrap();
    });

    let (server_result, client_result) = tokio::join!(server_handle, client_handle);
    server_result.unwrap();
    client_result.unwrap();
}

#[cfg(feature = "sevsnp")]
#[tokio::test]
async fn test_sevsnp_hardware_acceleration() {
    let mut context = TlsContext::new();
    context.load_certificate("test_data/server.crt").unwrap();
    context.load_private_key("test_data/server.key").unwrap();

    let config = TlsConfig {
        version: TlsVersion::Tls13,
        cipher_suites: vec![CipherSuite::TlsAes256GcmSha384],
        verify_peer: false,
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut server_context = context.clone();
    let server_config = config.clone();
    let server_handle = tokio::spawn(async move {
        server_context.bind(addr.port()).await.unwrap();
        let conn = server_context.accept(&server_config).await.unwrap();
        
        // Test data transfer with hardware acceleration
        let data = vec![0u8; 1024 * 1024]; // 1MB of data
        server_context.write(conn, &data).await.unwrap();
        
        let received = server_context.read(conn, 1024 * 1024).await.unwrap();
        assert_eq!(received, data);
        
        server_context.close(conn).await.unwrap();
    });

    let client_context = context.clone();
    let client_config = config;
    let client_handle = tokio::spawn(async move {
        let conn = client_context.connect("127.0.0.1", addr.port(), &client_config).await.unwrap();
        
        // Test data transfer with hardware acceleration
        let data = vec![0u8; 1024 * 1024]; // 1MB of data
        client_context.write(conn, &data).await.unwrap();
        
        let received = client_context.read(conn, 1024 * 1024).await.unwrap();
        assert_eq!(received, data);
        
        client_context.close(conn).await.unwrap();
    });

    match tokio::try_join!(server_handle, client_handle) {
        Ok(_) => (),
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_wasm_tls() {
    let mut context = TlsContext::new();
    context.load_certificate("test_data/server.crt").unwrap();
    context.load_private_key("test_data/server.key").unwrap();

    let config = TlsConfig {
        version: TlsVersion::Tls13,
        cipher_suites: vec![CipherSuite::TlsChaCha20Poly1305Sha256],
        verify_peer: false,
    };

    let socket = WasmTlsSocket::new("wss://localhost:8443", false).unwrap();
    
    // Wait for connection
    while !socket.is_connected() {
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                100,
            ).unwrap();
        })).await.unwrap();
    }

    // Test sending data
    let data = vec![0u8; 1024];
    socket.send(&data).unwrap();

    // Test receiving data
    let received = socket.pop_message().unwrap();
    assert_eq!(received, data);

    // Test message queue
    assert_eq!(socket.get_message_queue_length(), 0);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_wasm_tls_error_handling() {
    // Test invalid URL
    let result = WasmTlsSocket::new("invalid://url", false);
    assert!(result.is_err());

    // Test connection failure
    let socket = WasmTlsSocket::new("wss://nonexistent:8443", false).unwrap();
    assert!(!socket.is_connected());

    // Test sending when not connected
    let result = socket.send(&[0u8; 1024]);
    assert!(result.is_err());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_wasm_tls_sevsnp() {
    let socket = WasmTlsSocket::new("wss://localhost:8443", true).unwrap();
    
    // Wait for connection
    while !socket.is_connected() {
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                100,
            ).unwrap();
        })).await.unwrap();
    }

    // Test sending data with SEV-SNP
    let data = vec![0u8; 1024 * 1024]; // 1MB of data
    socket.send(&data).unwrap();

    // Test receiving data with SEV-SNP
    let received = socket.pop_message().unwrap();
    assert_eq!(received, data);
} 