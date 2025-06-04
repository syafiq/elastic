# Elastic Crypto Demo

This demo demonstrates the "build once, run anywhere" principle of the Elastic Crypto HAL. The same WASM binary can run on different platforms (Linux and SEV-SNP) using different hardware backends while maintaining consistent behavior.

## WIT Interface

The demo uses this WIT interface (`crates/elastic-crypto/wit/crypto.wit`):

```wit
interface crypto {
  // Key management
  generate-key: func(config: key-config) -> result<u32, error>;
  import-key: func(key-data: list<u8>, config: key-config) -> result<u32, error>;
  export-key: func(handle: u32) -> result<list<u8>, error>;
  delete-key: func(handle: u32) -> result<unit, error>;

  // Crypto operations
  encrypt: func(handle: u32, data: list<u8>) -> result<list<u8>, error>;
  decrypt: func(handle: u32, data: list<u8>) -> result<list<u8>, error>;
  hash: func(data: list<u8>) -> result<list<u8>, error>;
}

record key-config {
  key-type: key-type;
  key-size: u32;
  secure-storage: bool;
}

enum key-type {
  symmetric,
  asymmetric,
  hmac,
}

enum error {
  invalid-key-length,
  encryption-error,
  decryption-error,
  unsupported-operation,
  key-not-found,
  operation-not-permitted,
}
```

## Cross-Platform Workflow

### 1. Build on Local Machine

```bash
# Build the WASM binary
cargo build --target wasm32-wasip1
```

The resulting binary will be at `target/wasm32-wasip1/debug/crypto-demo.wasm`

### 2. Run on Different Platforms

#### On Linux:
```bash
wasmtime ../../target/wasm32-wasip1/debug/crypto-demo.wasm
```

#### On AWS SEV-SNP:
```bash
# Enable SEV-SNP mode
export ELASTIC_SEV_SNP=1
# Mount /dev for SEV-SNP device access
wasmtime --dir /dev ../../target/wasm32-wasip1/debug/crypto-demo.wasm
```

## What to Expect

The demo will:
1. Generate an AES-256 key
2. Encrypt and decrypt a test message
3. Verify the decrypted data matches the original
4. Print the results in a consistent format

The same binary will use different hardware backends:
- On Linux: Uses the Linux crypto backend
- On SEV-SNP: Uses the SEV-SNP hardware crypto backend

## Key Points

- The same WASM binary works on both platforms
- The WIT interface ensures consistent behavior
- Encryption results are consistent across platforms
- No platform-specific code in the demo 