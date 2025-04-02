use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ClockError {
    SystemTimeError(std::time::SystemTimeError),
    TimeZoneError(String),
    MonotonicClockError(String),
}

impl fmt::Display for ClockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClockError::SystemTimeError(e) => write!(f, "System time error: {}", e),
            ClockError::TimeZoneError(e) => write!(f, "Time zone error: {}", e),
            ClockError::MonotonicClockError(e) => write!(f, "Monotonic clock error: {}", e),
        }
    }
}

impl Error for ClockError {}

impl From<std::time::SystemTimeError> for ClockError {
    fn from(err: std::time::SystemTimeError) -> ClockError {
        ClockError::SystemTimeError(err)
    }
}

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
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH)?;
        Ok(duration.as_secs())
    }

    /// Read current timezone
    pub fn read_timezone(&self) -> Result<String, ClockError> {
        // Try to get timezone from system
        match std::env::var("TZ") {
            Ok(tz) => Ok(tz),
            Err(_) => {
                // Fallback to UTC if TZ not set
                Ok("UTC".to_string())
            }
        }
    }

    /// Start monotonic clock measurement
    pub fn start_monotonic(&mut self) -> Result<(), ClockError> {
        self.monotonic_start = Some(SystemTime::now());
        Ok(())
    }

    /// Stop monotonic clock and return elapsed time in milliseconds
    pub fn stop_monotonic(&mut self) -> Result<u64, ClockError> {
        match self.monotonic_start {
            Some(start) => {
                let elapsed = SystemTime::now()
                    .duration_since(start)
                    .map_err(|e| ClockError::MonotonicClockError(e.to_string()))?;
                self.monotonic_start = None;
                Ok(elapsed.as_millis() as u64)
            }
            None => Err(ClockError::MonotonicClockError(
                "Monotonic clock not started".to_string(),
            )),
        }
    }

    /// Read elapsed time from monotonic clock in milliseconds without stopping it
    pub fn read_monotonic(&self) -> Result<u64, ClockError> {
        match self.monotonic_start {
            Some(start) => {
                let elapsed = SystemTime::now()
                    .duration_since(start)
                    .map_err(|e| ClockError::MonotonicClockError(e.to_string()))?;
                Ok(elapsed.as_millis() as u64)
            }
            None => Err(ClockError::MonotonicClockError(
                "Monotonic clock not started".to_string(),
            )),
        }
    }
} 