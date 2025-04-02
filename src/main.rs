mod clock;
mod file;

use clock::Clock;

fn main() {
    println!("ELASTIC HAL Implementation");

    let mut clock = Clock::new();

    // Start monotonic clock
    match clock.start_monotonic() {
        Ok(_) => println!("Started monotonic clock"),
        Err(e) => eprintln!("Error starting monotonic clock: {}", e),
    }

    // Read current time
    match clock.read_current_time() {
        Ok(time) => println!("Current time (seconds since epoch): {}", time),
        Err(e) => eprintln!("Error reading current time: {}", e),
    }

    // Read timezone
    match clock.read_timezone() {
        Ok(tz) => println!("Current timezone: {}", tz),
        Err(e) => eprintln!("Error reading timezone: {}", e),
    }

    // Stop monotonic clock
    match clock.stop_monotonic() {
        Ok(_) => println!("Stopped monotonic clock"),
        Err(e) => eprintln!("Error stopping monotonic clock: {}", e),
    }

    // Read final elapsed time
    match clock.read_monotonic() {
        Ok(elapsed) => println!("Final elapsed time (seconds): {}", elapsed),
        Err(e) => eprintln!("Error reading monotonic time: {}", e),
    }
} 