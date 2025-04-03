use elastic_clock::{clock::ClockManager, ClockConfig, ClockType};

#[no_mangle]
pub extern "C" fn get_current_time() -> u64 {
    let manager = ClockManager::new();
    let config = ClockConfig {
        clock_type: ClockType::Monotonic,
        high_resolution: true,
    };
    let handle = manager.create_clock(&config).unwrap();
    let time = manager.get_time(handle).unwrap();
    manager.destroy_clock(handle).unwrap();
    time
}

fn main() {
    let time = get_current_time();
    println!("Current time: {} ns", time);
} 