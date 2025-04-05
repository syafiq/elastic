# WASM Crypto Demo

This demo showcases the workflow of building and running the WASM crypto module across different environments.

## Features Demonstrated
- Symmetric encryption/decryption using AES-256-GCM
- Key generation and management
- Secure message handling
- Error handling and logging

## Prerequisites
- Python 3.6 or higher
- pip (Python package manager)
- Rust and wasm32-unknown-unknown target
- wasmtime

## Setup

1. Install the required Python packages:
```bash
pip install wasmtime
```

2. Build the WASM module (from the project root):
```bash
cargo build --target wasm32-unknown-unknown --release
```

## Running the Demo

From the demo directory:
```bash
python demo.py
```

The demo will:
1. Generate a random symmetric key
2. Create a random message
3. Encrypt the message
4. Decrypt the message
5. Verify the decryption
6. Display the results

The demo runs continuously, showing a new encryption/decryption cycle every 2 seconds. Press Ctrl+C to exit.

## Security Features
- Uses AES-256-GCM for symmetric encryption
- Secure key generation using system RNG
- Proper key cleanup after use
- Error handling for all operations

## Notes
- The demo uses the elastic-crypto crate's symmetric encryption capabilities
- All operations are performed within the WASM environment
- The demo shows both successful operations and error handling
- Results are displayed in hexadecimal format for clarity 