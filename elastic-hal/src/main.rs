mod clock;

use clock::{Clock, ClockError};

#[tokio::main]
async fn main() -> Result<(), ClockError> {
    // Create and initialize the clock
    let mut clock = Clock::new();
    clock.init()?;

    // Get current time
    let ms_time = clock.get_time_ms()?;
    let us_time = clock.get_time_us()?;

    println!("Current time (ms): {}", ms_time);
    println!("Current time (us): {}", us_time);

    // Test sleep functions
    println!("Sleeping for 100ms...");
    clock.sleep_ms(100).await?;

    println!("Sleeping for 1000us...");
    clock.sleep_us(1000).await?;

    println!("All operations completed successfully!");
    Ok(())
}
