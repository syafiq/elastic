# Getting Started with WASI Compliance

This guide will help you implement WASI compliance in your ELASTIC project step by step.

## Prerequisites

1. **Rust Toolchain**: Latest stable Rust
2. **WASI Runtime**: Install Wasmtime for testing
3. **WASI Dependencies**: Add WASI crates to your project

## Step 1: Add WASI Dependencies

Update your workspace `Cargo.toml` to include WASI dependencies:

```toml
[workspace.dependencies]
wasi = "0.2"
wasi-crypto = "0.2"
wasi-filesystem = "0.2"
wasi-clocks = "0.2"
wasi-sockets = "0.2"
```

## Step 2: Create WASI-Compliant Wrapper

Start with the clock interface as it's the simplest. Create a new crate `wasi-clock`:

```bash
mkdir crates/wasi-clock
cd crates/wasi-clock
```

### Cargo.toml for wasi-clock

```toml
[package]
name = "wasi-clock"
version = "0.1.0"
edition = "2021"

[dependencies]
elastic-clock = { path = "../elastic-clock" }
wasi = "0.2"
thiserror = "1.0"

[features]
sevsnp = ["elastic-clock/sevsnp"]
linux = ["elastic-clock/linux"]
wasi = ["elastic-clock/wasi"]
```

### Implementation (src/lib.rs)

```rust
use elastic_clock::ClockContext;
use wasi::clocks::{monotonic_clock, wall_clock};
use wasi::clocks::types::{Datetime, Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WasiClockError {
    #[error("ELASTIC clock error: {0}")]
    ElasticError(#[from] elastic_clock::Error),
}

pub type Result<T> = std::result::Result<T, WasiClockError>;

pub struct WasiClock {
    elastic_clock: ClockContext,
}

impl WasiClock {
    pub fn new() -> Result<Self> {
        let elastic_clock = ClockContext::new()?;
        Ok(Self { elastic_clock })
    }
}

impl monotonic_clock::Host for WasiClock {
    fn now(&mut self) -> Result<Instant> {
        let time_nanos = self.elastic_clock.read_monotonic()? * 1_000_000;
        Ok(Instant::from_nanos(time_nanos))
    }

    fn resolution(&mut self) -> Result<Duration> {
        Ok(Duration::from_nanos(1_000)) // 1 microsecond
    }
}

impl wall_clock::Host for WasiClock {
    fn now(&mut self) -> Result<Datetime> {
        let unix_seconds = self.elastic_clock.read_current_time()?;
        Ok(Datetime {
            seconds: unix_seconds,
            nanoseconds: 0,
        })
    }

    fn resolution(&mut self) -> Result<Duration> {
        Ok(Duration::from_secs(1))
    }
}
```

## Step 3: Test Your WASI Implementation

Create a test example:

```rust
// examples/wasi-clock-example/src/main.rs
use wasi_clock::WasiClock;
use wasi::clocks::monotonic_clock::Host as MonotonicHost;
use wasi::clocks::wall_clock::Host as WallHost;

fn main() {
    let mut clock = WasiClock::new().expect("Failed to create WASI clock");

    // Test monotonic clock
    let now = clock.now().unwrap();
    let resolution = clock.resolution().unwrap();
    
    println!("Monotonic time: {} ns", now.as_nanos());
    println!("Resolution: {} ns", resolution.as_nanos());

    // Test wall clock
    let wall_now = clock.now().unwrap();
    println!("Wall time: {} seconds since epoch", wall_now.seconds);
}
```

## Step 4: Build and Test

```bash
# Build the WASI clock crate
cargo build -p wasi-clock

# Build the example
cargo build -p wasi-clock-example

# Test on native platform
cargo run -p wasi-clock-example

# Build for WASI target
cargo build -p wasi-clock-example --target wasm32-wasi

# Test with Wasmtime
wasmtime target/wasm32-wasi/debug/wasi-clock-example.wasm
```

## Step 5: Implement Other Interfaces

Follow the same pattern for other interfaces:

### Filesystem Interface

```rust
// crates/wasi-filesystem/src/lib.rs
use elastic_file::FileContext;
use wasi::filesystem::types::*;

pub struct WasiFilesystem {
    elastic_file: FileContext,
}

impl filesystem::Host for WasiFilesystem {
    fn create_directory_at(&mut self, fd: Descriptor, path: String) -> Result<(), ErrorCode> {
        // Map to ELASTIC file operations
        self.elastic_file.mkdir(&path)
            .map_err(|_| ErrorCode::Io)
    }
}
```

### Crypto Interface

```rust
// crates/wasi-crypto/src/lib.rs
use elastic_crypto::ElasticCrypto;
use wasi::crypto::symmetric;

pub struct WasiCrypto {
    elastic_crypto: ElasticCrypto,
}

impl symmetric::Host for WasiCrypto {
    fn generate_key(&mut self, algorithm: SymmetricAlgorithm) -> Result<SymmetricKey, CryptoError> {
        // Map to ELASTIC crypto operations
        let config = KeyConfig {
            key_type: KeyType::Symmetric,
            key_size: 256,
            secure_storage: true,
        };
        
        let handle = self.elastic_crypto.generate_key(config)?;
        Ok(SymmetricKey::from(handle))
    }
}
```

## Step 6: Add SEV-SNP Extensions

Create WASI extensions for SEV-SNP specific features:

```rust
// crates/wasi-sevsnp/src/lib.rs
use wasi::sevsnp;

pub struct WasiSevSnp;

impl sevsnp::Host for WasiSevSnp {
    fn is_sevsnp_environment(&mut self) -> bool {
        std::env::var("ELASTIC_SEV_SNP").map(|v| v == "1").unwrap_or(false)
    }

    fn get_attestation_report(&mut self) -> Result<Vec<u8>, SevSnpError> {
        // Implement SEV-SNP attestation
        todo!("Implement SEV-SNP attestation")
    }
}
```

## Step 7: Update Build Configuration

Update your main `Cargo.toml` to support WASI targets:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasi = "0.2"
wasi-crypto = "0.2"
wasi-filesystem = "0.2"
wasi-clocks = "0.2"
wasi-sockets = "0.2"

[features]
wasi = ["wasi-crypto", "wasi-filesystem", "wasi-clocks", "wasi-sockets"]
sevsnp = ["wasi-sevsnp"]
```

## Step 8: Create Migration Guide

Document how users can migrate from ELASTIC interfaces to WASI:

```markdown
## Migration Guide

### From ELASTIC Clock to WASI Clock

**Before:**
```rust
use elastic_clock::ClockContext;

let clock = ClockContext::new()?;
let time = clock.read_current_time()?;
```

**After:**
```rust
use wasi_clock::WasiClock;
use wasi::clocks::wall_clock::Host;

let mut clock = WasiClock::new()?;
let datetime = clock.now()?;
let time = datetime.seconds;
```

### Benefits of Migration

1. **Standard Compliance**: Follow official WASI specifications
2. **Runtime Compatibility**: Work with any WASI-compliant runtime
3. **Ecosystem Integration**: Use existing WASI tools and libraries
4. **Future-Proofing**: Align with WebAssembly ecosystem direction
```

## Step 9: Testing Strategy

Create comprehensive tests for WASI compliance:

```rust
#[cfg(test)]
mod wasi_compliance_tests {
    use super::*;

    #[test]
    fn test_wasi_clock_compliance() {
        let mut clock = WasiClock::new().unwrap();
        
        // Test monotonic clock
        let now = clock.now().unwrap();
        let resolution = clock.resolution().unwrap();
        
        assert!(now.as_nanos() > 0);
        assert!(resolution.as_nanos() > 0);
    }

    #[test]
    fn test_wasi_filesystem_compliance() {
        let mut fs = WasiFilesystem::new().unwrap();
        
        // Test filesystem operations
        let result = fs.create_directory_at(Descriptor::from(0), "test".to_string());
        assert!(result.is_ok());
    }
}
```

## Step 10: Performance Optimization

Optimize your WASI implementations:

```rust
impl WasiClock {
    // Cache the clock context for better performance
    pub fn new() -> Result<Self> {
        let elastic_clock = ClockContext::new()?;
        Ok(Self { 
            elastic_clock,
            cached_time: None,
            cache_valid_until: 0,
        })
    }

    fn get_cached_time(&mut self) -> Result<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        if self.cached_time.is_none() || now > self.cache_valid_until {
            self.cached_time = Some(self.elastic_clock.read_current_time()?);
            self.cache_valid_until = now + 1; // Cache for 1 second
        }
        
        Ok(self.cached_time.unwrap())
    }
}
```

## Next Steps

1. **Implement Remaining Interfaces**: Follow the same pattern for filesystem, crypto, and sockets
2. **Add SEV-SNP Extensions**: Create WASI proposals for SEV-SNP specific features
3. **Performance Testing**: Benchmark WASI implementations against native ELASTIC
4. **Documentation**: Update all documentation to reflect WASI compliance
5. **Community Feedback**: Share your WASI implementations with the community

## Resources

- [WASI Specification](https://wasi.dev/)
- [WASI GitHub Repository](https://github.com/WebAssembly/WASI)
- [Wasmtime Runtime](https://wasmtime.dev/)
- [WASI Component Model](https://component-model.bytecodealliance.org/)

This approach allows you to maintain your SEV-SNP security features while becoming WASI-compliant and integrating with the broader WebAssembly ecosystem. 