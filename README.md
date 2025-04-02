# Clock Interface Implementation

This project implements the Clock Interface as specified in the ELASTIC HAL specification. The implementation provides WASI-compatible functions for time-related operations in a confidential computing environment.

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

## Usage

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

## Error Handling

The implementation provides proper error handling through the `ClockError` enum:

- `SystemTimeError`: For system time-related errors
- `TimeZoneError`: For timezone-related errors
- `MonotonicClockError`: For monotonic clock operation errors

## Testing

The project includes a test program that verifies all Clock Interface functionality:

1. Reading current time
2. Reading timezone
3. Starting monotonic clock
4. Reading elapsed time
5. Stopping monotonic clock
6. Error handling for invalid operations

To run the tests:
```bash
cargo run
```

## Dependencies

- Rust standard library
- No external dependencies required

## License

This project is part of the ELASTIC project and follows its licensing terms.
