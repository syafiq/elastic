use elastic_file::{FileOps, FileError};
use std::io::{self, Write};

#[cfg(feature = "linux")]
struct LinuxFileOps;

#[cfg(feature = "linux")]
impl FileOps for LinuxFileOps {
    fn open(&self, path: &str) -> Result<u32, FileError> {
        // Simulate opening a file
        println!("Opening file: {}", path);
        Ok(1)
    }

    fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError> {
        // Simulate reading from a file
        println!("Reading from handle: {}", handle);
        Ok(buf.len())
    }

    fn write(&self, handle: u32, data: &[u8]) -> Result<usize, FileError> {
        // Simulate writing to a file
        println!("Writing to handle: {}", handle);
        Ok(data.len())
    }

    fn seek(&self, handle: u32, pos: u64) -> Result<u64, FileError> {
        // Simulate seeking in a file
        println!("Seeking handle: {} to position: {}", handle, pos);
        Ok(pos)
    }

    fn metadata(&self, handle: u32) -> Result<(u64, bool, bool), FileError> {
        // Simulate getting file metadata
        println!("Getting metadata for handle: {}", handle);
        Ok((0, true, false))
    }

    fn close(&self, handle: u32) -> Result<(), FileError> {
        // Simulate closing a file
        println!("Closing handle: {}", handle);
        Ok(())
    }
}

#[cfg(feature = "sevsnp")]
struct SevSnpFileOps;

#[cfg(feature = "sevsnp")]
impl FileOps for SevSnpFileOps {
    fn open(&self, path: &str) -> Result<u32, FileError> {
        // Simulate opening a file in SEV-SNP mode
        println!("Opening file in SEV-SNP mode: {}", path);
        Ok(1)
    }

    fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError> {
        // Simulate reading from a file in SEV-SNP mode
        println!("Reading from handle in SEV-SNP mode: {}", handle);
        Ok(buf.len())
    }

    fn write(&self, handle: u32, data: &[u8]) -> Result<usize, FileError> {
        // Simulate writing to a file in SEV-SNP mode
        println!("Writing to handle in SEV-SNP mode: {}", handle);
        Ok(data.len())
    }

    fn seek(&self, handle: u32, pos: u64) -> Result<u64, FileError> {
        // Simulate seeking in a file in SEV-SNP mode
        println!("Seeking handle in SEV-SNP mode: {} to position: {}", handle, pos);
        Ok(pos)
    }

    fn metadata(&self, handle: u32) -> Result<(u64, bool, bool), FileError> {
        // Simulate getting file metadata in SEV-SNP mode
        println!("Getting metadata for handle in SEV-SNP mode: {}", handle);
        Ok((0, true, false))
    }

    fn close(&self, handle: u32) -> Result<(), FileError> {
        // Simulate closing a file in SEV-SNP mode
        println!("Closing handle in SEV-SNP mode: {}", handle);
        Ok(())
    }
}

#[cfg(feature = "wasm")]
struct WasmFileOps;

#[cfg(feature = "wasm")]
impl FileOps for WasmFileOps {
    fn open(&self, path: &str) -> Result<u32, FileError> {
        // Simulate opening a file in WASM mode
        println!("Opening file in WASM mode: {}", path);
        Ok(1)
    }

    fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError> {
        // Simulate reading from a file in WASM mode
        println!("Reading from handle in WASM mode: {}", handle);
        Ok(buf.len())
    }

    fn write(&self, handle: u32, data: &[u8]) -> Result<usize, FileError> {
        // Simulate writing to a file in WASM mode
        println!("Writing to handle in WASM mode: {}", handle);
        Ok(data.len())
    }

    fn seek(&self, handle: u32, pos: u64) -> Result<u64, FileError> {
        // Simulate seeking in a file in WASM mode
        println!("Seeking handle in WASM mode: {} to position: {}", handle, pos);
        Ok(pos)
    }

    fn metadata(&self, handle: u32) -> Result<(u64, bool, bool), FileError> {
        // Simulate getting file metadata in WASM mode
        println!("Getting metadata for handle in WASM mode: {}", handle);
        Ok((0, true, false))
    }

    fn close(&self, handle: u32) -> Result<(), FileError> {
        // Simulate closing a file in WASM mode
        println!("Closing handle in WASM mode: {}", handle);
        Ok(())
    }
}

fn main() {
    println!("File Demo");

    #[cfg(feature = "linux")]
    {
        let file_ops = LinuxFileOps;
        run_demo(&file_ops);
    }

    #[cfg(feature = "sevsnp")]
    {
        let file_ops = SevSnpFileOps;
        run_demo(&file_ops);
    }

    #[cfg(feature = "wasm")]
    {
        let file_ops = WasmFileOps;
        run_demo(&file_ops);
    }
}

fn run_demo(file_ops: &dyn FileOps) {
    // Basic file operations
    let handle = file_ops.open("test.txt").unwrap();
    let mut buf = vec![0u8; 1024];
    let _ = file_ops.read(handle, &mut buf).unwrap();
    let _ = file_ops.write(handle, b"Hello, ELASTIC!").unwrap();
    let _ = file_ops.seek(handle, 0).unwrap();
    let _ = file_ops.metadata(handle).unwrap();
    let _ = file_ops.close(handle).unwrap();

    // Error handling
    match file_ops.open("nonexistent.txt") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {:?}", e),
    }
} 