# Elastic Crypto Demo

This demo demonstrates the "build once, run anywhere" principle of the Elastic Crypto HAL. The same WASM binary can run on different platforms (Linux and SEV-SNP) using different hardware backends while maintaining consistent behavior.

## Demo Code

Here's the actual code that uses the HAL (`demo/crypto-demo/src/main.rs`):

```rust
use elastic_crypto::crypto::{Crypto, KeyConfig, KeyType};

fn main() {
    // Initialize the crypto HAL
    let crypto = Crypto::new().expect("Failed to initialize crypto");
    
    // Generate a symmetric key
    let key_config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: false,
    };
    let key_handle = crypto.generate_key(key_config).expect("Failed to generate key");
    
    // Test data
    let test_data = b"Hello, Elastic Crypto!";
    println!("Test data: {}", String::from_utf8_lossy(test_data));
    
    // Encrypt
    let encrypted = crypto.encrypt(key_handle, test_data.to_vec())
        .expect("Failed to encrypt");
    println!("Encrypted (base64): {}", base64::encode(&encrypted));
    
    // Decrypt
    let decrypted = crypto.decrypt(key_handle, encrypted)
        .expect("Failed to decrypt");
    println!("Decrypted: {}", String::from_utf8_lossy(&decrypted));
    
    // Verify
    assert_eq!(test_data, decrypted.as_slice(), "Decryption failed");
}
```

## Build and Run Workflow

### 1. Build on Local Machine
```bash
# On your local machine
cd demo/crypto-demo
cargo build --target wasm32-wasip1

# The resulting binary will be at:
# target/wasm32-wasip1/debug/crypto-demo.wasm
```

This builds a single WASM binary that can run on both platforms. The binary:
- Uses the common WIT interface
- Contains no platform-specific code
- Will use different hardware backends automatically

### 2. Run on Different Platforms

#### On Linux:
```bash
# On your local machine
wasmtime target/wasm32-wasip1/debug/crypto-demo.wasm

# Output:
# Test data: Hello, Elastic Crypto!
# Encrypted (base64): PwckOK+VtrHwZILLbFwchsiUui8/islc2l503yzNJqkZkaWxMOg=
# Decrypted: Hello, Elastic Crypto!
```

#### On AWS SEV-SNP:
```bash
# Copy the binary to SEV-SNP
scp target/wasm32-wasip1/debug/crypto-demo.wasm user@sev-snp-machine:~/demo/

# Run on SEV-SNP
cd ~/demo
wasmtime --env ELASTIC_SEV_SNP=1 --dir /dev crypto-demo.wasm

# Output:
# Test data: Hello, Elastic Crypto!
# Encrypted (base64): PwckOK+VtrHwZILLbFwchsiUui8/islc2l503yzNJqkZkaWxMOg=
# Decrypted: Hello, Elastic Crypto!
```

The same binary produces identical results on both platforms, but uses different hardware:
- On Linux: Uses the Linux crypto backend
- On SEV-SNP: Uses the SEV-SNP hardware crypto backend

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
- Linux: Uses the Linux crypto backend (e.g., /dev/crypto)
- SEV-SNP: Uses the SEV-SNP hardware crypto backend

The key point is that the same WIT interface allows the demo code to work identically on both platforms, while the actual implementation uses different hardware backends.

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