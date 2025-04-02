use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum ClockError {
    SystemTimeError(std::time::SystemTimeError),
}

impl std::fmt::Display for ClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ClockError::SystemTimeError(err) => write!(f, "System time error: {}", err),
        }
    }
}

impl std::error::Error for ClockError {}

pub struct Clock {
    monotonic_start: Option<SystemTime>,
}

impl Clock {
    pub fn new() -> Self {
        Clock {
            monotonic_start: None,
        }
    }

    /// Read current time as seconds since UNIX epoch
    pub fn read_current_time(&self) -> Result<u64, ClockError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(ClockError::SystemTimeError)
    }

    /// Read current timezone
    pub fn read_timezone(&self) -> Result<String, ClockError> {
        Ok("UTC".to_string())
    }

    /// Start monotonic clock measurement
    pub fn start_monotonic(&mut self) -> Result<(), ClockError> {
        self.monotonic_start = Some(SystemTime::now());
        Ok(())
    }

    /// Stop monotonic clock and return elapsed time in milliseconds
    pub fn stop_monotonic(&mut self) -> Result<(), ClockError> {
        self.monotonic_start = None;
        Ok(())
    }

    /// Read elapsed time from monotonic clock in milliseconds without stopping it
    pub fn read_monotonic(&self) -> Result<u64, ClockError> {
        match self.monotonic_start {
            Some(start) => {
                SystemTime::now()
                    .duration_since(start)
                    .map(|d| d.as_secs())
                    .map_err(ClockError::SystemTimeError)
            }
            None => Ok(0),
        }
    }
} 