# Clock Interface Implementation

This project implements the Clock Interface as specified in the ELASTIC HAL specification. The implementation provides WASI-compatible functions for time-related operations in a confidential computing environment, with WebAssembly Interface Types (WIT) support.

## Features

The Clock Interface provides the following functionality:

1. **Read Current Time**
   - Returns the current system time in seconds since UNIX epoch
   - WASI-compatible implementation

2. **Read Time Zone**
   - Returns the configured time zone for the system
   - WASI-compatible implementation

3. **Monotonic Clock Operations**
   - Start monotonic clock measurement
   - Stop monotonic clock and return elapsed time
   - Read elapsed time without stopping the clock
   - Single monotonic clock instance support
   - WASI-compatible implementation

4. **WIT Interface Support**
   - WebAssembly Interface Types (WIT) definition
   - Multi-language binding generation
   - Cross-platform compatibility

## WIT Interface

The Clock Interface is defined using WIT in `tests/wit/clock.wit`:

```wit
package elastic:clock@0.1.0;

interface types {
    variant clock-error {
        system-time-error(string),
        timezone-error(string),
        monotonic-clock-error(string),
    }
}

interface clock {
    func read-current-time() -> result<u64, types.clock-error>;
    func read-timezone() -> result<string, types.clock-error>;
    func start-monotonic() -> result<u32, types.clock-error>;
    func stop-monotonic() -> result<u64, types.clock-error>;
    func read-monotonic() -> result<u64, types.clock-error>;
}

world clock-impl {
    export clock;
}
```

### Generating Bindings

To generate bindings for different languages:

```bash
# Install wit-bindgen
cargo install wit-bindgen-cli

# Generate Rust bindings
wit-bindgen rust tests/wit/clock.wit

# Generate Python bindings
wit-bindgen python tests/wit/clock.wit
```

## Usage

### Rust Usage

```rust
use clock::Clock;

fn main() {
    let mut clock = Clock::new();
    
    // Read current time
    match clock.read_current_time() {
        Ok(time) => println!("Current time: {}", time),
        Err(e) => println!("Error reading time: {}", e),
    }
    
    // Read timezone
    match clock.read_timezone() {
        Ok(tz) => println!("Timezone: {}", tz),
        Err(e) => println!("Error reading timezone: {}", e),
    }
    
    // Use monotonic clock
    match clock.start_monotonic() {
        Ok(_) => println!("Started monotonic clock"),
        Err(e) => println!("Error starting monotonic clock: {}", e),
    }
    
    // Read elapsed time
    match clock.read_monotonic() {
        Ok(elapsed) => println!("Elapsed time: {}ms", elapsed),
        Err(e) => println!("Error reading monotonic clock: {}", e),
    }
    
    // Stop clock and get final time
    match clock.stop_monotonic() {
        Ok(elapsed) => println!("Final elapsed time: {}ms", elapsed),
        Err(e) => println!("Error stopping monotonic clock: {}", e),
    }
}
```

### Python Usage

```python
from clock_impl import Clock

clock = Clock()

# Read current time
try:
    time = clock.read_current_time()
    print(f"Current time: {time}")
except Exception as e:
    print(f"Error reading time: {e}")

# Read timezone
try:
    tz = clock.read_timezone()
    print(f"Timezone: {tz}")
except Exception as e:
    print(f"Error reading timezone: {e}")

# Use monotonic clock
try:
    clock.start_monotonic()
    print("Started monotonic clock")
    
    elapsed = clock.read_monotonic()
    print(f"Elapsed time: {elapsed}ms")
    
    final_time = clock.stop_monotonic()
    print(f"Final elapsed time: {final_time}ms")
except Exception as e:
    print(f"Error with monotonic clock: {e}")
```

## Error Handling

The implementation provides proper error handling through the `ClockError` enum:

- `SystemTimeError`: For system time-related errors
- `TimeZoneError`: For timezone-related errors
- `MonotonicClockError`: For monotonic clock operation errors

## Testing

The project includes comprehensive tests for all Clock Interface functionality:

1. WIT interface validation
2. Rust implementation tests
3. Python implementation tests

To run the tests:
```bash
# Run all tests
./tests/run_tests.sh

# Run specific test suites
cargo test  # Rust tests
python -m pytest tests/python/  # Python tests
```

## Dependencies

- Rust standard library
- wit-bindgen-cli (for generating bindings)
- Python 3.x (for Python bindings)

## License

This project is part of the ELASTIC project and follows its licensing terms.
