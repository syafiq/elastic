# ELASTIC (Efficient, Lightweight And Secure orchesTration for reliable servICes)

This repository contains the implementation of secure interfaces for the ELASTIC project, focusing on providing secure and efficient operations in WebAssembly environments.

## Project Structure

The project is organized into several crates, each providing specific functionality with support for different platforms:

```
crates/
├── elastic-clock/      # Secure time operations
│   ├── src/
│   │   ├── linux/     # Linux-specific implementation
│   │   └── sev/       # SEV-SNP specific implementation
│   └── tests/
│       ├── linux/     # Linux-specific tests
│       └── sev/       # SEV-SNP specific tests
├── elastic-crypto/     # Cryptographic operations
│   ├── src/
│   │   ├── linux/     # Linux-specific implementation
│   │   ├── sev/       # SEV-SNP specific implementation
│   │   └── wasm.rs    # WASM-specific implementation
│   └── tests/
│       ├── linux/     # Linux-specific tests
│       └── sev/       # SEV-SNP specific tests
├── elastic-file/       # Secure file operations
│   ├── src/
│   │   ├── linux/     # Linux-specific implementation
│   │   └── sev/       # SEV-SNP specific implementation (in progress)
│   └── tests/
│       ├── linux/     # Linux-specific tests
│       └── sev/       # SEV-SNP specific tests
└── elastic-tls/        # Secure communication
    ├── src/
    │   ├── linux/     # Linux-specific implementation (planned)
    │   └── sev/       # SEV-SNP specific implementation (planned)
    └── tests/
        ├── linux/     # Linux-specific tests
        └── sev/       # SEV-SNP specific tests
```

Each crate follows a similar structure:
- `src/`: Contains the implementation code
  - `linux/`: Linux-specific implementation
  - `sev/`: SEV-SNP specific implementation
- `tests/`: Contains test code
  - `linux/`: Linux-specific tests
  - `sev/`: SEV-SNP specific tests
- `wit/`: Contains WebAssembly Interface Types definitions

## Features

### Clock Interface (`elastic-clock`)
- Current time reading in seconds since UNIX epoch
- Timezone information retrieval
- Monotonic clock for precise time measurements
- Error handling for system time and timezone operations
- WebAssembly Interface Types (WIT) support for language interoperability
- SEV-SNP environment detection and automatic clock mechanism selection
- WASM example demonstrating SEV-SNP detection and clock usage

### File Interface (`elastic-file`)
- Secure file operations with container-based isolation
- Support for both regular and encrypted file storage
- File metadata access and manipulation
- Directory listing and file management
- AES-GCM encryption for secure storage
- WebAssembly Interface Types (WIT) support for language interoperability

### Crypto Interface (`elastic-crypto`)
- Symmetric encryption/decryption (AES-256-GCM)
  - Linux: Software implementation using `aes-gcm` crate
  - SEV-SNP: Hardware-accelerated implementation using SEV firmware
  - WASM: Secure software implementation with SEV-SNP environment detection
- Random number generation
  - Linux: System RNG using `rand` crate
  - SEV-SNP: Hardware RNG with timestamp-based entropy
  - WASM: Secure RNG with SEV-SNP environment detection
- Key management and context handling
- WebAssembly Interface Types (WIT) support for language interoperability
- Environment-based SEV-SNP detection for WASM environments

### TLS Interface (`elastic-tls`)
- Secure communication using TLS 1.2 and 1.3
- Support for multiple cipher suites (AES-128-GCM, AES-256-GCM, ChaCha20-Poly1305)
- Certificate and key management
- Client and server connection handling
- Secure data transfer with automatic encryption/decryption
- Connection information and peer certificate verification
- WebAssembly Interface Types (WIT) support for language interoperability

## Platform Support

| Interface | Linux | SEV-SNP | TDX | Notes |
|-----------|-------|---------|-----|-------|
| Clock     | ✅    | ✅      | ❌  | SEV-SNP uses TSC, Linux uses system calls |
| File      | ✅    | ⏳      | ❌  | SEV-SNP implementation in progress |
| Crypto    | ✅    | ✅      | ❌  | SEV-SNP uses hardware-accelerated RNG and AES |
| TLS       | ⏳    | ⏳      | ❌  | Planning phase for both platforms |

Legend:
- ✅: Implemented and tested
- ⏳: In progress or planned
- ❌: Not implemented

## Getting Started

### Prerequisites
- Rust (latest stable version)
- WebAssembly tools (wasm-bindgen, wasm-pack)
- OpenSSL or equivalent TLS library
- For SEV-SNP: AMD SEV-SNP capable hardware and firmware

### Building
```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p elastic-tls

# Build WASM example with SEV-SNP support
cargo build -p wasm-crypto-example --target wasm32-wasip1 --features elastic-crypto/sevsnp
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p elastic-tls

# Test WASM example on SEV-SNP machine
wasmtime --env ELASTIC_SEV_SNP=1 target/wasm32-wasip1/debug/wasm-crypto-example.wasm

# Test WASM example on standard Linux
wasmtime target/wasm32-wasip1/debug/wasm-crypto-example.wasm
```

## WIT Interface Support

Each crate provides a WebAssembly Interface Types (WIT) definition for language interoperability. The interfaces are located in the `wit/` directory of each crate.

### Clock Interface (`clock.wit`)
```wit
package elastic:clock@0.1.0;

interface types {
    variant clock-error {
        system-time-error(string),
    }
}

interface clock {
    use types.{clock-error};

    read-current-time: func() -> result<u64, clock-error>;
    read-timezone: func() -> result<string, clock-error>;
    start-monotonic: func() -> result<_, clock-error>;
    stop-monotonic: func() -> result<_, clock-error>;
    read-monotonic: func() -> result<u64, clock-error>;
}

world clock-impl {
    export clock;
}
```

### File Interface (`file.wit`)
```wit
package elastic:file@0.1.0;

interface types {
    variant file-error {
        not-found(string),
        permission-denied(string),
        already-exists(string),
        invalid-operation(string),
        encryption-error(string),
        decryption-error(string),
        io-error(string),
    }

    enum file-mode {
        read,
        write,
        append,
        read-write,
    }

    enum file-type {
        regular,
        directory,
        symbolic-link,
    }

    record file-metadata {
        name: string,
        size: u64,
        file-type: file-type,
        created: u64,
        modified: u64,
    }
}

interface file {
    use types.{file-error, file-mode, file-type, file-metadata};

    open: func(path: string, mode: file-mode) -> result<u32, file-error>;
    close: func(handle: u32) -> result<_, file-error>;
    read: func(handle: u32, size: u32) -> result<list<u8>, file-error>;
    write: func(handle: u32, data: list<u8>) -> result<_, file-error>;
    seek: func(handle: u32, offset: i64, whence: u32) -> result<u64, file-error>;
    tell: func(handle: u32) -> result<u64, file-error>;
    stat: func(path: string) -> result<file-metadata, file-error>;
    mkdir: func(path: string) -> result<_, file-error>;
    rmdir: func(path: string) -> result<_, file-error>;
    unlink: func(path: string) -> result<_, file-error>;
    readdir: func(path: string) -> result<list<string>, file-error>;
}

world file-impl {
    export file;
}
```

### Crypto Interface (`crypto.wit`)
```wit
package elastic:crypto@0.1.0;

interface types {
    variant crypto-error {
        invalid-key(string),
        invalid-iv(string),
        encryption-failed(string),
        decryption-failed(string),
        signing-failed(string),
        verification-failed(string),
        hashing-failed(string),
        hmac-failed(string),
    }

    enum hash-algorithm {
        sha256,
        sha512,
    }

    enum cipher-algorithm {
        aes256gcm,
        chacha20poly1305,
    }

    enum signature-algorithm {
        rsa2048,
        ecdsa256,
    }
}

interface crypto {
    use types.{crypto-error, hash-algorithm, cipher-algorithm, signature-algorithm};

    create-context: func() -> result<u32, crypto-error>;
    destroy-context: func(handle: u32) -> result<_, crypto-error>;

    generate-key: func(handle: u32, algorithm: cipher-algorithm) -> result<list<u8>, crypto-error>;
    generate-iv: func(handle: u32, algorithm: cipher-algorithm) -> result<list<u8>, crypto-error>;

    encrypt: func(handle: u32, data: list<u8>, key: list<u8>, iv: list<u8>) -> result<list<u8>, crypto-error>;
    decrypt: func(handle: u32, data: list<u8>, key: list<u8>, iv: list<u8>) -> result<list<u8>, crypto-error>;

    sign: func(handle: u32, data: list<u8>, key: list<u8>, algorithm: signature-algorithm) -> result<list<u8>, crypto-error>;
    verify: func(handle: u32, data: list<u8>, signature: list<u8>, key: list<u8>, algorithm: signature-algorithm) -> result<bool, crypto-error>;

    hash: func(handle: u32, data: list<u8>, algorithm: hash-algorithm) -> result<list<u8>, crypto-error>;
    hmac: func(handle: u32, data: list<u8>, key: list<u8>, algorithm: hash-algorithm) -> result<list<u8>, crypto-error>;
}

world crypto-impl {
    export crypto;
}
```

### TLS Interface (`tls.wit`)
```wit
package elastic:tls@0.1.0;

interface types {
    variant tls-error {
        invalid-certificate(string),
        invalid-key(string),
        connection-failed(string),
        handshake-failed(string),
        write-failed(string),
        read-failed(string),
        unsupported-protocol(string),
        unsupported-cipher(string),
    }

    enum tls-version {
        tls-1-2,
        tls-1-3,
    }

    enum cipher-suite {
        tls-aes-128-gcm-sha256,
        tls-aes-256-gcm-sha384,
        tls-chacha20-poly1305-sha256,
    }

    record tls-config {
        version: tls-version,
        cipher-suites: list<cipher-suite>,
        verify-peer: bool,
        verify-hostname: bool,
    }
}

interface tls {
    use types.{tls-error, tls-version, cipher-suite, tls-config};

    create-context: func(config: tls-config) -> result<u32, tls-error>;
    destroy-context: func(handle: u32) -> result<_, tls-error>;

    load-certificate: func(handle: u32, cert: list<u8>) -> result<_, tls-error>;
    load-private-key: func(handle: u32, key: list<u8>) -> result<_, tls-error>;
    load-ca-certificates: func(handle: u32, certs: list<u8>) -> result<_, tls-error>;

    connect: func(handle: u32, hostname: string, port: u16) -> result<u32, tls-error>;
    accept: func(handle: u32, port: u16) -> result<u32, tls-error>;
    close: func(conn: u32) -> result<_, tls-error>;

    write: func(conn: u32, data: list<u8>) -> result<u32, tls-error>;
    read: func(conn: u32, max-size: u32) -> result<list<u8>, tls-error>;

    get-peer-certificate: func(conn: u32) -> result<list<u8>, tls-error>;
    get-protocol-version: func(conn: u32) -> result<tls-version, tls-error>;
    get-cipher-suite: func(conn: u32) -> result<cipher-suite, tls-error>;
}

world tls-impl {
    export tls;
}
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
