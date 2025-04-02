# ELASTIC HAL Implementation

This repository contains the implementation of the Hardware Abstraction Layer (HAL) for the ELASTIC project, focusing on providing a secure and efficient interface for hardware access in WebAssembly environments.

## Features

### Clock Interface
- Current time reading in seconds since UNIX epoch
- Timezone information retrieval
- Monotonic clock for precise time measurements
- Error handling for system time and timezone operations
- WebAssembly Interface Types (WIT) support for language interoperability

### File Interface
- Secure file operations with container-based isolation
- Support for both regular and encrypted file storage
- File metadata access and manipulation
- Directory listing and file management
- AES-GCM encryption for secure storage
- WebAssembly Interface Types (WIT) support for language interoperability

## WIT Interface Support

### Clock Interface (clock.wit)
```wit
package elastic:clock@0.1.0;

interface types {
    variant clock-error {
        system-time-error(string),
    }
}

interface clock {
    use types.{clock-error};

    read-current-time: func() -> result<u64, clock-error>;
    read-timezone: func() -> result<string, clock-error>;
    start-monotonic: func() -> result<_, clock-error>;
    stop-monotonic: func() -> result<_, clock-error>;
    read-monotonic: func() -> result<u64, clock-error>;
}

world clock-impl {
    export clock;
}
```

### File Interface (file.wit)
```wit
package elastic:file@0.1.0;

interface types {
    variant file-error {
        not-found(string),
        permission-denied(string),
        already-exists(string),
        invalid-operation(string),
        encryption-error(string),
        decryption-error(string),
        io-error(string),
    }

    enum file-mode {
        read,
        write,
        append,
        read-write,
    }

    enum file-type {
        regular,
        directory,
        symbolic-link,
    }

    record file-metadata {
        name: string,
        size: u64,
        file-type: file-type,
        created: u64,
        modified: u64,
        accessed: u64,
        permissions: u32,
    }
}

interface file {
    use types.{file-error, file-mode, file-type, file-metadata};

    open-container: func(path: string, mode: file-mode) -> result<u32, file-error>;
    close-container: func(handle: u32) -> result<_, file-error>;
    read-file: func(handle: u32, path: string) -> result<list<u8>, file-error>;
    write-file: func(handle: u32, path: string, data: list<u8>) -> result<_, file-error>;
    delete-file: func(handle: u32, path: string) -> result<_, file-error>;
    list-files: func(handle: u32, path: string) -> result<list<string>, file-error>;
    get-metadata: func(handle: u32, path: string) -> result<file-metadata, file-error>;
    load-key: func(handle: u32, key: list<u8>) -> result<_, file-error>;
    remove-key: func(handle: u32) -> result<_, file-error>;
    is-encrypted: func(handle: u32, path: string) -> result<bool, file-error>;
}

world file-impl {
    export file;
}
```

## Usage Examples

### Clock Interface

```rust
use elastic::clock::Clock;

let mut clock = Clock::new();

// Read current time
match clock.read_current_time() {
    Ok(time) => println!("Current time (seconds since epoch): {}", time),
    Err(e) => eprintln!("Error reading current time: {}", e),
}

// Start monotonic clock
clock.start_monotonic().unwrap();

// ... do some work ...

// Read elapsed time
match clock.read_monotonic() {
    Ok(elapsed) => println!("Elapsed time (seconds): {}", elapsed),
    Err(e) => eprintln!("Error reading monotonic time: {}", e),
}
```

### File Interface

```rust
use elastic::file::{FileSystem, FileMode};

let fs = FileSystem::new();

// Open a container
let handle = fs.open_container("/path/to/container", FileMode::ReadWrite).unwrap();

// Write a file
let data = b"Hello, World!";
fs.write_file(handle, "test.txt", data).unwrap();

// Read the file
let contents = fs.read_file(handle, "test.txt").unwrap();
assert_eq!(contents, data);

// Enable encryption
let key = vec![1u8; 32]; // Use a secure key in production
fs.load_key(handle, &key).unwrap();

// Write encrypted data
let secret_data = b"Secret message";
fs.write_file(handle, "secret.txt", secret_data).unwrap();

// Read encrypted data
let decrypted = fs.read_file(handle, "secret.txt").unwrap();
assert_eq!(decrypted, secret_data);

// Close the container
fs.close_container(handle).unwrap();
```

## Testing

The project includes comprehensive test suites for both interfaces:

### Clock Interface Tests
```bash
./tests/run_tests.sh
```

### File Interface Tests
```bash
./tests/run_file_tests.sh
```

## Dependencies

- Rust 1.70.0 or later
- wit-bindgen 0.11.0
- aes-gcm 0.10.3 (for file encryption)
- tempfile 3.8.1 (for testing)

## Building

```bash
cargo build
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
