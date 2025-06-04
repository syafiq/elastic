# Elastic Crypto Demo Script

This document provides a step-by-step script for demonstrating the Elastic Crypto HAL's "build once, run anywhere" capabilities.

## Demo Flow (Total: 10-12 minutes)

### 1. Introduction (1 minute)
"Today I'm going to demonstrate how Elastic's Crypto HAL enables true 'build once, run anywhere' functionality. We'll take a single WASM binary and run it on two different platforms - a standard Linux machine and an AWS SEV-SNP machine - without any code changes. The same binary will use different hardware backends automatically while maintaining consistent behavior."

### 2. Show the Code (2 minutes)
"Let's look at the demo code that uses our HAL. This code:
- Uses a common WIT interface
- Contains no platform-specific code
- Will work identically on both platforms

The key parts are:
1. Initializing the crypto HAL
2. Generating a symmetric key
3. Encrypting and decrypting data
4. Verifying the results"

[Show the code from `src/main.rs`]

### 3. Build Process (1 minute)
"Now, let's build this code into a WASM binary. This is a one-time build that will work on both platforms:"

```bash
# On your local machine
cd demo/crypto-demo
cargo build --target wasm32-wasip1
```

"Notice that we're building for `wasm32-wasip1` target, which is our common platform. The resulting binary will be at `target/wasm32-wasip1/debug/crypto-demo.wasm`."

### 4. Run on Linux (2 minutes)
"Let's first run it on our local Linux machine:"

```bash
wasmtime target/wasm32-wasip1/debug/crypto-demo.wasm
```

"Notice the output:
- It shows we're running on the Linux platform
- The encryption and decryption work perfectly
- The data is preserved exactly as expected"

### 5. Run on SEV-SNP (3 minutes)
"Now, let's take the exact same binary and run it on an AWS SEV-SNP machine:"

```bash
# Copy the binary to SEV-SNP
scp target/wasm32-wasip1/debug/crypto-demo.wasm user@sev-snp-machine:~/demo/

# Run on SEV-SNP
cd ~/demo
wasmtime --env ELASTIC_SEV_SNP=1 --dir /dev crypto-demo.wasm
```

"Notice that:
- We're using the exact same binary
- No code changes were needed
- The output is identical to Linux
- But now it's using the SEV-SNP hardware backend"

### 6. Key Points to Highlight (2 minutes)
"Let me highlight what makes this possible:

1. **Common Interface**: The WIT interface defines a common API that both platforms implement
2. **Platform Detection**: The HAL automatically detects which platform it's running on
3. **Hardware Abstraction**: Different hardware backends are used transparently
4. **Consistent Behavior**: The same input produces the same output on both platforms

This means you can:
- Build your application once
- Deploy it anywhere
- Get the best hardware acceleration available
- Maintain consistent behavior"

### 7. Q&A (2-3 minutes)
"Any questions about how this works or how you can use it in your applications?"

## Demo Preparation Checklist

Before the demo:
1. [ ] Build the WASM binary on your local machine
2. [ ] Test the binary locally
3. [ ] Ensure SEV-SNP machine is accessible
4. [ ] Have the demo code ready to show
5. [ ] Prepare the terminal windows for both platforms

During the demo:
1. [ ] Show the code first
2. [ ] Run the build command
3. [ ] Demonstrate local execution
4. [ ] Copy and run on SEV-SNP
5. [ ] Highlight the identical results
6. [ ] Explain the hardware differences

## Troubleshooting

If something goes wrong:
1. Check if the WASM binary was built correctly
2. Verify the SEV-SNP environment variable is set
3. Ensure the SEV-SNP device is accessible
4. Check the logs for any error messages 