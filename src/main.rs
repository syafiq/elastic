mod clock;

use std::thread;
use std::time::Duration;

fn main() {
    let mut clock = clock::Clock::new();

    // Test current time
    match clock.read_current_time() {
        Ok(time) => println!("Current time (seconds since epoch): {}", time),
        Err(e) => println!("Error reading current time: {}", e),
    }

    // Test timezone
    match clock.read_timezone() {
        Ok(tz) => println!("Current timezone: {}", tz),
        Err(e) => println!("Error reading timezone: {}", e),
    }

    // Test monotonic clock
    match clock.start_monotonic() {
        Ok(_) => println!("Started monotonic clock"),
        Err(e) => println!("Error starting monotonic clock: {}", e),
    }

    // Sleep for 2 seconds
    thread::sleep(Duration::from_secs(2));

    // Read elapsed time without stopping
    match clock.read_monotonic() {
        Ok(elapsed) => println!("Elapsed time (ms): {}", elapsed),
        Err(e) => println!("Error reading monotonic clock: {}", e),
    }

    // Sleep for 1 more second
    thread::sleep(Duration::from_secs(1));

    // Stop and get final elapsed time
    match clock.stop_monotonic() {
        Ok(elapsed) => println!("Final elapsed time (ms): {}", elapsed),
        Err(e) => println!("Error stopping monotonic clock: {}", e),
    }

    // Try to read monotonic clock after stopping (should error)
    match clock.read_monotonic() {
        Ok(elapsed) => println!("Unexpected success reading stopped clock: {}", elapsed),
        Err(e) => println!("Expected error reading stopped clock: {}", e),
    }
} 