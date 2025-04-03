use elastic_clock::{ClockConfig, ClockContext, ClockType};

#[tokio::test]
async fn test_clock_operations() {
    let ctx = ClockContext::new();
    
    // Test clock creation
    let config = ClockConfig {
        clock_type: ClockType::System,
        high_resolution: true,
    };
    
    let handle = ctx.create_clock(&config).unwrap();
    
    // Test get_time
    let time1 = ctx.get_time(handle).unwrap();
    assert!(time1 > 0);
    
    // Test get_resolution
    let resolution = ctx.get_resolution(handle).unwrap();
    assert_eq!(resolution, 1); // 1 nanosecond for high resolution
    
    // Test sleep
    let duration = 100_000_000; // 100ms
    let start = ctx.get_time(handle).unwrap();
    ctx.sleep(handle, duration).await.unwrap();
    let end = ctx.get_time(handle).unwrap();
    assert!(end - start >= duration);
    
    // Test get_elapsed
    let elapsed = ctx.get_elapsed(handle).unwrap();
    assert!(elapsed > 0);
    
    // Test clock destruction
    ctx.destroy_clock(handle).unwrap();
}

#[tokio::test]
async fn test_multiple_clocks() {
    let ctx = ClockContext::new();
    
    // Create multiple clocks with different configurations
    let config1 = ClockConfig {
        clock_type: ClockType::System,
        high_resolution: true,
    };
    
    let config2 = ClockConfig {
        clock_type: ClockType::Monotonic,
        high_resolution: false,
    };
    
    let handle1 = ctx.create_clock(&config1).unwrap();
    let handle2 = ctx.create_clock(&config2).unwrap();
    
    // Verify different clocks work independently
    let time1 = ctx.get_time(handle1).unwrap();
    let time2 = ctx.get_time(handle2).unwrap();
    assert!(time1 > 0 && time2 > 0);
    
    // Clean up
    ctx.destroy_clock(handle1).unwrap();
    ctx.destroy_clock(handle2).unwrap();
}

#[tokio::test]
async fn test_error_handling() {
    let ctx = ClockContext::new();
    
    // Test invalid handle
    assert!(ctx.get_time(999).is_err());
    assert!(ctx.get_resolution(999).is_err());
    assert!(ctx.destroy_clock(999).is_err());
    
    // Test valid handle
    let handle = ctx.create_clock(&ClockConfig::default()).unwrap();
    assert!(ctx.get_time(handle).is_ok());
    assert!(ctx.get_resolution(handle).is_ok());
    assert!(ctx.destroy_clock(handle).is_ok());
    
    // Test double destruction
    assert!(ctx.destroy_clock(handle).is_err());
} 