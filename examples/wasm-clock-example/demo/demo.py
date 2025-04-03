#!/usr/bin/env python3

import wasmtime
import time
import os
import sys
from datetime import datetime

def clear_screen():
    os.system('cls' if os.name == 'nt' else 'clear')

def format_time(time_ns):
    # Convert nanoseconds to seconds
    seconds = time_ns / 1_000_000_000
    return f"{seconds:.9f} seconds"

def main():
    # Load the WASM module
    engine = wasmtime.Engine()
    module = wasmtime.Module.from_file(engine, "../target/wasm32-unknown-unknown/release/wasm_clock_example.wasm")
    store = wasmtime.Store(engine)
    instance = wasmtime.Instance(store, module, [])

    # Get the get_current_time function
    get_current_time = instance.exports(store)["get_current_time"]

    try:
        while True:
            clear_screen()
            print("WASM Clock Demo")
            print("===============")
            print(f"Current time: {format_time(get_current_time(store))}")
            print("\nPress Ctrl+C to exit")
            time.sleep(0.1)
    except KeyboardInterrupt:
        print("\nDemo ended")

if __name__ == "__main__":
    main() 