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

# Function to check SEV support
check_sev_support() {
    if dmesg | grep -i sev > /dev/null; then
        return 0  # SEV supported
    else
        return 1  # SEV not supported
    fi
}

# Function to check required tools
check_required_tools() {
    local missing_tools=()
    
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if ! command -v wasmtime &> /dev/null; then
        missing_tools+=("wasmtime")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo "❌ Missing required tools: ${missing_tools[*]}"
        echo
        echo "Please install the missing tools:"
        for tool in "${missing_tools[@]}"; do
            case $tool in
                "cargo")
                    echo "- Install Rust and Cargo:"
                    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    echo "  source $HOME/.cargo/env"
                    ;;
                "wasmtime")
                    echo "- Install Wasmtime:"
                    echo "  curl https://github.com/bytecodealliance/wasmtime/releases/download/v16.0.0/wasmtime-v16.0.0-x86_64-linux.tar.xz -L | tar xJ"
                    echo "  sudo cp wasmtime-v16.0.0-x86_64-linux/wasmtime /usr/local/bin/"
                    ;;
            esac
        done
        return 1
    fi
    return 0
}

# Get the absolute path of the script's directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
WASM_DIR="$(dirname "$SCRIPT_DIR")"

# Clear screen and start demo
clear
echo "=== WASM Clock Demo: Build and Run on SEV-SNP ==="
echo
sleep 2

# Check for required tools
if ! check_required_tools; then
    exit 1
fi

# Step 1: Show the source code
echo "Step 1: Source Code"
echo "==================="
show_file "$WASM_DIR/src/lib.rs" || exit 1
sleep 2

# Step 2: Check SEV support
echo "Step 2: Check SEV-SNP support"
echo "============================="
run_command "dmesg | grep -i sev"
if check_sev_support; then
    echo "❌ SEV-SNP support detected - this is an SEV machine"
    echo "We cannot build WASM modules on SEV machines"
    echo
    sleep 2

    # Step 3: Show build instructions
    echo "Step 3: Build Instructions"
    echo "========================="
    echo "We need to build the WASM module on a non-SEV machine:"
    echo "1. Use a regular Linux machine"
    echo "2. Run: cargo build --target wasm32-unknown-unknown --release"
    echo "3. Transfer the .wasm file to this SEV machine"
    echo
    sleep 2

    # Step 4: Show run instructions
    echo "Step 4: Run Instructions"
    echo "======================="
    echo "Once the .wasm file is transferred, we can run it here:"
    run_command "wasmtime run wasm_clock_example.wasm"
    echo "✓ WASM module can be executed on this SEV machine"
    echo
else
    echo "✓ No SEV support detected - this is a regular Linux machine"
    echo "We can build WASM modules here"
    echo
    sleep 2

    # Step 3: Build on non-SEV machine
    echo "Step 3: Build WASM Module"
    echo "========================"
    run_command "cd $WASM_DIR && cargo build --target wasm32-unknown-unknown --release"
    echo "Building..."
    # Actually run the build command
    if ! (cd "$WASM_DIR" && cargo build --target wasm32-unknown-unknown --release); then
        echo "Error: WASM module not built successfully"
        exit 1
    fi
    if [ ! -f "$WASM_DIR/target/wasm32-unknown-unknown/release/wasm_clock_example.wasm" ]; then
        echo "Error: WASM module not built successfully"
        exit 1
    fi
    echo "✓ WASM module built successfully"
    echo
    sleep 1

    # Step 4: Run on build machine
    echo "Step 4: Run on Build Machine"
    echo "==========================="
    run_command "cd $WASM_DIR && wasmtime run target/wasm32-unknown-unknown/release/wasm_clock_example.wasm"
    # Actually run the wasm module
    if ! (cd "$WASM_DIR" && wasmtime run target/wasm32-unknown-unknown/release/wasm_clock_example.wasm); then
        echo "Error: Failed to run WASM module"
        exit 1
    fi
    echo "✓ WASM module executed successfully on build machine"
    echo
    sleep 2

    # Step 5: Transfer instructions
    echo "Step 5: Transfer Instructions"
    echo "==========================="
    echo "To run on SEV machine (34.253.195.127):"
    echo "1. scp $WASM_DIR/target/wasm32-unknown-unknown/release/wasm_clock_example.wasm aws:~/"
    echo "2. ssh aws"
    echo "3. wasmtime run wasm_clock_example.wasm"
    echo
fi

echo "Demo completed!" 