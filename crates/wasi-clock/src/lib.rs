use elastic_clock::{ClockContext, ClockConfig, ClockType};
use thiserror::Error;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Error)]
pub enum WasiClockError {
    #[error("ELASTIC clock error: {0}")]
    ElasticError(#[from] elastic_clock::ClockError),
    #[error("Invalid datetime conversion")]
    InvalidDatetime,
    #[error("Clock operation failed: {0}")]
    OperationFailed(String),
}

pub type Result<T> = std::result::Result<T, WasiClockError>;

/// WASI-compliant clock types
#[derive(Debug, Clone)]
pub struct WasiDatetime {
    pub seconds: u64,
    pub nanoseconds: u32,
}

#[derive(Debug, Clone)]
pub struct WasiDuration {
    pub nanoseconds: u64,
}

impl WasiDuration {
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanoseconds: nanos }
    }
    
    pub fn from_secs(secs: u64) -> Self {
        Self { nanoseconds: secs * 1_000_000_000 }
    }
    
    pub fn as_nanos(&self) -> u64 {
        self.nanoseconds
    }
    
    pub fn as_secs(&self) -> u64 {
        self.nanoseconds / 1_000_000_000
    }
}

#[derive(Debug, Clone)]
pub struct WasiInstant {
    pub nanoseconds: u64,
}

impl WasiInstant {
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanoseconds: nanos }
    }
    
    pub fn as_nanos(&self) -> u64 {
        self.nanoseconds
    }
}

/// WASI-compliant clock implementation using ELASTIC
pub struct WasiClock {
    elastic_clock: ClockContext,
    system_clock_handle: u32,
    monotonic_clock_handle: u32,
}

impl WasiClock {
    pub fn new() -> Result<Self> {
        let elastic_clock = ClockContext::new();
        
        // Create system clock
        let system_config = ClockConfig {
            clock_type: ClockType::System,
            high_resolution: true,
        };
        let system_clock_handle = elastic_clock.create_clock(&system_config)
            .map_err(|e| WasiClockError::OperationFailed(e.to_string()))?;
        
        // Create monotonic clock
        let monotonic_config = ClockConfig {
            clock_type: ClockType::Monotonic,
            high_resolution: true,
        };
        let monotonic_clock_handle = elastic_clock.create_clock(&monotonic_config)
            .map_err(|e| WasiClockError::OperationFailed(e.to_string()))?;
        
        Ok(Self { 
            elastic_clock,
            system_clock_handle,
            monotonic_clock_handle,
        })
    }
    
    /// WASI monotonic clock interface
    pub fn monotonic_now(&mut self) -> Result<WasiInstant> {
        let time_nanos = self.elastic_clock.get_time(self.monotonic_clock_handle)
            .map_err(|e| WasiClockError::OperationFailed(e.to_string()))?;
        Ok(WasiInstant::from_nanos(time_nanos))
    }

    pub fn monotonic_resolution(&mut self) -> Result<WasiDuration> {
        let resolution_nanos = self.elastic_clock.get_resolution(self.monotonic_clock_handle)
            .map_err(|e| WasiClockError::OperationFailed(e.to_string()))?;
        Ok(WasiDuration::from_nanos(resolution_nanos))
    }
    
    /// WASI wall clock interface
    pub fn wall_now(&mut self) -> Result<WasiDatetime> {
        let time_nanos = self.elastic_clock.get_time(self.system_clock_handle)
            .map_err(|e| WasiClockError::OperationFailed(e.to_string()))?;
        
        // Convert nanoseconds to seconds and nanoseconds
        let seconds = time_nanos / 1_000_000_000;
        let nanoseconds = (time_nanos % 1_000_000_000) as u32;
        
        let datetime = WasiDatetime {
            seconds,
            nanoseconds,
        };
        
        Ok(datetime)
    }

    pub fn wall_resolution(&mut self) -> Result<WasiDuration> {
        let resolution_nanos = self.elastic_clock.get_resolution(self.system_clock_handle)
            .map_err(|e| WasiClockError::OperationFailed(e.to_string()))?;
        Ok(WasiDuration::from_nanos(resolution_nanos))
    }
}

impl Default for WasiClock {
    fn default() -> Self {
        Self::new().expect("Failed to create WasiClock")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_clock_creation() {
        let clock = WasiClock::new();
        assert!(clock.is_ok());
    }

    #[test]
    fn test_monotonic_clock() {
        let mut clock = WasiClock::new().unwrap();
        
        let now = clock.monotonic_now().unwrap();
        let resolution = clock.monotonic_resolution().unwrap();
        
        assert!(now.as_nanos() > 0);
        assert!(resolution.as_nanos() > 0);
    }

    #[test]
    fn test_wall_clock() {
        let mut clock = WasiClock::new().unwrap();
        
        let now = clock.wall_now().unwrap();
        let resolution = clock.wall_resolution().unwrap();
        
        assert!(now.seconds > 0);
        assert!(resolution.as_secs() > 0);
    }
} 