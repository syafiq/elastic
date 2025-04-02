# ELASTIC Implementation Log

## Project Overview
ELASTIC (Enclave Layer for Secure Time, Information, and Cryptography) is a Hardware Abstraction Layer (HAL) for AMD SEV-SNP VMs. The project aims to provide secure abstractions for time, file operations, and cryptographic functions in a SEV-SNP environment.

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
- ⚠️ In Progress: Fixing compilation issues
  - Need to fix RSA signing implementation
    - Issues with `Pkcs1v15Sign` and `SignatureScheme` traits
    - Problems with key loading and conversion
  - Need to resolve HMAC initialization
    - Multiple `new` methods causing ambiguity
    - Need to properly specify which implementation to use
  - Need to update dependencies
    - Added `pkcs8` as a separate dependency
    - Updated RSA features to include only available ones

## Current Status
- Clock Interface: Complete and tested
  - All functionality implemented and working
  - Tests passing successfully
  - Documentation updated in README.md
- File Interface: Complete and tested
  - All functionality implemented and working
  - Tests passing successfully
  - Documentation updated in README.md
- Crypto Interface: In progress, needs fixes
  - Basic structure implemented
  - Symmetric operations working
  - Asymmetric operations need fixes
  - Tests partially implemented

## Next Steps
1. Fix Crypto Interface compilation issues:
   - Update RSA signing implementation
     - Import correct traits (`SignatureScheme`)
     - Fix key loading and conversion
     - Update signing and verification methods
   - Fix HMAC initialization
     - Use fully qualified syntax to disambiguate
     - Ensure proper key handling
   - Resolve dependency conflicts
     - Update Cargo.toml with correct features
     - Ensure all dependencies are compatible
2. Complete Crypto Interface tests
   - Implement tests for symmetric operations
   - Implement tests for asymmetric operations
   - Implement tests for hashing and MAC
   - Implement tests for error handling
3. Consider implementing standard (non-SEV) versions of interfaces
   - Plan architecture for dual implementations
   - Design abstraction layer for environment detection
   - Implement standard versions of each interface

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

## Notes
- All interfaces follow WIT specification
  - Defined in `tests/wit/` directory
  - Follow consistent naming and structure
  - Provide clear documentation
- Tests are comprehensive and passing for completed interfaces
  - Each interface has dedicated test file
  - Tests cover all functionality
  - Error handling is thoroughly tested
- Documentation is maintained in README.md
  - Updated with each interface implementation
  - Includes usage examples
  - Lists all dependencies
- WIT interfaces are stored in tests/wit/
  - `clock.wit`: Time-related operations
  - `file.wit`: File operations
  - `crypto.wit`: Cryptographic operations

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

## Next Session Plan
1. Fix Crypto Interface compilation issues
2. Complete Crypto Interface tests
3. Consider implementing standard versions
4. Update documentation with all changes 