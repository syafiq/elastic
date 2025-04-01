mod clock;

use clock::{Clock, ClockError};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Initializing Clock Interface...");
    let mut clock = Clock::new();
    clock.init()?;
    println!("Clock Interface initialized successfully");

    // Test time functions
    println!("\nTesting time functions:");
    let ms_time = clock.get_time_ms()?;
    println!("Current time (ms): {}", ms_time);
    
    let us_time = clock.get_time_us()?;
    println!("Current time (us): {}", us_time);
    println!("Time difference (us - ms): {}", us_time - (ms_time * 1000));

    // Test sleep functions
    println!("\nTesting sleep functions:");
    println!("Sleeping for 100ms...");
    let start_ms = clock.get_time_ms()?;
    clock.sleep_ms(100).await?;
    let end_ms = clock.get_time_ms()?;
    println!("Actual sleep duration: {}ms", end_ms - start_ms);

    println!("Sleeping for 1000us...");
    let start_us = clock.get_time_us()?;
    clock.sleep_us(1000).await?;
    let end_us = clock.get_time_us()?;
    println!("Actual sleep duration: {}us", end_us - start_us);

    Ok(())
}
