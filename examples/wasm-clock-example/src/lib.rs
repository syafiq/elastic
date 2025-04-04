use elastic_clock::{ClockConfig, ClockContext, ClockType};
use std::error::Error;

#[no_mangle]
pub extern "C" fn get_current_time() -> u64 {
    match try_get_current_time() {
        Ok(time) => time,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    let time = get_current_time();
    // Print the time to stdout
    println!("Current time: {}", time);
}

fn try_get_current_time() -> Result<u64, Box<dyn Error>> {
    let context = ClockContext::new();
    let config = ClockConfig {
        clock_type: ClockType::System,
        high_resolution: true,
    };
    
    let handle = context.create_clock(&config)?;
    let time = context.get_time(handle)?;
    context.destroy_clock(handle)?;
    
    Ok(time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_time() {
        let time = get_current_time();
        assert!(time > 0);
    }
}
