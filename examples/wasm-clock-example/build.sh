#!/bin/bash

# Build native binary
echo "Building native binary..."
cargo build --release

# Build WASM binary
echo "Building WASM binary..."
cargo build --target wasm32-unknown-unknown --release

# Copy WASM binary to demo directory
echo "Copying WASM binary to demo directory..."
mkdir -p demo
cp target/wasm32-unknown-unknown/release/wasm_clock_example.wasm demo/

echo "Build complete!"
echo "Native binary: target/release/wasm_clock_example"
echo "WASM binary: demo/wasm_clock_example.wasm" 