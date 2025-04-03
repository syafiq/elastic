use elastic_clock::{ClockConfig, ClockType, ClockError};
use tokio::time::sleep;
use std::time::Duration;
use tokio_test::block_on;

#[tokio::test]
async fn test_clock_operations() {
    let ctx = ClockContext::new();
    
    // Test clock creation
    let config = ClockConfig {
        clock_type: ClockType::System,
        high_resolution: true,
    };
    
    let handle = ctx.create_clock(&config).await.unwrap();
    
    // Test get_time
    let time1 = ctx.get_time(handle).await.unwrap();
    assert!(time1 > 0);
    
    // Test get_resolution
    let resolution = ctx.get_resolution(handle).await.unwrap();
    assert_eq!(resolution, 1); // 1 nanosecond for high resolution
    
    // Test sleep
    let duration = 100_000_000; // 100ms
    let start = ctx.get_time(handle).await.unwrap();
    ctx.sleep(handle, duration).await.unwrap();
    let end = ctx.get_time(handle).await.unwrap();
    assert!(end - start >= duration);
    
    // Test get_elapsed
    let elapsed = ctx.get_elapsed(handle).await.unwrap();
    assert!(elapsed > 0);
    
    // Test clock destruction
    ctx.destroy_clock(handle).await.unwrap();
}

#[tokio::test]
async fn test_invalid_handle() {
    let ctx = ClockContext::new();
    
    // Test operations with invalid handle
    assert!(ctx.get_time(999).await.is_err());
    assert!(ctx.get_resolution(999).await.is_err());
    assert!(ctx.sleep(999, 1000).await.is_err());
    assert!(ctx.get_elapsed(999).await.is_err());
    assert!(ctx.destroy_clock(999).await.is_err());
}

#[tokio::test]
async fn test_sev_snp_initialization() {
    // Test that clock context creation works in SEV-SNP environment
    let ctx = ClockContext::new();
    
    // Create a system clock to test GHCB initialization
    let config = ClockConfig {
        clock_type: ClockType::System,
        high_resolution: true,
    };
    
    let handle = ctx.create_clock(&config).await.unwrap();
    
    // Get time to verify GHCB communication works
    let time = ctx.get_time(handle).await.unwrap();
    assert!(time > 0);
    
    ctx.destroy_clock(handle).await.unwrap();
}

#[test]
fn test_sev_snp_tsc_access() {
    let clock_manager = elastic_clock::sev::ClockManager::new();
    assert!(clock_manager.is_ok(), "Failed to create clock manager");
}

#[test]
fn test_sev_snp_monotonic_clock() {
    let clock_manager = elastic_clock::sev::ClockManager::new().unwrap();
    
    let config = ClockConfig {
        clock_type: ClockType::Monotonic,
        high_resolution: true,
    };
    
    let handle = block_on(clock_manager.create_clock(&config)).unwrap();
    
    // Get initial time
    let time1 = block_on(clock_manager.get_time(handle)).unwrap();
    
    // Sleep for a short duration
    block_on(sleep(Duration::from_millis(10)));
    
    // Get time after sleep
    let time2 = block_on(clock_manager.get_time(handle)).unwrap();
    
    // Time should be strictly increasing
    assert!(time2 > time1, "Monotonic clock not increasing");
    
    // Cleanup
    block_on(clock_manager.destroy_clock(handle)).unwrap();
}

#[test]
fn test_sev_snp_system_clock() {
    let clock_manager = elastic_clock::sev::ClockManager::new().unwrap();
    
    let config = ClockConfig {
        clock_type: ClockType::System,
        high_resolution: true,
    };
    
    let handle = block_on(clock_manager.create_clock(&config)).unwrap();
    
    // Get initial time
    let time1 = block_on(clock_manager.get_time(handle)).unwrap();
    
    // Sleep for a short duration
    block_on(sleep(Duration::from_millis(10)));
    
    // Get time after sleep
    let time2 = block_on(clock_manager.get_time(handle)).unwrap();
    
    // Time should be increasing
    assert!(time2 > time1, "System clock not increasing");
    
    // Cleanup
    block_on(clock_manager.destroy_clock(handle)).unwrap();
}

#[test]
fn test_sev_snp_process_thread_clocks() {
    let clock_manager = elastic_clock::sev::ClockManager::new().unwrap();
    
    // Create process clock
    let process_config = ClockConfig {
        clock_type: ClockType::Process,
        high_resolution: true,
    };
    let process_handle = block_on(clock_manager.create_clock(&process_config)).unwrap();
    
    // Create thread clock
    let thread_config = ClockConfig {
        clock_type: ClockType::Thread,
        high_resolution: true,
    };
    let thread_handle = block_on(clock_manager.create_clock(&thread_config)).unwrap();
    
    // Get times from both clocks
    let process_time = block_on(clock_manager.get_time(process_handle)).unwrap();
    let thread_time = block_on(clock_manager.get_time(thread_handle)).unwrap();
    
    // Times should be different due to different offsets
    assert_ne!(process_time, thread_time, "Process and thread clocks should have different values");
    
    // Cleanup
    block_on(clock_manager.destroy_clock(process_handle)).unwrap();
    block_on(clock_manager.destroy_clock(thread_handle)).unwrap();
}

#[test]
fn test_sev_snp_resolution() {
    let clock_manager = elastic_clock::sev::ClockManager::new().unwrap();
    
    // Test high resolution clock
    let high_res_config = ClockConfig {
        clock_type: ClockType::Monotonic,
        high_resolution: true,
    };
    let high_res_handle = block_on(clock_manager.create_clock(&high_res_config)).unwrap();
    let high_res = block_on(clock_manager.get_resolution(high_res_handle)).unwrap();
    
    // Test low resolution clock
    let low_res_config = ClockConfig {
        clock_type: ClockType::Monotonic,
        high_resolution: false,
    };
    let low_res_handle = block_on(clock_manager.create_clock(&low_res_config)).unwrap();
    let low_res = block_on(clock_manager.get_resolution(low_res_handle)).unwrap();
    
    // High resolution should be better than low resolution
    assert!(high_res < low_res, "High resolution clock should have better resolution");
    
    // Cleanup
    block_on(clock_manager.destroy_clock(high_res_handle)).unwrap();
    block_on(clock_manager.destroy_clock(low_res_handle)).unwrap();
}

#[test]
fn test_sev_snp_elapsed_time() {
    let clock_manager = elastic_clock::sev::ClockManager::new().unwrap();
    
    let config = ClockConfig {
        clock_type: ClockType::Monotonic,
        high_resolution: true,
    };
    
    let handle = block_on(clock_manager.create_clock(&config)).unwrap();
    
    // Sleep for a known duration
    let sleep_duration = Duration::from_millis(10);
    block_on(sleep(sleep_duration));
    
    // Get elapsed time
    let elapsed = block_on(clock_manager.get_elapsed(handle)).unwrap();
    
    // Elapsed time should be at least the sleep duration
    assert!(elapsed >= sleep_duration.as_nanos() as u64, "Elapsed time too short");
    
    // Cleanup
    block_on(clock_manager.destroy_clock(handle)).unwrap();
}

#[test]
fn test_sev_snp_error_handling() {
    let clock_manager = elastic_clock::sev::ClockManager::new().unwrap();
    
    // Test invalid handle
    let invalid_handle = 9999;
    let result = block_on(clock_manager.get_time(invalid_handle));
    assert!(result.is_err(), "Should fail with invalid handle");
    
    // Test double destruction
    let config = ClockConfig {
        clock_type: ClockType::Monotonic,
        high_resolution: true,
    };
    
    let handle = block_on(clock_manager.create_clock(&config)).unwrap();
    block_on(clock_manager.destroy_clock(handle)).unwrap();
    
    let result = block_on(clock_manager.destroy_clock(handle));
    assert!(result.is_err(), "Should fail on double destruction");
} 