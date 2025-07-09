# ELASTIC WASI Compliance Roadmap

## Current State Analysis

### Existing ELASTIC Interfaces vs. WASI Standards

| ELASTIC Interface | Current Implementation | WASI Equivalent | Compliance Status |
|------------------|----------------------|-----------------|-------------------|
| `elastic:clock` | Custom WIT interface | `wasi:clocks` | ❌ Not compliant |
| `elastic:file` | Custom WIT interface | `wasi:filesystem` | ❌ Not compliant |
| `elastic:crypto` | Custom WIT interface | `wasi:random` + `wasi:crypto` | ❌ Not compliant |
| `elastic:tls` | Custom WIT interface | `wasi:sockets` + TLS extensions | ❌ Not compliant |

## Phase 1: WASI Interface Mapping

### 1. Clock Interface (`elastic:clock` → `wasi:clocks`)

**Current ELASTIC Interface:**
```wit
package elastic:clock@0.1.0;

interface clock {
    read-current-time: func() -> result<u64, clock-error>;
    read-timezone: func() -> result<string, clock-error>;
    start-monotonic: func() -> result<_, clock-error>;
    stop-monotonic: func() -> result<u64, clock-error>;
    read-monotonic: func() -> result<u64, clock-error>;
}
```

**Target WASI Interface:**
```wit
package wasi:clocks@0.2.0;

interface monotonic-clock {
    now: func() -> instant;
    resolution: func() -> duration;
    subscribe: func(when: instant, absolute: bool) -> pollable;
}

interface wall-clock {
    now: func() -> datetime;
    resolution: func() -> duration;
}
```

**Migration Strategy:**
1. Create WASI-compliant wrapper around existing SEV-SNP clock implementation
2. Map `read-current-time` → `wall-clock.now`
3. Map `read-monotonic` → `monotonic-clock.now`
4. Implement `monotonic-clock.subscribe` for async time operations

### 2. File Interface (`elastic:file` → `wasi:filesystem`)

**Current ELASTIC Interface:**
```wit
package elastic:file@0.1.0;

interface file {
    open: func(path: string, mode: file-mode) -> result<u32, file-error>;
    close: func(handle: u32) -> result<_, file-error>;
    read: func(handle: u32, size: u32) -> result<list<u8>, file-error>;
    write: func(handle: u32, data: list<u8>) -> result<_, file-error>;
    // ... more operations
}
```

**Target WASI Interface:**
```wit
package wasi:filesystem@0.2.0;

interface filesystem {
    create-directory-at: func(fd: descriptor, path: string) -> result<_, error-code>;
    stat: func(fd: descriptor, path-flags: path-flags, path: string) -> result<descriptor-stat, error-code>;
    // ... more operations
}

interface descriptor {
    read: func(len: u64) -> result<list<u8>, error-code>;
    write: func(buf: list<u8>) -> result<u64, error-code>;
    // ... more operations
}
```

**Migration Strategy:**
1. Implement WASI filesystem interface using existing SEV-SNP file operations
2. Add encryption layer as WASI extension
3. Map file handles to WASI descriptors
4. Implement proper error code mapping

### 3. Crypto Interface (`elastic:crypto` → `wasi:random` + `wasi:crypto`)

**Current ELASTIC Interface:**
```wit
package elastic:crypto@0.1.0;

interface crypto {
    generate-key: func(config: key-config) -> result<u32, crypto-error>;
    encrypt: func(handle: u32, data: list<u8>) -> result<list<u8>, crypto-error>;
    decrypt: func(handle: u32, data: list<u8>) -> result<list<u8>, crypto-error>;
    // ... more operations
}
```

**Target WASI Interfaces:**
```wit
package wasi:random@0.2.0;

interface random {
    get-random-bytes: func(len: u64) -> list<u8>;
    get-random-u64: func() -> u64;
}

package wasi:crypto@0.2.0;

interface symmetric {
    generate-key: func(algorithm: symmetric-algorithm) -> symmetric-key;
    encrypt: func(key: symmetric-key, data: list<u8>) -> result<list<u8>, crypto-error>;
    decrypt: func(key: symmetric-key, data: list<u8>) -> result<list<u8>, crypto-error>;
}
```

**Migration Strategy:**
1. Implement `wasi:random` using SEV-SNP hardware RNG
2. Implement `wasi:crypto` using SEV-SNP hardware crypto
3. Create compatibility layer for existing ELASTIC crypto interface
4. Add SEV-SNP specific extensions as WASI proposals

### 4. TLS Interface (`elastic:tls` → `wasi:sockets` + TLS)

**Current ELASTIC Interface:**
```wit
package elastic:tls@0.1.0;

interface tls {
    connect: func(handle: u32, hostname: string, port: u16) -> result<u32, tls-error>;
    write: func(conn: u32, data: list<u8>) -> result<u32, tls-error>;
    read: func(conn: u32, max-size: u32) -> result<list<u8>, tls-error>;
    // ... more operations
}
```

**Target WASI Interface:**
```wit
package wasi:sockets@0.2.0;

interface tcp-socket {
    connect: func(network: network, remote-address: ip-socket-address) -> result<_, error-code>;
    write: func(data: list<u8>) -> result<u64, error-code>;
    read: func(len: u64) -> result<list<u8>, error-code>;
    // ... more operations
}
```

**Migration Strategy:**
1. Implement WASI sockets interface
2. Add TLS as WASI extension or use existing TLS libraries
3. Map SEV-SNP hardware acceleration to WASI crypto interface
4. Implement proper network abstraction

## Phase 2: Implementation Strategy

### Step 1: Create WASI-Compliant Wrappers

Create new crates that implement WASI interfaces while using existing ELASTIC implementations:

```
crates/
├── wasi-clock/          # WASI clocks implementation using elastic-clock
├── wasi-filesystem/     # WASI filesystem using elastic-file
├── wasi-crypto/         # WASI crypto using elastic-crypto
└── wasi-sockets/        # WASI sockets with TLS support
```

### Step 2: Implement WASI Interfaces

For each interface, create WASI-compliant implementations:

```rust
// Example: wasi-clock implementation
use wasi::clocks::{monotonic_clock, wall_clock};
use elastic_clock::ClockContext;

pub struct WasiClock {
    elastic_clock: ClockContext,
}

impl monotonic_clock::Host for WasiClock {
    fn now(&mut self) -> Result<instant::Instant, Error> {
        let time = self.elastic_clock.read_monotonic()?;
        Ok(instant::Instant::from_secs(time))
    }
    
    fn resolution(&mut self) -> Result<duration::Duration, Error> {
        // Return SEV-SNP TSC resolution
        Ok(duration::Duration::from_nanos(1)) // 1ns resolution for TSC
    }
}
```

### Step 3: Add SEV-SNP Extensions

Create WASI proposals for SEV-SNP specific features:

```wit
package wasi:sevsnp@0.1.0;

interface sevsnp {
    // SEV-SNP specific extensions
    is-sevsnp-environment: func() -> bool;
    get-attestation-report: func() -> result<list<u8>, error>;
    hardware-encrypt: func(data: list<u8>) -> result<list<u8>, error>;
}
```

### Step 4: Update Build System

Modify Cargo.toml to support WASI targets:

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

## Phase 3: Testing and Validation

### Step 1: WASI Compliance Testing

Create tests that verify WASI interface compliance:

```rust
#[cfg(test)]
mod wasi_compliance_tests {
    use wasi::clocks::monotonic_clock::Host;
    
    #[test]
    fn test_wasi_clock_compliance() {
        let mut clock = WasiClock::new();
        
        // Test WASI interface compliance
        let now = clock.now().unwrap();
        let resolution = clock.resolution().unwrap();
        
        assert!(resolution.as_nanos() > 0);
        assert!(now.as_secs() > 0);
    }
}
```

### Step 2: Cross-Platform Testing

Test WASI implementations across different platforms:

```bash
# Test on standard WASI runtime
wasmtime --wasm-features=threads target/wasm32-wasi/debug/elastic-wasi.wasm

# Test on SEV-SNP with WASI extensions
wasmtime --env ELASTIC_SEV_SNP=1 --wasm-features=threads target/wasm32-wasi/debug/elastic-wasi.wasm
```

## Phase 4: Documentation and Migration

### Step 1: Update Documentation

Update README.md to reflect WASI compliance:

```markdown
## WASI Compliance

ELASTIC now implements WASI-compliant interfaces:

- `wasi:clocks` - Time and clock operations
- `wasi:filesystem` - File system operations with encryption
- `wasi:crypto` - Cryptographic operations with SEV-SNP acceleration
- `wasi:sockets` - Network operations with TLS support

### SEV-SNP Extensions

For SEV-SNP environments, ELASTIC provides additional WASI extensions:

- `wasi:sevsnp` - SEV-SNP specific features
- Hardware-accelerated cryptography
- Secure attestation
- Protected memory operations
```

### Step 2: Migration Guide

Create migration guide for existing users:

```markdown
## Migration from ELASTIC Interfaces to WASI

### Clock Interface Migration

**Before (ELASTIC):**
```rust
use elastic_clock::ClockContext;

let clock = ClockContext::new();
let time = clock.read_current_time()?;
```

**After (WASI):**
```rust
use wasi::clocks::wall_clock::Host;

let clock = WasiClock::new();
let time = clock.now()?;
```

## Implementation Timeline

### Week 1-2: Research and Planning
- [ ] Study WASI 0.2 specifications
- [ ] Create detailed interface mappings
- [ ] Set up WASI development environment

### Week 3-4: Core WASI Implementation
- [ ] Implement `wasi:clocks` interface
- [ ] Implement `wasi:random` interface
- [ ] Create basic WASI compliance tests

### Week 5-6: Filesystem and Crypto
- [ ] Implement `wasi:filesystem` interface
- [ ] Implement `wasi:crypto` interface
- [ ] Add SEV-SNP hardware acceleration

### Week 7-8: Networking and TLS
- [ ] Implement `wasi:sockets` interface
- [ ] Add TLS support
- [ ] Create SEV-SNP extensions

### Week 9-10: Testing and Documentation
- [ ] Comprehensive WASI compliance testing
- [ ] Update documentation
- [ ] Create migration guides

### Week 11-12: Integration and Release
- [ ] Integrate all WASI interfaces
- [ ] Performance optimization
- [ ] Release WASI-compliant version

## Benefits of WASI Compliance

1. **Standardization**: Follow official WASI specifications
2. **Interoperability**: Work with any WASI-compliant runtime
3. **Ecosystem Integration**: Leverage existing WASI tools and libraries
4. **Future-Proofing**: Align with WebAssembly ecosystem direction
5. **Security**: Maintain SEV-SNP security features while being standards-compliant

## Next Steps

1. **Start with Clock Interface**: Implement `wasi:clocks` as it's the simplest
2. **Create WASI Wrapper**: Build WASI-compliant wrapper around existing SEV-SNP implementation
3. **Add SEV-SNP Extensions**: Propose WASI extensions for SEV-SNP specific features
4. **Test Thoroughly**: Ensure compatibility across different WASI runtimes
5. **Document Migration**: Help existing users transition to WASI interfaces

This roadmap provides a clear path to WASI compliance while maintaining the security and performance benefits of your SEV-SNP implementation. 