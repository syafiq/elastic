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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
