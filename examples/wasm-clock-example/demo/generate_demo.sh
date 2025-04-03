#!/bin/bash

# Function to simulate typing effect
type_effect() {
    local text="$1"
    for ((i=0; i<${#text}; i++)); do
        echo -n "${text:$i:1}"
        sleep 0.05
    done
    echo
}

# Function to simulate command execution
run_command() {
    local cmd="$1"
    echo -n "$ "
    type_effect "$cmd"
    sleep 1
}

# Function to show file content
show_file() {
    local file="$1"
    if [ ! -f "$file" ]; then
        echo "Error: File not found: $file"
        return 1
    fi
    echo "File: $file"
    echo "----------------------------------------"
    cat "$file"
    echo "----------------------------------------"
    sleep 2
}

# Clear screen and start demo
clear
echo "=== WASM Clock Demo: Build and Run on SEV-SNP ==="
echo
sleep 2

# Step 1: Show the source code
echo "Step 1: Source Code"
echo "==================="
show_file "../../src/lib.rs" || exit 1
sleep 2

# Step 2: Verify we're on SEV machine
echo "Step 2: Verify SEV-SNP support"
echo "============================="
run_command "dmesg | grep -i sev"
echo "✓ SEV-SNP support detected on this machine"
echo
sleep 2

# Step 3: Try to build (should fail)
echo "Step 3: Attempt to build (should fail)"
echo "====================================="
run_command "cd ../.. && cargo build --target wasm32-unknown-unknown --release"
echo "❌ Build failed - SEV machines cannot build WASM modules"
echo
sleep 2

# Step 4: Show we need to build elsewhere
echo "Step 4: Build on non-SEV machine"
echo "==============================="
echo "We need to build the WASM module on a non-SEV machine:"
echo "1. Use a regular Linux machine"
echo "2. Run: cargo build --target wasm32-unknown-unknown --release"
echo "3. Transfer the .wasm file to this SEV machine"
echo
sleep 2

# Step 5: Show how to run on SEV machine
echo "Step 5: Run on SEV machine"
echo "========================="
echo "Once the .wasm file is transferred, we can run it here:"
run_command "wasmtime run wasm_clock_example.wasm"
echo "✓ WASM module executed successfully on SEV machine"
echo
sleep 2

echo "Demo completed!" 