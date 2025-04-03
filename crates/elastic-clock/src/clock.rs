use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crate::{ClockConfig, ClockType};

#[derive(Debug)]
struct Clock {
    clock_type: ClockType,
    high_resolution: bool,
    start_time: Option<Instant>,
    last_time: Option<u64>,
}

pub struct ClockManager {
    clocks: Arc<Mutex<HashMap<u32, Clock>>>,
    next_handle: Arc<Mutex<u32>>,
}

impl ClockManager {
    pub fn new() -> Self {
        Self {
            clocks: Arc::new(Mutex::new(HashMap::new())),
            next_handle: Arc::new(Mutex::new(1)),
        }
    }

    pub fn create_clock(&self, config: &ClockConfig) -> Result<u32, String> {
        let mut handles = self.next_handle.lock().unwrap();
        let handle = *handles;
        *handles += 1;

        let clock = Clock {
            clock_type: config.clock_type,
            high_resolution: config.high_resolution,
            start_time: Some(Instant::now()),
            last_time: None,
        };

        let mut clocks = self.clocks.lock().unwrap();
        clocks.insert(handle, clock);

        Ok(handle)
    }

    pub fn destroy_clock(&self, handle: u32) -> Result<(), String> {
        let mut clocks = self.clocks.lock().unwrap();
        clocks.remove(&handle)
            .map(|_| ())
            .ok_or_else(|| format!("Clock handle {} not found", handle))
    }

    pub fn get_time(&self, handle: u32) -> Result<u64, String> {
        let clocks = self.clocks.lock().unwrap();
        let clock = clocks.get(&handle)
            .ok_or_else(|| format!("Clock handle {} not found", handle))?;

        match clock.clock_type {
            ClockType::System => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?;
                Ok(now.as_nanos() as u64)
            }
            ClockType::Monotonic => {
                let start = clock.start_time
                    .ok_or_else(|| "Clock not initialized".to_string())?;
                let elapsed = start.elapsed();
                Ok(elapsed.as_nanos() as u64)
            }
            ClockType::Process => {
                // For process time, we'll use monotonic time as an approximation
                let start = clock.start_time
                    .ok_or_else(|| "Clock not initialized".to_string())?;
                let elapsed = start.elapsed();
                Ok(elapsed.as_nanos() as u64)
            }
            ClockType::Thread => {
                // For thread time, we'll use monotonic time as an approximation
                let start = clock.start_time
                    .ok_or_else(|| "Clock not initialized".to_string())?;
                let elapsed = start.elapsed();
                Ok(elapsed.as_nanos() as u64)
            }
        }
    }

    pub fn get_resolution(&self, handle: u32) -> Result<u64, String> {
        let clocks = self.clocks.lock().unwrap();
        let clock = clocks.get(&handle)
            .ok_or_else(|| format!("Clock handle {} not found", handle))?;

        // Return resolution in nanoseconds
        if clock.high_resolution {
            Ok(1) // 1 nanosecond resolution
        } else {
            Ok(1_000_000) // 1 millisecond resolution
        }
    }

    pub async fn sleep(&self, handle: u32, duration: u64) -> Result<(), String> {
        let clocks = self.clocks.lock().unwrap();
        let clock = clocks.get(&handle)
            .ok_or_else(|| format!("Clock handle {} not found", handle))?;

        let duration = if clock.high_resolution {
            Duration::from_nanos(duration)
        } else {
            Duration::from_millis(duration / 1_000_000)
        };

        tokio::time::sleep(duration).await;
        Ok(())
    }

    pub fn get_elapsed(&self, handle: u32) -> Result<u64, String> {
        let clocks = self.clocks.lock().unwrap();
        let clock = clocks.get(&handle)
            .ok_or_else(|| format!("Clock handle {} not found", handle))?;

        let start = clock.start_time
            .ok_or_else(|| "Clock not initialized".to_string())?;
        let elapsed = start.elapsed();
        Ok(elapsed.as_nanos() as u64)
    }
} 