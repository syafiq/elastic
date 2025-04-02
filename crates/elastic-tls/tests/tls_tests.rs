use elastic_tls::{TlsContext, TlsConfig};
use tokio;

#[tokio::test]
async fn test_tls_client_server() {
    println!("Starting TLS client-server test...");
    
    // Create server context
    println!("Creating server context...");
    let mut server_ctx = TlsContext::new();
    println!("Created server context");
    
    // Load server certificate and key
    println!("Loading server certificate and key...");
    println!("Current directory: \"{}\"", std::env::current_dir().unwrap().display());
    println!("Loading certificate from: tests/certs/server.crt");
    server_ctx.load_certificate("tests/certs/server.crt").expect("Failed to load server certificate");
    println!("Certificate loaded successfully");
    println!("Loading private key from: tests/certs/server.key");
    server_ctx.load_private_key("tests/certs/server.key").expect("Failed to load server key");
    println!("Private key loaded successfully");
    
    // Create client context
    println!("Creating client context...");
    let client_ctx = TlsContext::new();
    println!("Created client context");
    
    // Configure TLS
    let mut server_config = TlsConfig::default();
    server_config.verify_peer = false;
    let mut client_config = TlsConfig::default();
    client_config.verify_peer = false;
    
    // Bind server to port
    println!("Trying to bind to port 8080...");
    println!("Attempting to bind to port 8080");
    server_ctx.bind(8080).await.expect("Failed to bind server");
    println!("Successfully bound to port 8080");
    
    // Start server accept in background
    println!("Starting to accept connections...");
    let server_ctx_clone = server_ctx.clone();
    let server_handle = tokio::spawn(async move {
        println!("Waiting for incoming connection...");
        server_ctx_clone.accept(&server_config).await.expect("Failed to accept connection")
    });
    
    // Connect client
    println!("Connecting client to 127.0.0.1:8080...");
    let client_handle = client_ctx.connect("127.0.0.1", 8080, &client_config).await.expect("Failed to connect client");
    println!("Client connected successfully");
    
    // Wait for server to accept
    println!("Waiting for server to accept the connection...");
    let server_conn = server_handle.await.expect("Server task failed");
    
    // Send data from client to server
    let test_data = b"Hello, server!";
    client_ctx.write(client_handle, test_data).await.expect("Failed to write from client");
    
    // Read data on server
    let read_size = server_ctx.read(server_conn, 1024).await.expect("Failed to read on server");
    assert_eq!(&read_size[..test_data.len()], test_data);
    
    // Clean up
    client_ctx.close(client_handle).expect("Failed to close client connection");
    server_ctx.close(server_conn).expect("Failed to close server connection");
} 