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

This interface is implemented differently on each platform:

### Linux Implementation
```rust
// crates/elastic-crypto/src/linux.rs
impl Crypto for LinuxCrypto {
    fn encrypt(&self, handle: u32, data: Vec<u8>) -> Result<Vec<u8>> {
        // Uses Linux crypto backend (e.g., /dev/crypto)
        let cipher = aes_gcm::Aes256Gcm::new_from_slice(&key.data)?;
        let nonce = aes_gcm::Nonce::from_slice(b"elastic-nc12");
        cipher.encrypt(nonce, data.as_ref())
    }
}
```

### SEV-SNP Implementation
```rust
// crates/elastic-crypto/src/sev/mod.rs
impl Crypto for SevsnpCrypto {
    fn encrypt(&self, handle: u32, data: Vec<u8>) -> Result<Vec<u8>> {
        // Uses SEV-SNP hardware crypto
        if let Some(aes) = self.aes.as_mut() {
            aes.encrypt(data)
        } else {
            Err(Error::SevsnpNotAvailable)
        }
    }
}
```

## Complete Cross-Platform Workflow

### 1. Build on Local Machine
```bash
# On your local machine
cargo build --target wasm32-wasip1
```

The resulting binary will be at `target/wasm32-wasip1/debug/crypto-demo.wasm`

### 2. Copy Binary to SEV-SNP
```bash
# Copy the binary to your SEV-SNP machine
scp target/wasm32-wasip1/debug/crypto-demo.wasm user@sev-snp-machine:~/demo/
```

### 3. Run on Different Platforms

#### On Linux:
```bash
# On your local machine
wasmtime ../../target/wasm32-wasip1/debug/crypto-demo.wasm
```

#### On AWS SEV-SNP:
```bash
# On the SEV-SNP machine
# Note: Environment variables must be passed explicitly to wasmtime
wasmtime --env ELASTIC_SEV_SNP=1 --dir /dev ~/demo/crypto-demo.wasm
```

## Platform-Specific Implementations

The same WIT interface is implemented differently on each platform:

### Linux Implementation
- Uses the Linux crypto backend
- No special hardware requirements
- Implementation in `crates/elastic-crypto/src/linux.rs`

### SEV-SNP Implementation
- Uses SEV-SNP hardware crypto
- Requires `/dev/sev-guest`
- Implementation in `crates/elastic-crypto/src/sev/mod.rs`

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

- The same WASM binary works on both platforms without recompilation
- The WIT interface ensures consistent behavior across platforms
- Encryption results are consistent across platforms
- No platform-specific code in the demo
- Different hardware backends are used transparently 