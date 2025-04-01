#[repr(C)]
#[derive(Debug, Default)]
struct SnpGuestRequestIoctl {
    msg_version: u8,
    req_data: u64,
    resp_data: u64,
    exitinfo2: u64,
}

#[repr(C)]
#[derive(Debug, Default)]
struct SnpReportReq {
    user_data: [u8; 64],
    vmpl: u32,
    rsvd: [u8; 28],
}

#[repr(C)]
#[derive(Debug, Default)]
struct SnpReportResp {
    data: [u8; 4000],
}

// SEV IOCTL commands
const SNP_GUEST_REQ_IOC_TYPE: u8 = b'S';
const SNP_GET_REPORT: Ioctl<WriteRead, &SnpGuestRequestIoctl> = unsafe { 
    Ioctl::write_read((SNP_GUEST_REQ_IOC_TYPE as u32) << 8 | 0x2, std::mem::size_of::<SnpGuestRequestIoctl>() as u32)
};

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
        println!("Struct size: {} bytes", std::mem::size_of::<SnpGuestRequestIoctl>());
        println!("Struct alignment: {} bytes", std::mem::align_of::<SnpGuestRequestIoctl>());
        
        // Create request and response structures
        let mut req = SnpReportReq::default();
        let mut resp = SnpReportResp::default();
        
        // Create ioctl request
        let mut ioctl_req = SnpGuestRequestIoctl {
            msg_version: 1,
            req_data: &req as *const _ as u64,
            resp_data: &mut resp as *mut _ as u64,
            exitinfo2: 0,
        };
        
        // Try to get SNP report
        match SNP_GET_REPORT.ioctl(file, &mut ioctl_req) {
            Ok(_) => {
                println!("Successfully got SNP report");
                println!("Response data length: {} bytes", resp.data.len());
                println!("Exit info 2: 0x{:x}", ioctl_req.exitinfo2);
                Ok(true)
            }
            Err(err) => {
                println!("Failed to get SNP report: {}", err);
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