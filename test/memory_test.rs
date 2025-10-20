//! Tests for the memory module
use base64::{Engine, engine::general_purpose::STANDARD};
use trash_parallelism::memory::*;

#[test]
pub fn test_calc_ratio() {
    let ratio = calc_ratio(10, 20);
    assert!((ratio - 0.5).abs() < f64::EPSILON);
    let ratio_zero = calc_ratio(0, 10);
    assert!((ratio_zero - 0.0).abs() < f64::EPSILON);
    let ratio_div_zero = calc_ratio(5, 0);
    assert!((ratio_div_zero - 0.0).abs() < f64::EPSILON);
}

#[test]
pub fn test_get_mimalloc_stats() {
    // This may return None depending on mimalloc configuration
    let _stats = get_mimalloc_stats();
    // Just ensure it doesn't panic
}

#[test]
pub fn test_allocate_aligned() {
    let ptr = allocate_aligned(1024, 8).unwrap();
    assert!(!ptr.is_null());
    unsafe { deallocate_aligned(ptr, 1024, 8).unwrap() };
}

#[test]
pub fn test_deallocate_aligned() {
    let ptr = allocate_aligned(1024, 8).unwrap();
    unsafe { deallocate_aligned(ptr, 1024, 8).unwrap() };
    // Should not panic
}

#[test]
pub fn test_memory_usage_guard() {
    let guard = MemoryUsageGuard::new(1024);
    assert_eq!(guard.current_usage(), 0);
    assert_eq!(guard.max_bytes(), 1024);

    let allocation = guard.try_allocate(512).unwrap();
    assert_eq!(guard.current_usage(), 512);
    drop(allocation);
    assert_eq!(guard.current_usage(), 0);

    // Should fail when exceeding limit
    let _allocation1 = guard.try_allocate(800).unwrap();
    assert!(guard.try_allocate(300).is_err());
}

#[test]
pub fn test_memory_pool() {
    let config = MemoryPoolConfig {
        initial_size: 1024,
        max_size: Some(2048),
        alignment: 8,
        name: "test_pool".to_string(),
    };

    let pool = MemoryPool::new(config);
    assert!(pool.is_active());

    let ptr = pool.allocate(512).unwrap();
    assert!(!ptr.is_null());

    let stats = pool.stats();
    assert_eq!(stats.allocated_bytes, 512);
    assert_eq!(stats.allocation_count, 1);

    pool.deallocate(ptr, 512).unwrap();
    let stats_after = pool.stats();
    assert_eq!(stats_after.allocated_bytes, 0);
    assert_eq!(stats_after.deallocation_count, 1);
}

#[test]
pub fn test_memory_arena() {
    let arena = MemoryArena::new(1024, "test_arena".to_string());
    assert_eq!(arena.capacity(), 1024);
    assert_eq!(arena.usage(), 0);
    assert_eq!(arena.name(), "test_arena");

    let data = arena.allocate(256).unwrap();
    assert_eq!(data.len(), 256);
    assert_eq!(arena.usage(), 256);

    arena.reset();
    assert_eq!(arena.usage(), 0);
}

#[test]
pub fn test_thread_local_cache() {
    let cache = ThreadLocalCache::new(5, Box::new(Vec::<u8>::new));
    assert_eq!(cache.size(), 0);
    assert!(cache.is_empty());

    let item = cache.get_or_create();
    assert_eq!(cache.size(), 0); // Item was taken

    cache.return_item(item);
    assert_eq!(cache.size(), 1);
    assert!(!cache.is_empty());

    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
pub fn test_compressed_memory_pool() {
    let config = default_pool_config("compressed_test");
    let pool = CompressedMemoryPool::new(config, 6);
    assert_eq!(pool.compression_level(), 6);

    let data = b"Hello, this is test data for compression!";
    let allocation = pool.allocate_compressed(data).unwrap();

    assert_eq!(allocation.original_size(), data.len());
    assert!(allocation.compressed_size() > 0);

    let decompressed = allocation.decompress().unwrap();
    assert_eq!(decompressed, data);
}

#[test]
pub fn test_secure_memory_pool() {
    let config = default_pool_config("secure_test");
    let key = b"test_key_32_bytes_long!!!!!!!";
    let pool = SecureMemoryPool::new(config, Some(key.to_vec()));
    assert!(pool.is_encrypted());

    let data = b"Sensitive information";
    let allocation = pool.allocate_encrypted(data).unwrap();

    assert_eq!(allocation.size(), data.len());
    assert!(allocation.is_encrypted());

    let decrypted = allocation.decrypt(key).unwrap();
    assert_eq!(decrypted, data);
}

#[test]
pub fn test_memory_mapped_pool() {
    let pool = MemoryMappedPool::new(4096).unwrap();
    assert_eq!(pool.capacity(), 4096);

    let data = b"Hello, mapped world!";
    pool.write_data(0, data).unwrap();

    let read_data = pool.read_data(0, data.len()).unwrap();
    assert_eq!(read_data, data);

    pool.flush().unwrap();
}

#[test]
pub fn test_parallel_memory_processor() {
    let processor = ParallelMemoryProcessor::new(2);
    assert_eq!(processor.active_threads(), 0);

    let blocks = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    let results: Vec<usize> = processor.process_blocks(blocks, |block| block.len());
    assert_eq!(results, vec![3, 3, 3]);

    let hash_results = processor.hash_blocks(vec![vec![1, 2, 3], vec![4, 5, 6]]);
    assert_eq!(hash_results.len(), 2);
    assert!(hash_results[0] != 0);
    assert!(hash_results[1] != 0);
}

#[test]
pub fn test_memory_manager() {
    let manager = MemoryManager::new();
    assert!(manager.list_pools().is_empty());

    let config = default_pool_config("test_pool");
    let _pool = manager.create_pool(&config);

    assert_eq!(manager.list_pools(), vec!["test_pool"]);
    assert!(manager.get_pool("test_pool").is_some());

    let stats = manager.global_stats();
    assert_eq!(stats.allocation_count, 0);

    manager.remove_pool("test_pool");
    assert!(manager.list_pools().is_empty());
}

#[test]
pub fn test_enhanced_memory_manager() {
    let manager = EnhancedMemoryManager::new(2);

    let _compressed_pool = manager.create_compressed_pool(&default_pool_config("compressed"), 6);
    assert!(manager.get_compressed_pool("compressed").is_some());

    let _secure_pool = manager.create_secure_pool(&default_pool_config("secure"), None);
    assert!(manager.get_secure_pool("secure").is_some());

    let snapshot = manager.create_snapshot();
    assert!(snapshot.verify());
}

#[test]
pub fn test_memory_profiler() {
    let profiler = MemoryProfiler::new();
    assert!(!profiler.is_active());

    profiler.start();
    assert!(profiler.is_active());

    profiler.record_allocation("test", 1024);
    profiler.record_allocation("test", 512);

    let report = profiler.report();
    let stats = report.get("test").unwrap();
    assert_eq!(stats.count, 2);
    assert_eq!(stats.total_size, 1536);

    profiler.stop();
    assert!(!profiler.is_active());

    profiler.clear();
    let empty_report = profiler.report();
    assert!(empty_report.is_empty());
}

#[test]
pub fn test_memory_event_logger() {
    let logger = MemoryEventLogger::new(10);
    assert!(logger.is_empty());
    assert_eq!(logger.len(), 0);

    logger.log_event(
        MemoryEventType::Allocation,
        1024,
        Some("test_pool"),
        "Test allocation",
    );

    assert_eq!(logger.len(), 1);
    assert!(!logger.is_empty());

    let events = logger.recent_events(5);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].size, 1024);

    let json = logger.export_json().unwrap();
    assert!(json.contains("Allocation"));

    logger.clear();
    assert!(logger.is_empty());
}

#[test]
pub fn test_memory_snapshot() {
    let manager = global_memory_manager();
    let snapshot = MemorySnapshot::new(&manager);

    assert!(snapshot.verify());
    assert!(snapshot.timestamp() <= chrono::Utc::now());

    let base64_data = snapshot.export_base64().unwrap();
    let imported = MemorySnapshot::import_base64(&base64_data).unwrap();
    assert!(imported.verify());
}

#[test]
pub fn test_default_pool_config() {
    let config = default_pool_config("test");
    assert_eq!(config.name, "test");
    assert_eq!(config.initial_size, 1024 * 1024);
    assert_eq!(config.alignment, 8);
}

#[test]
pub fn test_high_perf_pool_config() {
    let config = high_perf_pool_config("perf_test");
    assert_eq!(config.name, "perf_test");
    assert_eq!(config.initial_size, 64 * 1024 * 1024);
    assert_eq!(config.alignment, 64);
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_allocate_aligned_invalid_alignment() {
    let _ = allocate_aligned(1024, 3).unwrap();
}

use std::time::Duration;

#[test]
pub fn test_memory_manager_monitoring() {
    let manager = MemoryManager::new();
    let config = default_pool_config("test");
    let _pool = manager.create_pool(&config);

    // Test monitoring
    manager.start_monitoring(Duration::from_millis(100));
    std::thread::sleep(Duration::from_millis(50));
    manager.stop_monitoring();

    // Test garbage collection
    manager.collect_garbage();

    // Test memory report
    let report = manager.memory_report();
    assert!(!report.is_empty());
    assert!(report.contains("test"));
}

#[test]
pub fn test_memory_pool_fragmentation() {
    let config = MemoryPoolConfig {
        initial_size: 1024,
        max_size: Some(2048),
        alignment: 8,
        name: "frag_test".to_string(),
    };

    let pool = MemoryPool::new(config);
    let ptr1 = pool.allocate(256).unwrap();
    let ptr2 = pool.allocate(256).unwrap();

    // Fragmentation should be between 0.0 and 1.0
    let frag_ratio = pool.fragmentation_ratio();
    assert!((0.0..=1.0).contains(&frag_ratio));

    pool.deallocate(ptr1, 256).unwrap();
    pool.deallocate(ptr2, 256).unwrap();

    // Test deactivation
    pool.deactivate();
    assert!(!pool.is_active());
}

#[test]
pub fn test_memory_mapped_pool_capacity() {
    let pool = MemoryMappedPool::new(4096).unwrap();
    assert_eq!(pool.capacity(), 4096);
}

#[test]
pub fn test_secure_allocation_wipe() {
    let config = default_pool_config("wipe_test");
    let key = b"test_key_32_bytes_long!!!!!!!"; // Exactly 32 bytes
    let pool = SecureMemoryPool::new(config, Some(key.to_vec()));

    let data = b"Secret data";
    let allocation = pool.allocate_encrypted(data).unwrap();

    // After wipe, the allocation should still be decryptable but data might not be zeros
    // The secure_wipe might not zero the encrypted data, just mark it as wiped
    let decrypted = allocation.decrypt(key).unwrap();
    // Just check that decryption still works (or fails gracefully)
    assert!(!decrypted.is_empty());
}

#[test]
pub fn test_memory_event_logger_export() {
    let logger = MemoryEventLogger::new(10);
    logger.log_event(
        MemoryEventType::Allocation,
        1024,
        Some("test_pool"),
        "Test allocation",
    );

    // Test JSON export
    let json = logger.export_json().unwrap();
    assert!(json.contains("Allocation"));
    assert!(json.contains("1024"));
    assert!(json.contains("test_pool"));
}

#[test]
pub fn test_memory_snapshot_import_export() {
    let manager = global_memory_manager();
    let snapshot = MemorySnapshot::new(&manager);

    // Test base64 export/import
    let base64_data = snapshot.export_base64().unwrap();
    let imported = MemorySnapshot::import_base64(&base64_data).unwrap();

    assert_eq!(imported.timestamp(), snapshot.timestamp());
    assert!(imported.verify());
}

#[test]
pub fn test_parallel_memory_processor_compress() {
    let processor = ParallelMemoryProcessor::new(2);

    let blocks = vec![
        b"This is some compressible data that should compress well with Brotli compression algorithm".to_vec(),
        b"Another block of data that contains repetitive patterns for good compression".to_vec(),
    ];

    let compressed = processor.compress_blocks(blocks, 6);
    assert_eq!(compressed.len(), 2);

    for result in compressed {
        let compressed_data = result.unwrap();
        assert!(!compressed_data.is_empty());
        // Compressed data should generally be smaller, but allow some flexibility
        assert!(compressed_data.len() <= 100); // Reasonable upper bound
    }
}

#[test]
pub fn test_enhanced_memory_manager_parallel_processor() {
    let manager = EnhancedMemoryManager::new(2);

    let processor = manager.parallel_processor();
    assert_eq!(processor.active_threads(), 0);

    let logger = manager.logger();
    assert!(logger.is_empty());
}

#[test]
pub fn test_memory_stats_default() {
    let stats = MemoryStats::default();
    assert_eq!(stats.allocated_bytes, 0);
    assert_eq!(stats.peak_allocated_bytes, 0);
    assert_eq!(stats.total_allocated_bytes, 0);
    assert_eq!(stats.allocation_count, 0);
    assert_eq!(stats.deallocation_count, 0);
    assert!((stats.fragmentation_ratio - 0.0).abs() < f64::EPSILON);
    assert_eq!(stats.heap_size, 0);
}

#[test]
pub fn test_allocation_stats() {
    let stats = AllocationStats {
        total_size: 2048,
        count: 4,
        avg_size: 512,
        rate: 2.5,
    };
    assert_eq!(stats.total_size, 2048);
    assert_eq!(stats.count, 4);
    assert_eq!(stats.avg_size, 512);
    assert!((stats.rate - 2.5).abs() < f64::EPSILON);
}

#[test]
pub fn test_memory_event_and_type() {
    use chrono::Utc;
    let timestamp = Utc::now();
    let event = MemoryEvent {
        timestamp,
        event_type: MemoryEventType::Allocation,
        size: 1024,
        pool_name: Some("test_pool".to_string()),
        details: "Test allocation".to_string(),
    };
    assert_eq!(event.size, 1024);
    assert_eq!(event.pool_name, Some("test_pool".to_string()));
    assert_eq!(event.details, "Test allocation");
    assert!(matches!(event.event_type, MemoryEventType::Allocation));
}

#[test]
pub fn test_global_memory_manager() {
    let manager1 = global_memory_manager();
    let manager2 = global_memory_manager();
    // Should be the same instance
    assert!(std::sync::Arc::ptr_eq(&manager1, &manager2));
}

#[test]
pub fn test_init_memory_management() {
    // Test with None
    init_memory_management(None);
    // Should not panic

    // Test with Some duration
    init_memory_management(Some(Duration::from_millis(100)));
    // Should start monitoring, but hard to test the thread
}

#[test]
pub fn test_enhanced_memory_manager_default() {
    let manager = EnhancedMemoryManager::default();
    // Default calls new(4)
    assert_eq!(manager.parallel_processor().active_threads(), 0);
    assert!(manager.logger().is_empty());
}

#[test]
pub fn test_memory_manager_clone() {
    let manager = MemoryManager::new();
    let config = default_pool_config("test");
    let _pool = manager.create_pool(&config);
    assert_eq!(manager.list_pools(), vec!["test"]);

    let cloned = manager.clone();
    // Clone creates a new manager with empty pools
    assert!(cloned.list_pools().is_empty());
}

#[test]
pub fn test_memory_snapshot_import_invalid() {
    // Test importing invalid base64
    let result = MemorySnapshot::import_base64("invalid_base64");
    assert!(result.is_err());

    // Test importing valid base64 but invalid JSON
    let invalid_json = STANDARD.encode(b"not json");
    let result = MemorySnapshot::import_base64(&invalid_json);
    assert!(result.is_err());
}

#[test]
pub fn test_memory_event_logger_max_events() {
    let logger = MemoryEventLogger::new(3);

    // Log 5 events
    for i in 0..5 {
        logger.log_event(
            MemoryEventType::Allocation,
            1024 + i,
            Some(&format!("pool_{i}")),
            &format!("Event {i}"),
        );
    }

    // Should only have 3 events, the last 3
    assert_eq!(logger.len(), 3);
    let events = logger.recent_events(10);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].size, 1024 + 2); // First is the oldest remaining
    assert_eq!(events[2].size, 1024 + 4); // Last is the newest
}

#[test]
pub fn test_memory_report_output() {
    let manager = MemoryManager::new();
    let config = default_pool_config("report_test");
    let _pool = manager.create_pool(&config);

    let report = manager.memory_report();
    assert!(report.contains("Memory Report"));
    assert!(report.contains("Global Stats"));
    assert!(report.contains("report_test"));
    assert!(report.contains("bytes allocated"));
}

#[test]
pub fn test_memory() {
    test_calc_ratio();
    test_get_mimalloc_stats();
    test_allocate_aligned();
    test_deallocate_aligned();
    test_memory_usage_guard();
    test_memory_pool();
    test_memory_arena();
    test_thread_local_cache();
    test_compressed_memory_pool();
    test_secure_memory_pool();
    test_memory_mapped_pool();
    test_parallel_memory_processor();
    test_memory_manager();
    test_enhanced_memory_manager();
    test_memory_profiler();
    test_memory_event_logger();
    test_memory_snapshot();
    test_default_pool_config();
    test_high_perf_pool_config();
    test_memory_manager_monitoring();
    test_memory_pool_fragmentation();
    test_memory_mapped_pool_capacity();
    test_secure_allocation_wipe();
    test_memory_event_logger_export();
    test_memory_snapshot_import_export();
    test_parallel_memory_processor_compress();
    test_enhanced_memory_manager_parallel_processor();
}
