# ELASTIC Implementation Log

## Project Overview
ELASTIC (Enclave Layer for Secure Time, Information, and Cryptography) is a Hardware Abstraction Layer (HAL) for AMD SEV-SNP VMs. The project aims to provide secure abstractions for time, file operations, and cryptographic functions in a SEV-SNP environment.

## Platform Support Matrix

| Interface | Linux | SEV-SNP | TDX | Notes |
|-----------|-------|---------|-----|-------|
| Clock     | ✅    | ✅      | ❌  | SEV-SNP uses TSC, Linux uses system calls |
| File      | ✅    | ⏳      | ❌  | SEV-SNP implementation in progress |
| Crypto    | ✅    | ⏳      | ❌  | SEV-SNP implementation in progress |
| TLS       | ⏳    | ⏳      | ❌  | Planning phase for both platforms |

Legend:
- ✅: Implemented and tested
- ⏳: In progress or planned
- ❌: Not implemented

## Implementation Progress

### Clock Interface
- ✅ Implemented secure time measurements
  - Used `SystemTime` for current time retrieval
  - Implemented timezone handling with `read_timezone` function
  - Added error handling for time-related operations
- ✅ Added monotonic clock support
  - Implemented `start_monotonic` and `stop_monotonic` functions
  - Added `read_monotonic` to retrieve elapsed time
  - Used `std::time::Instant` for monotonic time tracking
- ✅ Implemented timezone handling
  - Added `read_timezone` function to retrieve system timezone
  - Implemented error handling for timezone operations
- ✅ Added WIT bindings
  - Defined `clock.wit` interface with appropriate types and functions
  - Implemented Rust bindings using `wit-bindgen`
- ✅ Created comprehensive tests
  - Tested current time retrieval
  - Tested monotonic clock functionality
  - Tested timezone handling
  - All tests passing successfully

### SEV-SNP Clock Implementation
- ✅ Implemented SEV-SNP specific clock functionality
  - Added `sev` crate dependency with `snp` feature
  - Created SEV-SNP specific clock implementation in `src/sev/clock.rs`
  - Implemented secure time measurement using TSC
  - Added SEV-SNP platform verification
  - Updated SEV-SNP detection to use `/dev/sev-guest` instead of `/dev/sev`
- ✅ Implemented clock types for SEV-SNP
  - System clock: Uses TSC with wallclock time
  - Monotonic clock: Uses TSC directly
  - Process/Thread clocks: Uses TSC with offsets
- ✅ Added SEV-SNP specific features
  - High-resolution support based on TSC frequency
  - Secure time measurement through SEV-SNP protected RDTSC
  - Platform status verification through SEV firmware
- ✅ Created comprehensive tests
  - Tested clock creation and destruction
  - Tested time retrieval for different clock types
  - Tested resolution and elapsed time calculations
  - All tests passing successfully
- ✅ Added WASM example with SEV-SNP detection
  - Created WASM binary that detects SEV-SNP environment
  - Added logging to show which clock mechanism is being used
  - Verified correct operation on both SEV-SNP and non-SEV-SNP machines

### File Interface
- ✅ Implemented secure file operations
  - Added file creation, reading, writing, and deletion
  - Implemented file metadata handling (size, permissions, timestamps)
  - Added directory listing functionality
- ✅ Added encryption support (AES-GCM)
  - Implemented AES-GCM encryption for file data
  - Added key generation and management
  - Implemented secure file modes (encrypted vs. regular)
- ✅ Implemented file metadata handling
  - Added `FileMetadata` struct with size, permissions, and timestamps
  - Implemented metadata retrieval functions
- ✅ Added WIT bindings
  - Defined `file.wit` interface with appropriate types and functions
  - Implemented Rust bindings using `wit-bindgen`
- ✅ Created comprehensive tests
  - Tested basic file operations (create, read, write, delete)
  - Tested encryption functionality
  - Tested error handling
  - Tested file modes
  - All tests passing successfully

### Crypto Interface
- ✅ Defined WIT interface
  - Created `crypto.wit` with appropriate types and functions
  - Defined key types, algorithms, and operations
- ✅ Started implementation
  - Implemented key generation and management
  - Added symmetric encryption/decryption (AES-GCM)
  - Started RSA implementation for asymmetric operations
  - Added hashing and MAC functionality
- ✅ Completed implementation
  - Fixed RSA signing implementation
  - Resolved HMAC initialization
  - Updated dependencies
  - Added comprehensive tests
- ✅ Refactored for WASM compatibility
  - Removed WIT bindings in favor of environment variables
  - Added WASI support for environment variable access
  - Implemented SEV-SNP detection using environment variables
  - Split error handling into separate module
  - Added WASM-specific implementations
  - Simplified Linux module to re-export AES implementation
  - Updated all dependencies to remove WIT bindings
  - Created WASM example with SEV-SNP environment detection

### TLS Interface
- ⏳ Planning phase
  - Define WIT interface for TLS operations
  - Plan certificate and key management
  - Design connection handling
  - Plan secure data transfer
- ⏳ Implementation pending
  - Need to implement TLS context management
  - Need to implement certificate handling
  - Need to implement connection management
  - Need to implement secure data transfer
  - Need to implement connection information
- ⏳ Testing pending
  - Need to create test suite
  - Need to test client connections
  - Need to test server connections
  - Need to test certificate verification
  - Need to test error handling

## Current Status
- Clock Interface: Complete and tested
  - All functionality implemented and working
  - Tests passing successfully
  - Documentation updated in README.md
- File Interface: Complete and tested
  - All functionality implemented and working
  - Tests passing successfully
  - Documentation updated in README.md
- Crypto Interface: Complete and tested
  - All functionality implemented and working
  - Tests passing successfully
  - Documentation updated in README.md
  - Refactored to use environment variables instead of WIT
  - Added WASM compatibility improvements
- TLS Interface: Planning phase
  - WIT interface defined
  - Implementation pending
  - Tests pending

## Next Steps
1. Begin TLS Interface implementation:
   - Create TLS context management
     - Implement context creation and destruction
     - Add configuration handling
     - Add certificate and key management
   - Implement connection handling
     - Add client connection support
     - Add server connection support
     - Add connection cleanup
   - Implement secure data transfer
     - Add write operations
     - Add read operations
     - Add buffer management
   - Implement connection information
     - Add peer certificate retrieval
     - Add protocol version information
     - Add cipher suite information
2. Create TLS Interface tests:
   - Test context management
   - Test certificate handling
   - Test client connections
   - Test server connections
   - Test data transfer
   - Test error handling
3. Update documentation:
   - Add TLS interface to README.md
   - Add usage examples
   - Add test instructions
4. Continue WASM improvements:
   - Update remaining interfaces to use environment variables
   - Add more WASM-specific optimizations
   - Improve error handling in WASM context
   - Add more comprehensive WASM tests

## Technical Challenges
1. RSA Implementation:
   - Issues with trait implementations and imports
   - Challenges with key loading and conversion
   - Need to properly handle signing and verification

2. HMAC Initialization:
   - Multiple `new` methods causing ambiguity
   - Need to properly specify which implementation to use
   - Challenges with key handling

3. Dependency Management:
   - Balancing feature requirements with compatibility
   - Ensuring all dependencies work together
   - Managing version constraints

4. WASM Compatibility:
   - Transitioning from WIT to environment variables
   - Ensuring proper SEV-SNP detection in WASM context
   - Managing platform-specific code paths
   - Handling WASM-specific limitations (e.g., sleep operations)

## Notes
- All interfaces follow consistent implementation patterns
  - Using environment variables for platform detection
  - Following consistent naming and structure
  - Providing clear documentation
- Tests are comprehensive and passing for completed interfaces
  - Each interface has dedicated test file
  - Tests cover all functionality
  - Error handling is thoroughly tested
- Documentation is maintained in README.md
  - Updated with each interface implementation
  - Includes usage examples
  - Lists all dependencies

## Questions & Decisions
1. Q: Should we implement standard (non-SEV) versions of these interfaces?
   A: Yes, would need new implementations for:
   - Clock: Standard system time functions
     - Use `std::time` for time operations
     - Implement standard monotonic clock
     - Use system timezone functions
   - File: Regular file system operations
     - Use `std::fs` for file operations
     - Implement software-based encryption
     - Use standard file metadata
   - Crypto: Standard cryptographic libraries
     - Use standard cryptographic libraries
     - Implement software-based key management
     - Use standard random number generators
   
   Suggested structure:
   ```
   src/
   ├── sev/           # SEV-SNP specific implementations
   │   ├── clock.rs   # SEV-SNP secure time
   │   ├── file.rs    # SEV-SNP secure files
   │   └── crypto.rs  # SEV-SNP secure crypto
   ├── standard/      # Regular environment implementations
   │   ├── clock.rs   # Standard time functions
   │   ├── file.rs    # Standard file operations
   │   └── crypto.rs  # Standard crypto libraries
   └── lib.rs         # Exports appropriate implementations
   ```

2. Q: How to handle environment detection?
   A: We could use:
   - Compile-time features to select implementation
   - Runtime detection of SEV-SNP environment
   - Configuration file to specify implementation

3. Q: How to maintain API consistency between implementations?
   A: We should:
   - Use the same WIT interfaces for both implementations
   - Ensure error types are consistent
   - Maintain similar function signatures
   - Document differences in behavior

## Dependencies
Current dependencies in Cargo.toml:
```toml
[dependencies]
wit-bindgen = "0.11.0"  # For WIT interface bindings
aes-gcm = "0.10.3"      # For AES-GCM encryption
rand = "0.8.5"          # For random number generation
rsa = { version = "0.9.6", features = ["sha2", "pem"] }  # For RSA operations
pkcs8 = "0.10.2"        # For PKCS#8 key handling
sha2 = "0.10.8"         # For SHA-256 hashing
hmac = "0.12.1"         # For HMAC operations
thiserror = "1.0"       # For error handling
sev = { version = "6.0", default-features = false, features = ["snp"] }  # SEV-SNP support

[dev-dependencies]
tempfile = "3.8.1"      # For temporary file handling in tests
```

## Git History
- Initial commit: Project setup
- Added Clock Interface implementation
- Added File Interface implementation
- Started Crypto Interface implementation
- Fixed dependencies and compilation issues
- Added implementation log
- Added SEV-SNP clock implementation
  - Implemented secure time measurement using TSC
  - Added SEV-SNP platform verification
  - Created comprehensive tests
  - Updated implementation log

## Next Session Plan
1. Fix Crypto Interface compilation issues
2. Complete Crypto Interface tests
3. Consider implementing standard versions
4. Update documentation with all changes 

### Technical Details
1. SEV-SNP Clock Implementation:
   - Uses `sev` crate for SEV-SNP platform verification
   - Implements secure time measurement using RDTSC
   - Maintains TSC-based timestamps for different clock types
   - Provides high-resolution support based on TSC frequency

2. Clock Types:
   - System Clock: Uses TSC with wallclock time (TODO: implement proper wallclock time retrieval)
   - Monotonic Clock: Uses TSC directly for secure monotonic time
   - Process/Thread Clocks: Uses TSC with process/thread specific offsets

3. Security Features:
   - RDTSC is protected in SEV-SNP environment
   - Platform status verification through SEV firmware
   - Secure time measurement through TSC

4. TODOs and Future Improvements:
   - Implement proper wallclock time retrieval through SEV-SNP mechanisms
   - Get TSC frequency from a more reliable source
   - Implement SEV-SNP specific sleep using TSC
   - Add more SEV-SNP specific error handling

## Questions & Decisions
1. Q: Should we implement standard (non-SEV) versions of these interfaces?
   A: Yes, would need new implementations for:
   - Clock: Standard system time functions
     - Use `std::time` for time operations
     - Implement standard monotonic clock
     - Use system timezone functions
   - File: Regular file system operations
     - Use `std::fs` for file operations
     - Implement software-based encryption
     - Use standard file metadata
   - Crypto: Standard cryptographic libraries
     - Use standard cryptographic libraries
     - Implement software-based key management
     - Use standard random number generators
   
   Suggested structure:
   ```
   src/
   ├── sev/           # SEV-SNP specific implementations
   │   ├── clock.rs   # SEV-SNP secure time
   │   ├── file.rs    # SEV-SNP secure files
   │   └── crypto.rs  # SEV-SNP secure crypto
   ├── standard/      # Regular environment implementations
   │   ├── clock.rs   # Standard time functions
   │   ├── file.rs    # Standard file operations
   │   └── crypto.rs  # Standard crypto libraries
   └── lib.rs         # Exports appropriate implementations
   ```

2. Q: How to handle environment detection?
   A: We could use:
   - Compile-time features to select implementation
   - Runtime detection of SEV-SNP environment
   - Configuration file to specify implementation

3. Q: How to maintain API consistency between implementations?
   A: We should:
   - Use the same WIT interfaces for both implementations
   - Ensure error types are consistent
   - Maintain similar function signatures
   - Document differences in behavior

## Dependencies
Current dependencies in Cargo.toml:
```toml
[dependencies]
wit-bindgen = "0.11.0"  # For WIT interface bindings
aes-gcm = "0.10.3"      # For AES-GCM encryption
rand = "0.8.5"          # For random number generation
rsa = { version = "0.9.6", features = ["sha2", "pem"] }  # For RSA operations
pkcs8 = "0.10.2"        # For PKCS#8 key handling
sha2 = "0.10.8"         # For SHA-256 hashing
hmac = "0.12.1"         # For HMAC operations
thiserror = "1.0"       # For error handling
sev = { version = "6.0", default-features = false, features = ["snp"] }  # SEV-SNP support

[dev-dependencies]
tempfile = "3.8.1"      # For temporary file handling in tests
```

## Git History
- Initial commit: Project setup
- Added Clock Interface implementation
- Added File Interface implementation
- Started Crypto Interface implementation
- Fixed dependencies and compilation issues
- Added implementation log
- Added SEV-SNP clock implementation
  - Implemented secure time measurement using TSC
  - Added SEV-SNP platform verification
  - Created comprehensive tests
  - Updated implementation log

## Next Session Plan
1. Fix Crypto Interface compilation issues
2. Complete Crypto Interface tests
3. Consider implementing standard versions
4. Update documentation with all changes 