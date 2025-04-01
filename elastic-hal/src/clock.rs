use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use sev::firmware::host::Firmware;
use std::path::PathBuf;
use std::fs;
use std::env;
use std::io;
use std::os::unix::fs::MetadataExt;

#[derive(Debug, Error)]
pub enum ClockError {
    #[error("Clock not initialized")]
    NotInitialized,
    
    #[error("SEV firmware error: {0}")]
    FirmwareError(String),
    
    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("SEV not available: {0}")]
    SevNotAvailable(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub struct Clock {
    firmware: Option<Firmware>,
}

impl Clock {
    pub fn new() -> Self {
        Self { firmware: None }
    }

    pub fn init(&mut self) -> Result<(), ClockError> {
        // Print diagnostic information
        println!("Checking SEV device paths:");
        let possible_paths = [
            "/dev/sev-guest",
            "/dev/sev",
            "/dev/sev/guest",
            "/dev/sev/guest/0",
        ];

        for path in &possible_paths {
            let path_buf = PathBuf::from(path);
            println!("Checking {}: {}", path, if path_buf.exists() { "exists" } else { "does not exist" });
            if path_buf.exists() {
                let metadata = fs::metadata(path_buf)?;
                println!("  Permissions: {:?}", metadata.permissions());
                println!("  File type: {:?}", metadata.file_type());
                println!("  Size: {} bytes", metadata.size());
            }
        }

        // Print environment information
        println!("\nEnvironment information:");
        println!("SEV_DEVICE: {:?}", env::var("SEV_DEVICE"));
        println!("SEV_GUEST_DEVICE: {:?}", env::var("SEV_GUEST_DEVICE"));
        println!("SEV_GUEST_PATH: {:?}", env::var("SEV_GUEST_PATH"));

        // Try to access the device directly
        println!("\nTrying to access /dev/sev-guest directly...");
        match fs::File::open("/dev/sev-guest") {
            Ok(file) => {
                println!("Successfully opened /dev/sev-guest");
                println!("File descriptor: {:?}", file);
            }
            Err(e) => {
                println!("Failed to open /dev/sev-guest: {}", e);
                println!("Error kind: {:?}", e.kind());
            }
        }

        // Try different approaches to initialize SEV
        println!("\nTrying to initialize SEV...");
        
        // First try: Set environment variable and try
        env::set_var("SEV_DEVICE", "/dev/sev-guest");
        match Firmware::open() {
            Ok(firmware) => {
                self.firmware = Some(firmware);
                println!("Successfully initialized SEV firmware using SEV_DEVICE");
                Ok(())
            }
            Err(e) => {
                println!("First attempt failed: {}", e);
                println!("Error details: {:?}", e);
                
                // Second try: Try without environment variable
                env::remove_var("SEV_DEVICE");
                match Firmware::open() {
                    Ok(firmware) => {
                        self.firmware = Some(firmware);
                        println!("Successfully initialized SEV firmware without environment variable");
                        Ok(())
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        println!("SEV initialization error: {}", error_msg);
                        println!("Error details: {:?}", e);
                        if error_msg.contains("No such file or directory") {
                            println!("SEV device not found. Please check if SEV is properly configured.");
                            Err(ClockError::SevNotAvailable(format!("SEV firmware not available: {}", error_msg)))
                        } else {
                            Err(ClockError::FirmwareError(error_msg))
                        }
                    }
                }
            }
        }
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
        match clock.init() {
            Ok(_) => {
                // If we're in an SEV environment, initialization should succeed
                assert!(clock.firmware.is_some());
            }
            Err(ClockError::SevNotAvailable(_)) => {
                // If we're not in an SEV environment, this is expected
                assert!(clock.firmware.is_none());
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
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
        match clock.init() {
            Ok(_) => {
                let ms_time = clock.get_time_ms()?;
                let us_time = clock.get_time_us()?;

                assert!(ms_time > 0);
                assert!(us_time > 0);
                assert!(us_time >= ms_time * 1000);
                Ok(())
            }
            Err(ClockError::SevNotAvailable(_)) => {
                // Skip test if not in SEV environment
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[tokio::test]
    async fn test_sleep_functions() -> Result<(), ClockError> {
        let mut clock = Clock::new();
        match clock.init() {
            Ok(_) => {
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
            Err(ClockError::SevNotAvailable(_)) => {
                // Skip test if not in SEV environment
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
} 