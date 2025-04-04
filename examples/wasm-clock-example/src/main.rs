use elastic_clock::{ClockConfig, ClockContext, ClockType};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let context = ClockContext::new();
    let config = ClockConfig {
        clock_type: ClockType::System,
        high_resolution: true,
    };
    
    let handle = context.create_clock(&config)?;
    let time = context.get_time(handle)?;
    context.destroy_clock(handle)?;
    
    println!("Current time: {}", time);
    Ok(())
} 