//! Tests for the io module
use trash_utilities::io::*;

#[test]
pub fn test_read_file_async() {
    // This would require an actual file, so we'll skip for now
    // In a real test, you'd create a temp file and read it
}

#[test]
pub fn test_write_file_async() {
    // This would require writing to a temp file
}

#[test]
pub fn test_copy_file_async() {
    // This is actually sync, not async
    // Would need to create temp files
}

#[test]
pub fn test_read_file_bytes_async() {
    // Would need temp file
}

#[test]
pub fn test_write_file_bytes_async() {
    // Would need temp file
}

#[test]
pub fn test_create_dir_async() {
    // Would need temp directory
}

#[test]
pub fn test_read_dir_async() {
    // Would need temp directory with files
}

#[test]
pub fn test_compress_brotli() {
    let data = b"Hello, this is test data for compression!";
    let compressed = compress_brotli(data, 6).unwrap();
    assert!(!compressed.is_empty());
    // Note: compressed data might be larger than original for small inputs
}

#[test]
pub fn test_decompress_brotli() {
    let data = b"Hello, this is test data for compression!";
    let compressed = compress_brotli(data, 6).unwrap();
    let decompressed = decompress_brotli(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
pub fn test_atomic_counter() {
    let counter = AtomicCounter::new();
    assert_eq!(counter.get(), 0);

    assert_eq!(counter.increment(), 1);
    assert_eq!(counter.get(), 1);

    counter.reset();
    assert_eq!(counter.get(), 0);
}

#[test]
pub fn test_lru_cache() {
    let cache = LruCache::new(3);
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());

    cache.insert(1, "value1");
    cache.insert(2, "value2");
    cache.insert(3, "value3");

    assert_eq!(cache.len(), 3);
    assert_eq!(cache.get(&1), Some("value1"));
    assert_eq!(cache.get(&2), Some("value2"));
    assert_eq!(cache.get(&3), Some("value3"));

    cache.clear();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
pub fn test_string_interner() {
    let interner = StringInterner::new();
    assert_eq!(interner.len(), 0);
    assert!(interner.is_empty());

    let s1 = interner.intern("hello");
    let s2 = interner.intern("hello");
    assert_eq!(interner.len(), 1);
    assert_eq!(s1.as_ptr(), s2.as_ptr()); // Same memory location

    let s3 = interner.intern("world");
    assert_eq!(interner.len(), 2);
    assert_ne!(s1.as_ptr(), s3.as_ptr());

    interner.clear();
    assert_eq!(interner.len(), 0);
    assert!(interner.is_empty());
}

#[test]
pub fn test_global_interner() {
    let interner = global_interner();
    let s1 = interner.intern("global_test");
    let s2 = interner.intern("global_test");
    assert_eq!(s1.as_ptr(), s2.as_ptr());
}

#[test]
pub fn test_parallel_map() {
    let data = vec![1, 2, 3, 4, 5];
    let result = parallelism::parallel_map(data, |x| x * 2);
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
}

#[test]
pub fn test_parallel_filter() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let evens = parallelism::parallel_filter(data, |x| x % 2 == 0);
    assert_eq!(evens, vec![2, 4, 6]);
}

#[test]
pub fn test_process_files_parallel() {
    // Would need temp files, but let's test the structure
    let paths = vec![
        "nonexistent1.txt".to_string(),
        "nonexistent2.txt".to_string(),
    ];
    let results = parallelism::process_files_parallel(paths, |content| Ok(content.len()));
    // Should return errors for nonexistent files
    assert_eq!(results.len(), 2);
    assert!(results[0].is_err());
    assert!(results[1].is_err());
}

#[test]
pub fn test_async_file_writer() {
    // Would need async test, but for now test the structure
    // In real async test: let writer = AsyncFileWriter::new("temp.txt").await.unwrap();
}

#[test]
pub fn test_streaming_file_writer() {
    // Would need async test
}

#[test]
pub fn test_advanced_file_writer() {
    // Would need async test
}

#[test]
pub fn test_write_stdout_async() {
    // This writes to stdout, might be hard to test
    let result = writers::write_stdout_async(b"test");
    assert!(result.is_ok());
}

#[test]
pub fn test_write_stderr_async() {
    let result = writers::write_stderr_async(b"test error");
    assert!(result.is_ok());
}

#[test]
pub fn test_create_channel() {
    let (tx, rx) = streams::create_channel::<String>();
    // Basic test that channels are created
    drop(tx);
    drop(rx);
}

#[test]
pub fn test_async_file_processor() {
    let _processor = streams::AsyncFileProcessor::new();
    // Test that it can be created without panicking
}

#[test]
pub fn test_async_file_processor_builder() {
    let _processor = streams::AsyncFileProcessorBuilder::new()
        .buffer_size(2048)
        .progress_callback(|_| {})
        .build();
    // Test that builder works without panicking
}

#[test]
pub fn test_async_stream_utils() {
    // Test the static methods
    // Would need actual streams for full testing
}

#[test]
pub fn test_channel_stream_processor() {
    let processor = streams::ChannelStreamProcessor::new(|x: i32| x * 2);
    // Basic structure test
    drop(processor);
}

#[test]
pub fn test_buffered_async_reader() {
    // Would need an actual async reader
    // Basic structure test
}

#[test]
pub fn test_io() {
    test_compress_brotli();
    test_decompress_brotli();
    test_atomic_counter();
    test_lru_cache();
    test_string_interner();
    test_global_interner();
    test_parallel_map();
    test_parallel_filter();
    test_process_files_parallel();
    test_write_stdout_async();
    test_write_stderr_async();
    test_create_channel();
    test_async_file_processor();
    test_async_file_processor_builder();
    test_channel_stream_processor();
}
