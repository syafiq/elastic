use elastic_clock::{ClockConfig, ClockContext, ClockType};
use std::error::Error;

#[no_mangle]
pub extern "C" fn get_current_time() -> u64 {
    match try_get_current_time() {
        Ok(time) => {
            println!("Successfully retrieved time: {}", time);
            time
        },
        Err(e) => {
            println!("Error getting time: {}", e);
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    println!("Starting WASM clock example");
    let time = get_current_time();
    println!("Current time: {}", time);
}

fn try_get_current_time() -> Result<u64, Box<dyn Error>> {
    println!("Creating clock context...");
    let context = ClockContext::new();
    
    // Check if we're running in a SEV-SNP environment
    let is_sev_snp = std::path::Path::new("/dev/sev-guest").exists();
    println!("Running in SEV-SNP environment: {}", is_sev_snp);
    
    println!("Configuring clock...");
    let config = ClockConfig {
        clock_type: ClockType::System,
        high_resolution: true,
    };
    
    println!("Creating clock with config: {:?}", config);
    let handle = context.create_clock(&config)?;
    
    println!("Getting time from clock...");
    let time = context.get_time(handle)?;
    
    println!("Destroying clock...");
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
