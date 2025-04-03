# WASM Clock Demo

This demo showcases the workflow of building and running the WASM clock module across different environments.

## Text-Based Demo Video

The demo shows the complete workflow:
1. Building the WASM module on Linux
2. Transferring the module to SEV-SNP
3. Running the module on SEV-SNP
4. Comparing results between environments

### Running the Demo

1. Make the script executable:
```bash
chmod +x generate_demo.sh
```

2. Run the demo:
```bash
./generate_demo.sh
```

The script will:
- Show the source code
- Simulate the build process
- Show the generated WASM module
- Simulate transferring to SEV-SNP
- Show the execution on SEV-SNP
- Compare results between environments

Note: The script uses simulated commands and timing for demonstration purposes. In a real environment, you would need to:
1. Have a SEV-SNP environment set up
2. Configure proper SSH access
3. Have wasmtime installed on both systems

## Prerequisites

- Python 3.6 or higher
- pip (Python package manager)

## Setup

1. Install the required Python packages:
```bash
pip install -r requirements.txt
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

The demo will display the current time in seconds, updating every 0.1 seconds. Press Ctrl+C to exit. 