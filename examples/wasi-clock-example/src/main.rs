use wasi_clock::WasiClock;

fn main() {
    println!("WASI Clock Example");
    println!("==================");

    // Create WASI-compliant clock
    let mut clock = WasiClock::new().expect("Failed to create WASI clock");

    // Test monotonic clock
    println!("\nMonotonic Clock:");
    match clock.monotonic_now() {
        Ok(instant) => println!("  Current time: {} nanoseconds", instant.as_nanos()),
        Err(e) => println!("  Error getting monotonic time: {:?}", e),
    }

    match clock.monotonic_resolution() {
        Ok(resolution) => println!("  Resolution: {} nanoseconds", resolution.as_nanos()),
        Err(e) => println!("  Error getting resolution: {:?}", e),
    }

    // Test wall clock
    println!("\nWall Clock:");
    match clock.wall_now() {
        Ok(datetime) => println!("  Current time: {} seconds since epoch", datetime.seconds),
        Err(e) => println!("  Error getting wall time: {:?}", e),
    }

    match clock.wall_resolution() {
        Ok(resolution) => println!("  Resolution: {} seconds", resolution.as_secs()),
        Err(e) => println!("  Error getting resolution: {:?}", e),
    }

    // Test SEV-SNP detection
    println!("\nEnvironment Detection:");
    let is_sevsnp = std::env::var("ELASTIC_SEV_SNP").map(|v| v == "1").unwrap_or(false);
    println!("  SEV-SNP environment: {}", is_sevsnp);

    if is_sevsnp {
        println!("  Using SEV-SNP hardware-accelerated clock");
    } else {
        println!("  Using standard system clock");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_clock_functionality() {
        let mut clock = WasiClock::new().unwrap();

        // Test monotonic clock
        let now1 = clock.monotonic_now().unwrap();
        let resolution = clock.monotonic_resolution().unwrap();
        
        assert!(now1.as_nanos() > 0);
        assert!(resolution.as_nanos() > 0);

        // Test wall clock
        let wall_now = clock.wall_now().unwrap();
        let wall_resolution = clock.wall_resolution().unwrap();
        
        assert!(wall_now.seconds > 0);
        assert!(wall_resolution.as_secs() > 0);
    }
} 