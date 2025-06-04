# Elastic Crypto Demo

This demo showcases the cross-platform capabilities of the Elastic Crypto interface. It demonstrates how to:
1. Build once on your local machine
2. Run the same binary on different platforms (Linux and SEV-SNP)
3. Get consistent results across platforms

## WIT Interface

The demo uses the WIT interface defined in `crates/elastic-crypto/wit/crypto.wit`. This interface ensures consistent behavior across different platforms.

## Building

```bash
# Build for WASI
cargo build --target wasm32-wasip1
```

## Running

### On Linux
```bash
wasmtime ../../target/wasm32-wasip1/debug/crypto-demo.wasm
```

### On AWS SEV-SNP
```bash
# Set the environment variable to enable SEV-SNP mode
export ELASTIC_SEV_SNP=1
# IMPORTANT: Mount /dev so the WASM module can access /dev/sev-guest
wasmtime --dir /dev ../../target/wasm32-wasip1/debug/crypto-demo.wasm
```

> **Note:**
> The `--dir /dev` flag is required for the WASM module to access `/dev/sev-guest` inside the sandbox. Without this, SEV-SNP hardware will not be detected by the demo, even if it exists on the host.

## What to Expect

The demo will:
1. Show which mode it's running in (Linux or SEV-SNP)
2. Generate an AES-256 key
3. Encrypt and decrypt a sample message
4. Calculate a SHA-256 hash
5. Clean up by deleting the key

The same WASM binary will use different hardware backends depending on the environment:
- On Linux: Uses the Linux crypto backend
- On SEV-SNP: Uses the SEV-SNP hardware crypto backend

## Notes

- The demo uses AES-GCM for encryption/decryption
- Keys are not stored in secure storage for demonstration purposes
- The same binary produces the same results on both platforms 