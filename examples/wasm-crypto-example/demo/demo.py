#!/usr/bin/env python3

import wasmtime
import time
import os
import sys
from datetime import datetime

def clear_screen():
    os.system('cls' if os.name == 'nt' else 'clear')

def format_hex(data):
    return ''.join(f'{b:02x}' for b in data)

def main():
    # Load the WASM module
    engine = wasmtime.Engine()
    module = wasmtime.Module.from_file(engine, "../target/wasm32-unknown-unknown/release/wasm_crypto_example.wasm")
    store = wasmtime.Store(engine)
    instance = wasmtime.Instance(store, module, [])

    # Get the demo_symmetric_crypto function
    demo_symmetric_crypto = instance.exports(store)["demo_symmetric_crypto"]

    try:
        while True:
            clear_screen()
            print("WASM Crypto Demo")
            print("===============")
            
            # Run the crypto demo
            result_ptr = demo_symmetric_crypto(store)
            if result_ptr != 0:
                print("\nCrypto demo completed successfully")
            else:
                print("\nCrypto demo failed")
            
            print("\nPress Ctrl+C to exit")
            time.sleep(2)  # Wait 2 seconds between demos
    except KeyboardInterrupt:
        print("\nDemo ended")

if __name__ == "__main__":
    main() 