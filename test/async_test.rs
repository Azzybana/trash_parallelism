//! Tests for the async module
use trash_utilities::r#async::*;

#[test]
pub fn test_sleep_for() {
    smol::block_on(async {
        let start = std::time::Instant::now();
        core::sleep_for(std::time::Duration::from_millis(10)).await;
        let elapsed = start.elapsed();
        assert!(elapsed >= std::time::Duration::from_millis(10));
    });
}

#[test]
pub fn test_race() {
    smol::block_on(async {
        let result = core::race(
            async {
                core::sleep_for(std::time::Duration::from_millis(50)).await;
                "slow"
            },
            async { "fast" },
        )
        .await;
        assert_eq!(result, "fast");
    });
}

#[test]
pub fn test_join() {
    smol::block_on(async {
        let (result1, result2) = core::join(async { 1 }, async { 2 }).await;
        assert_eq!((result1, result2), (1, 2));
    });
}

#[test]
pub fn test_create_cancellation_token() {
    let token = core::create_cancellation_token();
    assert!(!token.is_cancelled());
}

#[test]
pub fn test_with_cancellation() {
    smol::block_on(async {
        let token = core::create_cancellation_token();
        let result = core::with_cancellation(&token, async { 42 }).await;
        assert_eq!(result, Some(42));
    });
}

#[test]
pub fn test_create_mutex() {
    let mutex = core::create_mutex(42);
    let value = mutex.lock();
    assert_eq!(*value, 42);
}

#[test]
pub fn test_compress_data_async() {
    smol::block_on(async {
        let data = b"Hello, world! This is test data for compression.";
        let compressed = data::compress_data_async(data, 6).await.unwrap();
        assert!(!compressed.is_empty());
        assert!(compressed.len() < data.len()); // Should be smaller for compressible data
    });
}

#[test]
pub fn test_decompress_data_async() {
    smol::block_on(async {
        let data = b"Hello, world! This is test data.";
        let compressed = data::compress_data_async(data, 6).await.unwrap();
        let decompressed = data::decompress_data_async(&compressed).await.unwrap();
        assert_eq!(decompressed, data);
    });
}

#[test]
pub fn test_serialize_async() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let json = data::serialize_async(&test).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("42"));
}

#[test]
pub fn test_deserialize_async() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let json = r#"{"name":"test","value":42}"#;
    let parsed: TestStruct = data::deserialize_async(json).unwrap();
    assert_eq!(
        parsed,
        TestStruct {
            name: "test".to_string(),
            value: 42
        }
    );
}

#[test]
pub fn test_hash_data_async() {
    smol::block_on(async {
        let hash1 = data::hash_data_async(b"hello").await;
        let hash2 = data::hash_data_async(b"world").await;
        let hash1_again = data::hash_data_async(b"hello").await;

        assert_ne!(hash1, hash2);
        assert_eq!(hash1, hash1_again); // Deterministic
        assert_ne!(hash1, 0);
    });
}

#[test]
pub fn test_encode_base64_async() {
    smol::block_on(async {
        let data = b"Hello!";
        let encoded = data::encode_base64_async(data).await;
        assert!(!encoded.is_empty());
        // Should not contain padding for this input
        assert!(!encoded.ends_with('='));
    });
}

#[test]
pub fn test_decode_base64_async() {
    smol::block_on(async {
        let data = b"Hello!";
        let encoded = data::encode_base64_async(data).await;
        let decoded = data::decode_base64_async(&encoded).await.unwrap();
        assert_eq!(decoded, data);
    });
}

#[test]
pub fn test_with_timeout() {
    smol::block_on(async {
        // Test success case
        let result = patterns::with_timeout(std::time::Duration::from_secs(1), async {
            smol::Timer::after(std::time::Duration::from_millis(100)).await;
            42
        })
        .await;
        assert!(matches!(result, Ok(42)));

        // Test timeout case
        let result = patterns::with_timeout(std::time::Duration::from_millis(50), async {
            smol::Timer::after(std::time::Duration::from_millis(100)).await;
            42
        })
        .await;
        assert!(matches!(result, Err(patterns::TimeoutError::Timeout)));
    });
}

#[test]
pub fn test_retry_async() {
    smol::block_on(async {
        let result = patterns::retry_async(|| async { Ok::<_, std::io::Error>("success") }).await;
        assert!(matches!(result, Ok("success")));
    });
}

#[test]
pub fn test_retry_async_with_config() {
    smol::block_on(async {
        let result =
            patterns::retry_async_with_config(3, std::time::Duration::from_millis(10), || async {
                Ok::<_, std::io::Error>("success")
            })
            .await;
        assert!(matches!(result, Ok("success")));
    });
}

#[test]
pub fn test_async_circuit_breaker() {
    smol::block_on(async {
        let breaker = patterns::AsyncCircuitBreaker::new();
        let result = breaker
            .execute(|| async { Ok::<_, std::io::Error>("success") })
            .await;
        assert!(matches!(result, Ok("success")));
    });
}

#[test]
pub fn test_async_circuit_breaker_with_config() {
    let breaker = patterns::AsyncCircuitBreaker::with_config(3, std::time::Duration::from_secs(30));
    // Test that breaker was created successfully
    smol::block_on(async {
        let result = breaker
            .execute(|| async { Ok::<_, std::io::Error>("test") })
            .await;
        assert!(matches!(result, Ok("test")));
    });
}

#[test]
pub fn test_async_circuit_breaker_builder() {
    let breaker = patterns::AsyncCircuitBreaker::builder()
        .threshold(10)
        .timeout(std::time::Duration::from_secs(120))
        .build();
    // Test that breaker was created successfully
    smol::block_on(async {
        let result = breaker
            .execute(|| async { Ok::<_, std::io::Error>("test") })
            .await;
        assert!(matches!(result, Ok("test")));
    });
}

#[test]
pub fn test_parallel_process_async() {
    smol::block_on(async {
        let data = vec![1, 2, 3, 4, 5];
        let results = patterns::parallel_process_async(data, |x| x * 2).await;
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    });
}

#[test]
pub fn test_async_resource_pool() {
    let pool = patterns::AsyncResourcePool::new(|| String::from("resource"));
    let guard = pool.acquire();
    assert_eq!(*guard, "resource");
    // Resource is automatically returned when guard is dropped
}

#[test]
pub fn test_async_resource_pool_with_config() {
    let pool = patterns::AsyncResourcePool::with_config(|| 42, 5);
    let guard = pool.acquire();
    assert_eq!(*guard, 42);
}

#[test]
pub fn test_async_resource_pool_builder() {
    let pool = patterns::AsyncResourcePool::builder(|| String::from("test"))
        .max_size(20)
        .build();
    let guard = pool.acquire();
    assert_eq!(*guard, "test");
}

#[test]
pub fn test_async_stream_processor() {
    smol::block_on(async {
        let processor = patterns::AsyncStreamProcessor::new(|batch: Vec<i32>| async move {
            assert_eq!(batch.len(), 2);
            assert_eq!(batch, vec![1, 2]);
        });
        processor.push(1).await;
        processor.push(2).await;
        processor.flush().await; // Should trigger processing
    });
}

#[test]
pub fn test_async_stream_processor_with_config() {
    let processor = patterns::AsyncStreamProcessor::with_config(
        |_batch: Vec<String>| async move {
            // Process batch
        },
        50,
    );
    // Test that processor was created successfully
    smol::block_on(async {
        processor.push("test".to_string()).await;
        processor.flush().await;
    });
}

#[test]
pub fn test_async_stream_processor_builder() {
    let processor = patterns::AsyncStreamProcessor::builder(|_batch: Vec<i32>| async move {
        // Process batch
    })
    .buffer_size(200)
    .build();
    // Test that processor was created successfully
    smol::block_on(async {
        processor.push(42).await;
        processor.flush().await;
    });
}

#[test]
pub fn test_async_performance_monitor() {
    smol::block_on(async {
        let monitor = patterns::AsyncPerformanceMonitor::new();
        let result = monitor
            .time_operation("test_op", || async {
                smol::Timer::after(std::time::Duration::from_millis(10)).await;
                42
            })
            .await;
        assert_eq!(result, 42);

        let stats = monitor.stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].0, "test_op");
        assert!(stats[0].1 >= std::time::Duration::from_millis(10));
    });
}

#[test]
pub fn test_async_performance_monitor_clear() {
    let monitor = patterns::AsyncPerformanceMonitor::new();
    // Clear should work on empty monitor
    monitor.clear();
    assert_eq!(monitor.operation_count(), 0);
}

#[test]
pub fn test_async_performance_monitor_avg_duration() {
    let monitor = patterns::AsyncPerformanceMonitor::new();
    // Should return None for non-existent operation
    assert_eq!(monitor.avg_duration("nonexistent"), None);
}

#[test]
pub fn test_async_performance_monitor_operations_for() {
    let monitor = patterns::AsyncPerformanceMonitor::new();
    let operations = monitor.operations_for("test");
    assert!(operations.is_empty());
}

#[test]
pub fn test_async_task_spawner() {
    smol::block_on(async {
        let spawner = tasks::AsyncTaskSpawner::new();
        spawner.spawn(|| async {
            // Simple task
        });
        spawner.wait_all().await;
    });
}

#[test]
pub fn test_async_task_spawner_builder() {
    let token = smol_cancellation_token::CancellationToken::new();
    let spawner = tasks::AsyncTaskSpawner::builder()
        .with_cancellation_token(token)
        .build();
    // Should create spawner successfully
    drop(spawner);
}

#[test]
pub fn test_async_task_spawner_cancel() {
    let spawner = tasks::AsyncTaskSpawner::new();
    spawner.cancel();
    // Test that cancel doesn't panic
}

#[test]
pub fn test_async_task_group() {
    smol::block_on(async {
        let group = tasks::AsyncTaskGroup::new();
        group.add_task(|| async {
            // Simple task
        });
        group.wait_all().await;
    });
}

#[test]
pub fn test_async_task_group_cancel() {
    let group = tasks::AsyncTaskGroup::new();
    group.cancel();
    // Test that cancel doesn't panic
}

#[test]
pub fn test_async() {
    test_sleep_for();
    test_race();
    test_join();
    test_create_cancellation_token();
    test_with_cancellation();
    test_create_mutex();
    test_compress_data_async();
    test_decompress_data_async();
    test_serialize_async();
    test_deserialize_async();
    test_hash_data_async();
    test_encode_base64_async();
    test_decode_base64_async();
    test_with_timeout();
    test_retry_async();
    test_retry_async_with_config();
    test_async_circuit_breaker();
    test_async_circuit_breaker_with_config();
    test_async_circuit_breaker_builder();
    test_parallel_process_async();
    test_async_resource_pool();
    test_async_resource_pool_with_config();
    test_async_resource_pool_builder();
    test_async_stream_processor();
    test_async_stream_processor_with_config();
    test_async_stream_processor_builder();
    test_async_performance_monitor();
    test_async_performance_monitor_clear();
    test_async_performance_monitor_avg_duration();
    test_async_performance_monitor_operations_for();
    test_async_task_spawner();
    test_async_task_spawner_builder();
    test_async_task_spawner_cancel();
    test_async_task_group();
    test_async_task_group_cancel();
}
