use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;
use iocuddle::{Ioctl, WriteRead};
use std::fs::File;

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

    #[error("IOCTL error: {0}")]
    IoctlError(String),
}

// SEV IOCTL commands
const SEV_IOCTL_BASE: u64 = 0xAE00;
const SEV_IOCTL_GET_TIME: u64 = SEV_IOCTL_BASE + 1;

// Define the ioctl command
const SEV_GET_TIME: Ioctl<WriteRead, &mut u64> = unsafe { Ioctl::classic(SEV_IOCTL_GET_TIME) };

pub struct Clock {
    sev_fd: Option<File>,
}

impl Clock {
    pub fn new() -> Self {
        Self { sev_fd: None }
    }

    pub fn init(&mut self) -> Result<(), ClockError> {
        println!("Initializing Clock Interface...");
        
        // Check multiple possible SEV device paths
        let sev_paths = [
            "/dev/sev-guest",
            "/dev/sev",
            "/dev/sev/guest",
            "/dev/sev/guest/0"
        ];

        // Print diagnostic information about environment variables
        println!("\nSEV Environment Variables:");
        println!("SEV_DEVICE: {}", std::env::var("SEV_DEVICE").unwrap_or_else(|_| "not set".to_string()));
        println!("SEV_DEVICE_PATH: {}", std::env::var("SEV_DEVICE_PATH").unwrap_or_else(|_| "not set".to_string()));
        println!("SEV_DEVICE_FD: {}", std::env::var("SEV_DEVICE_FD").unwrap_or_else(|_| "not set".to_string()));

        // Print diagnostic information about device paths
        println!("\nSEV Device Paths:");
        for path in &sev_paths {
            match std::fs::metadata(path) {
                Ok(metadata) => {
                    println!("{} exists with permissions: {:o}", path, metadata.mode() & 0o777);
                    if let Ok(_) = File::open(path) {
                        println!("  Can open for reading");
                        if let Ok(_) = File::options().read(true).write(true).open(path) {
                            println!("  Can open for read/write");
                        }
                    }
                }
                Err(e) => println!("{} does not exist: {}", path, e),
            }
        }

        // Try to open the SEV device
        match File::options().read(true).write(true).open("/dev/sev-guest") {
            Ok(file) => {
                println!("\nSuccessfully opened /dev/sev-guest");
                self.sev_fd = Some(file);
                Ok(())
            }
            Err(e) => {
                println!("\nFailed to open /dev/sev-guest: {}", e);
                Err(ClockError::SevNotAvailable(format!("Failed to open SEV device: {}", e)))
            }
        }
    }

    pub fn get_time(&self) -> Result<u64, ClockError> {
        if let Some(file) = &self.sev_fd {
            let fd = file.as_raw_fd();
            let mut time: u64 = 0;
            
            // Try to get time from SEV device
            match unsafe { SEV_GET_TIME.read(fd, &mut time) } {
                Ok(_) => {
                    println!("Successfully got time from SEV device: {}", time);
                    Ok(time)
                }
                Err(e) => {
                    println!("Failed to get time from SEV device: {}", e);
                    // Fallback to system time
                    Ok(SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| ClockError::SystemTimeError(e))?
                        .as_secs())
                }
            }
        } else {
            // Fallback to system time if SEV is not available
            Ok(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| ClockError::SystemTimeError(e))?
                .as_secs())
        }
    }

    pub fn set_time(&self, _time: u64) -> Result<(), ClockError> {
        if self.sev_fd.is_some() {
            println!("Setting time in SEV environment...");
            // TODO: Implement SEV-specific time setting
        } else {
            println!("SEV not available, skipping time setting");
        }
        Ok(())
    }

    pub fn get_timezone(&self) -> Result<String, ClockError> {
        // TODO: Implement timezone handling
        Ok("UTC".to_string())
    }

    pub fn set_timezone(&self, _timezone: &str) -> Result<(), ClockError> {
        // TODO: Implement timezone setting
        Ok(())
    }

    pub fn get_time_ms(&self) -> Result<u64, ClockError> {
        if self.sev_fd.is_none() {
            return Err(ClockError::NotInitialized);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        Ok(now.as_millis() as u64)
    }

    pub fn get_time_us(&self) -> Result<u64, ClockError> {
        if self.sev_fd.is_none() {
            return Err(ClockError::NotInitialized);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        Ok(now.as_micros() as u64)
    }

    pub async fn sleep_ms(&self, ms: u32) -> Result<(), ClockError> {
        if self.sev_fd.is_none() {
            return Err(ClockError::NotInitialized);
        }

        tokio::time::sleep(Duration::from_millis(ms as u64)).await;
        Ok(())
    }

    pub async fn sleep_us(&self, us: u32) -> Result<(), ClockError> {
        if self.sev_fd.is_none() {
            return Err(ClockError::NotInitialized);
        }

        tokio::time::sleep(Duration::from_micros(us as u64)).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_initialization() {
        let mut clock = Clock::new();
        match clock.init() {
            Ok(_) => println!("Clock initialization successful"),
            Err(e) => println!("Clock initialization failed: {}", e),
        }
    }

    #[test]
    fn test_get_time() {
        let mut clock = Clock::new();
        if clock.init().is_ok() {
            match clock.get_time() {
                Ok(time) => println!("Current time: {}", time),
                Err(e) => println!("Failed to get time: {}", e),
            }
        }
    }

    #[test]
    fn test_set_time() {
        let mut clock = Clock::new();
        if clock.init().is_ok() {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            match clock.set_time(current_time) {
                Ok(_) => println!("Time set successfully"),
                Err(e) => println!("Failed to set time: {}", e),
            }
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