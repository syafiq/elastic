#[repr(C)]
#[derive(Debug, Default)]
struct SevStatus {
    major: u32,
    minor: u32,
    state: u32,
    _reserved: u32,
}

// SEV IOCTL commands
const SEV: Group = Group::new(0xae);
// Try SNP command number - 0x2
const SEV_GET_STATUS: Ioctl<Read, &SevStatus> = unsafe { SEV.read(0x2) };

pub fn init(&mut self) -> Result<(), ClockError> {
    println!("Initializing Clock Interface...");
    
    // Check multiple possible SEV device paths
    let sev_paths = [
        "/dev/sev",
        "/dev/sev-guest",
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
    let mut found_device = false;
    for path in &sev_paths {
        match std::fs::metadata(path) {
            Ok(metadata) => {
                println!("{} exists with permissions: {:o}", path, metadata.mode() & 0o777);
                if let Ok(_) = File::options().read(true).open(path) {
                    println!("  Can open for reading");
                    if let Ok(file) = File::options().read(true).write(true).open(path) {
                        println!("  Can open for read/write");
                        println!("\nSuccessfully opened {}", path);
                        self.sev_fd = Some(file);
                        found_device = true;
                        break;
                    }
                }
            }
            Err(e) => println!("{} does not exist: {}", path, e),
        }
    }

    if !found_device {
        println!("\nNo SEV device found");
        self.is_sev = false;
        return Ok(());
    }

    // Check if we're in an SEV environment
    match self.check_sev_environment() {
        Ok(true) => {
            println!("Running in SEV environment");
            self.is_sev = true;
        }
        Ok(false) => {
            println!("Not running in SEV environment");
            self.is_sev = false;
        }
        Err(e) => {
            println!("Error checking SEV environment: {}", e);
            self.is_sev = false;
        }
    }

    Ok(())
}

fn check_sev_environment(&self) -> Result<bool, ClockError> {
    if let Some(file) = &self.sev_fd {
        // Print diagnostic information about the ioctl command
        println!("\nAttempting SEV ioctl:");
        println!("Struct size: {} bytes", std::mem::size_of::<SevStatus>());
        println!("Struct alignment: {} bytes", std::mem::align_of::<SevStatus>());
        
        // Try to get status
        match SEV_GET_STATUS.ioctl(file) {
            Ok((_, status)) => {
                println!("SEV Status:");
                println!("  Major: {}", status.major);
                println!("  Minor: {}", status.minor);
                println!("  State: {}", status.state);
                Ok(true)
            }
            Err(err) => {
                println!("Failed to get SEV status: {}", err);
                println!("Error code: {}", err.kind());
                println!("Error message: {}", err.to_string());
                println!("Raw errno: {}", err.raw_os_error().unwrap_or(-1));
                Ok(false)
            }
        }
    } else {
        Ok(false)
    }
} 