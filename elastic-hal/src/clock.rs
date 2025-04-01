use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;
use std::fs::File;
use libc::{ioctl, c_void};

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
const SEV_IOCTL_GET_CAPABILITIES: u64 = SEV_IOCTL_BASE + 0x01;

#[repr(C)]
struct SevCapabilities {
    major: u32,
    minor: u32,
    build: u32,
    _reserved: u32,
}

pub struct Clock {
    sev_fd: Option<File>,
    is_sev: bool,
}

impl Clock {
    pub fn new() -> Self {
        Self { 
            sev_fd: None,
            is_sev: false,
        }
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
                
                // Check if we're in an SEV environment
                if let Ok(true) = self.check_sev_environment() {
                    println!("Running in SEV environment");
                    self.is_sev = true;
                } else {
                    println!("Not running in SEV environment");
                    self.is_sev = false;
                }
                
                Ok(())
            }
            Err(e) => {
                println!("\nFailed to open /dev/sev-guest: {}", e);
                self.is_sev = false;
                Err(ClockError::SevNotAvailable(format!("Failed to open SEV device: {}", e)))
            }
        }
    }

    fn check_sev_environment(&self) -> Result<bool, ClockError> {
        if let Some(file) = &self.sev_fd {
            let fd = file.as_raw_fd();
            let mut caps = SevCapabilities {
                major: 0,
                minor: 0,
                build: 0,
                _reserved: 0,
            };
            
            // Print diagnostic information about the ioctl command
            println!("\nAttempting SEV ioctl:");
            println!("Command: 0x{:X}", SEV_IOCTL_GET_CAPABILITIES);
            println!("Struct size: {} bytes", std::mem::size_of::<SevCapabilities>());
            println!("Struct alignment: {} bytes", std::mem::align_of::<SevCapabilities>());
            
            // Try to get capabilities
            let result = unsafe {
                ioctl(
                    fd,
                    SEV_IOCTL_GET_CAPABILITIES,
                    &mut caps as *mut SevCapabilities as *mut c_void
                )
            };

            if result == 0 {
                println!("SEV Capabilities:");
                println!("  Version: {}.{}", caps.major, caps.minor);
                println!("  Build: {}", caps.build);
                Ok(true)
            } else {
                let err = io::Error::last_os_error();
                println!("Failed to get SEV capabilities: {}", err);
                println!("Error code: {}", err.kind());
                println!("Error message: {}", err.to_string());
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub fn get_time(&self) -> Result<u64, ClockError> {
        // For now, always use system time
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ClockError::SystemTimeError(e))?
            .as_secs())
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