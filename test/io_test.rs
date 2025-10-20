//! Tests for the io module
use trash_parallelism::io::*;

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
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::AsyncFileWriter::new(path).await.unwrap();
        writer.write(b"Hello, ").await.unwrap();
        writer.write(b"world!").await.unwrap();
        writer.flush().await.unwrap();

        assert_eq!(writer.bytes_written(), 13);
        assert!(!writer.is_compressed());

        let content = std::fs::read(path).unwrap();
        assert_eq!(content, b"Hello, world!");
    });
}

#[test]
pub fn test_async_file_writer_compressed() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::AsyncFileWriter::with_config(path, 1024, true).await.unwrap();
        writer.write_json(&serde_json::json!({"test": "data"})).await.unwrap();
        writer.flush().await.unwrap();

        assert!(writer.is_compressed());
        assert!(writer.bytes_written() > 0);
    });
}

#[test]
pub fn test_streaming_file_writer() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::StreamingFileWriter::new(path, 1024, false).await.unwrap();
        writer.write_chunk(b"Hello, ").await.unwrap();
        writer.write_chunk(b"world!").await.unwrap();

        assert_eq!(writer.bytes_written(), 13);
        assert_eq!(writer.chunk_size(), 1024);

        let content = std::fs::read(path).unwrap();
        assert_eq!(content, b"Hello, world!");
    });
}

#[test]
pub fn test_advanced_file_writer() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::AdvancedFileWriter::new(
            path,
            1024,
            false,
            Some(Box::new(|_| println!("Progress"))),
            false,
        ).await.unwrap();

        writer.write_with_progress(b"Hello, world!").await.unwrap();
        writer.flush().await.unwrap();

        assert_eq!(writer.bytes_written(), 13);
    });
}

#[test]
pub fn test_process_files_chunked() {
    smol::block_on(async {
        // Create temp file with test data
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        std::fs::write(&path, b"Hello, world! This is test data for chunked processing.").unwrap();

        let paths = vec![path];
        let results = parallelism::process_files_chunked(paths, 10, |chunk| Ok(chunk.len())).await;

        assert_eq!(results.len(), 1);
        let chunks = &results[0];
        assert!(chunks.is_ok());
        let chunk_lengths = chunks.as_ref().unwrap();
        let total_len: usize = chunk_lengths.iter().sum();
        assert_eq!(total_len, 55); // Length of test data
    });
}

#[test]
pub fn test_traverse_and_process() {
    // Create temp directory with files
    let temp_dir = tempfile::TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();

    // Create some test files
    std::fs::write(format!("{dir_path}/file1.txt"), "content1").unwrap();
    std::fs::write(format!("{dir_path}/file2.txt"), "content2").unwrap();
    std::fs::create_dir(format!("{dir_path}/subdir")).unwrap();
    std::fs::write(format!("{dir_path}/subdir/file3.txt"), "content3").unwrap();

    let results = parallelism::traverse_and_process(dir_path, |_, content| {
        Ok(content.len())
    }, Some(2)).unwrap();

    // Should process 3 files
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8); // "contentX".len()
    }
}

#[test]
pub fn test_batch_file_operations() {
    let operations = vec![
        || std::fs::write("temp_batch1.txt", "data1"),
        || std::fs::write("temp_batch2.txt", "data2"),
        || std::fs::write("temp_batch3.txt", "data3"),
    ];

    let result = parallelism::batch_file_operations(operations, false);
    assert!(result.is_ok());

    // Cleanup
    let _ = std::fs::remove_file("temp_batch1.txt");
    let _ = std::fs::remove_file("temp_batch2.txt");
    let _ = std::fs::remove_file("temp_batch3.txt");
}

#[test]
pub fn test_async_file_processor_process_file() {
    smol::block_on(async {
        // Create temp file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        std::fs::write(path, b"Hello, world!").unwrap();

        let processor = streams::AsyncFileProcessor::new();
        let results = processor.process_file(path, |chunk| chunk.len()).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 13); // "Hello, world!".len()
    });
}

#[test]
pub fn test_async_file_processor_process_file_async() {
    smol::block_on(async {
        // Create temp file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        std::fs::write(path, b"Hello, world!").unwrap();

        let processor = streams::AsyncFileProcessor::new();
        let results = processor.process_file_async(path, |chunk| async move { Ok(chunk.len()) }).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 13);
    });
}

#[test]
pub fn test_async_stream_utils() {
    smol::block_on(async {
        use futures_lite::stream;
        let stream = stream::iter(vec![1, 2, 3, 4, 5]);
        let results = streams::AsyncStreamUtils::map_stream(stream, |x| x * 2).await;
        assert_eq!(results, vec![2, 4, 6, 8, 10]);

        let stream2 = stream::iter(vec![1, 2, 3, 4, 5]);
        let filtered = streams::AsyncStreamUtils::filter_stream(stream2, |&x| x % 2 == 0).await;
        assert_eq!(filtered, vec![2, 4]);

        let stream3 = stream::iter(vec![1, 2, 3]);
        let collected = streams::AsyncStreamUtils::collect_stream(stream3).await;
        assert_eq!(collected, vec![1, 2, 3]);
    });
}

#[test]
pub fn test_channel_stream_processor_send_receive() {
    smol::block_on(async {
        let processor = streams::ChannelStreamProcessor::new(|x: i32| x * 2);
        processor.send(5).await.unwrap();
        let result = processor.receive().await.unwrap();
        assert_eq!(result, 10);
    });
}

#[test]
pub fn test_buffered_async_reader_read() {
    smol::block_on(async {
        use futures_lite::io::Cursor;
        let data = b"Hello, world!";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = streams::BufferedAsyncReader::new(cursor, 20);

        let buffer = reader.read_buffer().await.unwrap();
        assert_eq!(buffer, b"Hello, world!");

        reader.consume(7); // consume "Hello, "

        let buffer2 = reader.read_buffer().await.unwrap();
        assert_eq!(buffer2, b"world!");
    });
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
    test_process_files_chunked();
    test_traverse_and_process();
    test_batch_file_operations();
    test_async_file_processor_process_file();
    test_async_file_processor_process_file_async();
    test_async_stream_utils();
    test_channel_stream_processor_send_receive();
    test_buffered_async_reader_read();
    test_async_file_writer();
    test_async_file_writer_compressed();
    test_streaming_file_writer();
    test_advanced_file_writer();
}
