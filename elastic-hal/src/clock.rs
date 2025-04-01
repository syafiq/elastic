use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use sev::firmware::host::Firmware;
use sev::firmware::Error as SevError;

#[derive(Debug, Error)]
pub enum ClockError {
    #[error("Clock not initialized")]
    NotInitialized,
    
    #[error("SEV firmware error: {0}")]
    FirmwareError(#[from] SevError),
    
    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}

pub struct Clock {
    firmware: Option<Firmware>,
}

impl Clock {
    pub fn new() -> Self {
        Self { firmware: None }
    }

    pub fn init(&mut self) -> Result<(), ClockError> {
        let firmware = Firmware::open()?;
        self.firmware = Some(firmware);
        Ok(())
    }

    pub fn get_time_ms(&self) -> Result<u64, ClockError> {
        if self.firmware.is_none() {
            return Err(ClockError::NotInitialized);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        Ok(now.as_millis() as u64)
    }

    pub fn get_time_us(&self) -> Result<u64, ClockError> {
        if self.firmware.is_none() {
            return Err(ClockError::NotInitialized);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        Ok(now.as_micros() as u64)
    }

    pub async fn sleep_ms(&self, ms: u32) -> Result<(), ClockError> {
        if self.firmware.is_none() {
            return Err(ClockError::NotInitialized);
        }

        tokio::time::sleep(Duration::from_millis(ms as u64)).await;
        Ok(())
    }

    pub async fn sleep_us(&self, us: u32) -> Result<(), ClockError> {
        if self.firmware.is_none() {
            return Err(ClockError::NotInitialized);
        }

        tokio::time::sleep(Duration::from_micros(us as u64)).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clock_initialization() {
        let mut clock = Clock::new();
        assert!(clock.init().is_ok());
    }

    #[tokio::test]
    async fn test_uninitialized_clock() {
        let clock = Clock::new();
        assert!(matches!(clock.get_time_ms(), Err(ClockError::NotInitialized)));
        assert!(matches!(clock.get_time_us(), Err(ClockError::NotInitialized)));
        assert!(matches!(clock.sleep_ms(100).await, Err(ClockError::NotInitialized)));
        assert!(matches!(clock.sleep_us(100).await, Err(ClockError::NotInitialized)));
    }

    #[tokio::test]
    async fn test_time_functions() -> Result<(), ClockError> {
        let mut clock = Clock::new();
        clock.init()?;

        let ms_time = clock.get_time_ms()?;
        let us_time = clock.get_time_us()?;

        assert!(ms_time > 0);
        assert!(us_time > 0);
        assert!(us_time >= ms_time * 1000);

        Ok(())
    }

    #[tokio::test]
    async fn test_sleep_functions() -> Result<(), ClockError> {
        let mut clock = Clock::new();
        clock.init()?;

        let start_ms = clock.get_time_ms()?;
        clock.sleep_ms(100).await?;
        let end_ms = clock.get_time_ms()?;
        assert!(end_ms - start_ms >= 100);

        let start_us = clock.get_time_us()?;
        clock.sleep_us(1000).await?;
        let end_us = clock.get_time_us()?;
        assert!(end_us - start_us >= 1000);

        Ok(())
    }
} 