use elastic_clock::{ClockConfig, ClockContext, ClockType};
use std::time::Duration;

#[tokio::test]
async fn test_clock_creation_and_destruction() {
    let ctx = ClockContext::new();
    let config = ClockConfig::default();
    
    let handle = ctx.create_clock(&config).unwrap();
    assert!(handle > 0);
    
    ctx.destroy_clock(handle).unwrap();
}

#[tokio::test]
async fn test_clock_types() {
    let ctx = ClockContext::new();
    
    // Test system clock
    let mut config = ClockConfig::default();
    config.clock_type = ClockType::System;
    let handle = ctx.create_clock(&config).unwrap();
    let time1 = ctx.get_time(handle).unwrap();
    let time2 = ctx.get_time(handle).unwrap();
    assert!(time2 >= time1);
    ctx.destroy_clock(handle).unwrap();
    
    // Test monotonic clock
    config.clock_type = ClockType::Monotonic;
    let handle = ctx.create_clock(&config).unwrap();
    let time1 = ctx.get_time(handle).unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    let time2 = ctx.get_time(handle).unwrap();
    assert!(time2 > time1);
    ctx.destroy_clock(handle).unwrap();
}

#[tokio::test]
async fn test_clock_resolution() {
    let ctx = ClockContext::new();
    let mut config = ClockConfig::default();
    
    // Test low resolution
    config.high_resolution = false;
    let handle = ctx.create_clock(&config).unwrap();
    let resolution = ctx.get_resolution(handle).unwrap();
    assert_eq!(resolution, 1_000_000); // 1 millisecond
    ctx.destroy_clock(handle).unwrap();
    
    // Test high resolution
    config.high_resolution = true;
    let handle = ctx.create_clock(&config).unwrap();
    let resolution = ctx.get_resolution(handle).unwrap();
    assert_eq!(resolution, 1); // 1 nanosecond
    ctx.destroy_clock(handle).unwrap();
}

#[tokio::test]
async fn test_clock_sleep() {
    let ctx = ClockContext::new();
    let config = ClockConfig::default();
    let handle = ctx.create_clock(&config).unwrap();
    
    let start = std::time::Instant::now();
    ctx.sleep(handle, 100_000_000).await.unwrap(); // 100ms
    let elapsed = start.elapsed();
    
    assert!(elapsed >= Duration::from_millis(100));
    ctx.destroy_clock(handle).unwrap();
}

#[tokio::test]
async fn test_clock_elapsed() {
    let ctx = ClockContext::new();
    let config = ClockConfig::default();
    let handle = ctx.create_clock(&config).unwrap();
    
    let elapsed1 = ctx.get_elapsed(handle).unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    let elapsed2 = ctx.get_elapsed(handle).unwrap();
    
    assert!(elapsed2 > elapsed1);
    ctx.destroy_clock(handle).unwrap();
}

#[tokio::test]
async fn test_invalid_handle() {
    let ctx = ClockContext::new();
    
    assert!(ctx.get_time(0).is_err());
    assert!(ctx.get_resolution(0).is_err());
    assert!(ctx.destroy_clock(0).is_err());
    assert!(ctx.get_elapsed(0).is_err());
} 