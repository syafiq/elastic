# ELASTIC TLS

A secure TLS implementation for the ELASTIC project, supporting multiple platforms including Linux, SEV-SNP, and WASM.

## Features

- TLS 1.2 and 1.3 support
- Hardware-accelerated cryptography (SEV-SNP)
- WASM support for browser environments
- Configurable cipher suites
- Certificate management
- Secure connection handling
- Automatic platform detection
- Comprehensive test suite

## Platform Support

| Platform | Status | Features |
|----------|--------|----------|
| Linux | ✅ | Full TLS 1.2/1.3 support, certificate management |
| SEV-SNP | ✅ | Hardware-accelerated cryptography, secure enclave support, attestation |
| WASM | ✅ | Browser-compatible TLS, WebSocket support, async/await |
| TDX | ❌ | Not implemented |

## Usage

### Basic TLS Client

```rust
use elastic_tls::{TlsContext, TlsConfig, TlsVersion, CipherSuite};

#[tokio::main]
async fn main() {
    let mut context = TlsContext::new();
    
    // Load certificates
    context.load_certificate("path/to/cert.crt").unwrap();
    context.load_private_key("path/to/key.key").unwrap();
    context.load_ca_certificates("path/to/ca.crt").unwrap();

    // Configure TLS
    let config = TlsConfig {
        version: TlsVersion::Tls13,
        cipher_suites: vec![
            CipherSuite::TlsAes256GcmSha384,
            CipherSuite::TlsChaCha20Poly1305Sha256,
        ],
        verify_peer: true,
    };

    // Connect to server
    let conn = context.connect("example.com", 443, &config).await.unwrap();
    
    // Send data
    context.write(conn, b"Hello, server!").await.unwrap();
    
    // Read response
    let response = context.read(conn, 1024).await.unwrap();
    
    // Close connection
    context.close(conn).unwrap();
}
```

### SEV-SNP Support

To use SEV-SNP hardware acceleration:

```rust
use elastic_tls::{TlsContext, TlsConfig};

#[tokio::main]
async fn main() {
    // Enable SEV-SNP
    std::env::set_var("ELASTIC_SEV_SNP", "1");
    
    let mut context = TlsContext::new();
    // ... rest of the code
}
```

The SEV-SNP implementation provides:
- Hardware-accelerated AES operations
- Secure memory handling
- SEV-SNP specific socket implementation
- Attestation support

### WASM Support

For browser environments:

```rust
use elastic_tls::{TlsContext, TlsConfig};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn connect_to_server() {
    let mut context = TlsContext::new();
    let config = TlsConfig::default();
    
    // Connect using WebSocket
    let conn = context.connect("wss://example.com", 443, &config).await.unwrap();
    // ... rest of the code
}
```

The WASM implementation provides:
- WebSocket fallback for browser environments
- WASM-specific error handling
- Proper async/await support
- WASM-specific memory management

## Building

### Standard Build

```bash
cargo build
```

### SEV-SNP Build

```bash
cargo build --features sevsnp
```

### WASM Build

```bash
cargo build --target wasm32-unknown-unknown
```

## Testing

Run the test suite:

```bash
cargo test
```

For SEV-SNP tests:

```bash
cargo test --features sevsnp
```

For WASM tests:

```bash
cargo test --target wasm32-unknown-unknown
```

## Security Considerations

- Always verify peer certificates in production
- Use strong cipher suites
- Keep certificates and private keys secure
- Enable SEV-SNP hardware acceleration when available
- Follow best practices for TLS configuration
- Use secure memory handling in SEV-SNP environments
- Implement proper attestation verification

## Examples

See the `examples` directory for:
- Simple client/server implementation
- WASM client implementation
- SEV-SNP specific examples

## License

This project is licensed under the MIT License - see the LICENSE file for details. 