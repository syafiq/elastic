use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep as tokio_sleep;

use super::{ClockConfig, ClockType};

pub struct ClockManager {
    clocks: Mutex<HashMap<u32, Clock>>,
    next_handle: Mutex<u32>,
}

struct Clock {
    config: ClockConfig,
    start_time: Instant,
    last_time: Instant,
}

impl ClockManager {
    pub fn new() -> Self {
        Self {
            clocks: Mutex::new(HashMap::new()),
            next_handle: Mutex::new(1),
        }
    }

    pub fn create_clock(&self, config: &ClockConfig) -> Result<u32, String> {
        let mut clocks = self.clocks.lock().map_err(|e| e.to_string())?;
        let mut next_handle = self.next_handle.lock().map_err(|e| e.to_string())?;
        
        let handle = *next_handle;
        *next_handle += 1;

        let clock = Clock {
            config: config.clone(),
            start_time: Instant::now(),
            last_time: Instant::now(),
        };

        clocks.insert(handle, clock);
        Ok(handle)
    }

    pub fn destroy_clock(&self, handle: u32) -> Result<(), String> {
        let mut clocks = self.clocks.lock().map_err(|e| e.to_string())?;
        clocks.remove(&handle).ok_or_else(|| "Clock not found".to_string())?;
        Ok(())
    }

    pub fn get_time(&self, handle: u32) -> Result<u64, String> {
        let clocks = self.clocks.lock().map_err(|e| e.to_string())?;
        let clock = clocks.get(&handle).ok_or_else(|| "Clock not found".to_string())?;

        let time = match clock.config.clock_type {
            ClockType::System => {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_nanos() as u64
            }
            ClockType::Monotonic | ClockType::Process | ClockType::Thread => {
                clock.start_time.elapsed().as_nanos() as u64
            }
        };

        Ok(time)
    }

    pub fn get_resolution(&self, handle: u32) -> Result<u64, String> {
        let clocks = self.clocks.lock().map_err(|e| e.to_string())?;
        let clock = clocks.get(&handle).ok_or_else(|| "Clock not found".to_string())?;

        let resolution = if clock.config.high_resolution {
            1 // 1 nanosecond
        } else {
            1_000_000 // 1 millisecond
        };

        Ok(resolution)
    }

    pub async fn sleep(&self, handle: u32, duration: u64) -> Result<(), String> {
        let clocks = self.clocks.lock().map_err(|e| e.to_string())?;
        let _clock = clocks.get(&handle).ok_or_else(|| "Clock not found".to_string())?;

        tokio_sleep(Duration::from_nanos(duration)).await;
        Ok(())
    }

    pub fn get_elapsed(&self, handle: u32) -> Result<u64, String> {
        let mut clocks = self.clocks.lock().map_err(|e| e.to_string())?;
        let clock = clocks.get_mut(&handle).ok_or_else(|| "Clock not found".to_string())?;

        let elapsed = clock.last_time.elapsed().as_nanos() as u64;
        clock.last_time = Instant::now();
        Ok(elapsed)
    }
} 