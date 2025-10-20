//! Tests for the io module
use trash_parallelism::io::*;

#[test]
pub fn test_read_file_async() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let content = "Test content for reading";

        std::fs::write(path, content).unwrap();

        let read_content = read_file_async(path).await.unwrap();
        assert_eq!(read_content, content);
    });
}

#[test]
pub fn test_write_file_async() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let content = "Hello, async world!";

        write_file_async(path, content).await.unwrap();

        let read_content = std::fs::read_to_string(path).unwrap();
        assert_eq!(read_content, content);
    });
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
pub fn test_async_file_writer_with_config() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::AsyncFileWriter::with_config(path, 10, false).await.unwrap();
        assert!(!writer.is_compressed());
        assert_eq!(writer.bytes_written(), 0);

        // Write data larger than buffer to trigger auto-flush
        writer.write(b"Hello, world! This is a longer message.").await.unwrap();
        writer.flush().await.unwrap();

        assert_eq!(writer.bytes_written(), 39);

        let content = std::fs::read(path).unwrap();
        assert_eq!(content, b"Hello, world! This is a longer message.");
    });
}

#[test]
pub fn test_async_file_writer_write_json() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::AsyncFileWriter::new(path).await.unwrap();

        #[allow(clippy::items_after_statements)]
        #[derive(serde::Serialize)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        writer.write_json(&data).await.unwrap();
        writer.flush().await.unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("\"name\":\"test\""));
        assert!(content.contains("\"value\":42"));
    });
}

#[test]
pub fn test_async_file_writer_multiple_flushes() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::AsyncFileWriter::with_config(path, 5, false).await.unwrap();

        writer.write(b"Hi").await.unwrap();
        writer.flush().await.unwrap();
        assert_eq!(writer.bytes_written(), 2);

        writer.write(b"Hello").await.unwrap();
        writer.flush().await.unwrap();
        assert_eq!(writer.bytes_written(), 7);

        let content = std::fs::read(path).unwrap();
        assert_eq!(content, b"HiHello");
    });
}

#[test]
pub fn test_streaming_file_writer_compressed() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::StreamingFileWriter::new(path, 1024, true).await.unwrap();
        assert_eq!(writer.chunk_size(), 1024);

        writer.write_chunk(b"{\"data\": \"This is test data for compression\"}").await.unwrap();
        assert!(writer.bytes_written() > 0); // Compressed data should be written

        // Since it's compressed, we can't easily check the content, but ensure bytes were written
        let content = std::fs::read(path).unwrap();
        assert!(!content.is_empty());
    });
}

#[test]
pub fn test_streaming_file_writer_large_chunk() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::StreamingFileWriter::new(path, 10, false).await.unwrap();

        let large_data = b"This is a larger chunk of data that exceeds the chunk size setting.";
        writer.write_chunk(large_data).await.unwrap();

        assert_eq!(writer.bytes_written(), large_data.len() as u64);

        let content = std::fs::read(path).unwrap();
        assert_eq!(content, large_data);
    });
}

#[test]
pub fn test_advanced_file_writer_with_progress() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let progress_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let progress_clone = progress_count.clone();

        let mut writer = writers::AdvancedFileWriter::new(
            path,
            5, // Small buffer
            false,
            Some(Box::new(move |_| {
                let mut count = progress_clone.lock().unwrap();
                *count += 1;
            })),
            false,
        ).await.unwrap();

        writer.write_with_progress(b"Hello, world! This is longer.").await.unwrap();
        writer.flush().await.unwrap();

        assert_eq!(writer.bytes_written(), 29);
        // Progress callback should have been called multiple times due to small buffer
        assert!(*progress_count.lock().unwrap() > 0);

        let content = std::fs::read(path).unwrap();
        assert_eq!(content, b"Hello, world! This is longer.");
    });
}

#[test]
pub fn test_advanced_file_writer_compressed() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = writers::AdvancedFileWriter::new(
            path,
            1024,
            true, // compressed
            None,
            false,
        ).await.unwrap();

        writer.write_with_progress(b"Repeated data for compression: test test test test").await.unwrap();
        writer.flush().await.unwrap();

        assert!(writer.bytes_written() > 0);
        let content = std::fs::read(path).unwrap();
        assert!(!content.is_empty());
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
pub fn test_async_file_processor_with_progress_callback() {
    smol::block_on(async {
        // Create temp file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        std::fs::write(path, b"Hello, world!").unwrap();

        let progress_called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let progress_clone = progress_called.clone();

        let processor = streams::AsyncFileProcessor::with_config(4096, Some(Box::new(move |_| {
            *progress_clone.lock().unwrap() = true;
        })));

        let results = processor.process_file(path, |chunk| chunk.len()).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 13);
        assert!(*progress_called.lock().unwrap());
    });
}

#[test]
pub fn test_async_file_processor_with_progress_callback_async() {
    smol::block_on(async {
        // Create temp file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        std::fs::write(path, b"Hello, world!").unwrap();

        let progress_called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let progress_clone = progress_called.clone();

        let processor = streams::AsyncFileProcessor::with_config(4096, Some(Box::new(move |_| {
            *progress_clone.lock().unwrap() = true;
        })));

        let results = processor.process_file_async(path, |chunk| async move { Ok(chunk.len()) }).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 13);
        assert!(*progress_called.lock().unwrap());
    });
}

#[test]
pub fn test_async_file_processor_default() {
    smol::block_on(async {
        // Create temp file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        std::fs::write(path, b"Hello, world!").unwrap();

        let processor = streams::AsyncFileProcessor::default();
        let results = processor.process_file(path, |chunk| chunk.len()).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 13);
    });
}

#[test]
pub fn test_async_file_processor_builder() {
    smol::block_on(async {
        // Create temp file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        std::fs::write(path, b"Hello, world!").unwrap();

        let progress_called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let progress_clone = progress_called.clone();

        let processor = streams::AsyncFileProcessor::builder()
            .buffer_size(1024)
            .progress_callback(move |_| {
                *progress_clone.lock().unwrap() = true;
            })
            .build();

        let results = processor.process_file(path, |chunk| chunk.len()).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 13);
        assert!(*progress_called.lock().unwrap());
    });
}

#[test]
pub fn test_process_file_async_function() {
    smol::block_on(async {
        // Create temp file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        std::fs::write(path, b"Hello, world!").unwrap();

        let content_received = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let content_clone = content_received.clone();

        streams::process_file_async(path, move |content| {
            *content_clone.lock().unwrap() = content;
        }).await.unwrap();

        assert_eq!(*content_received.lock().unwrap(), "Hello, world!");
    });
}

#[test]
pub fn test_channel_stream_processor_start_background_processing() {
    smol::block_on(async {
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        let counter_clone = counter.clone();

        let processor = streams::ChannelStreamProcessor::new(move |x: i32| {
            *counter_clone.lock().unwrap() += x;
            x
        });

        processor.start_background_processing();

        processor.send(5).await.unwrap();
        processor.send(10).await.unwrap();

        // Give some time for background processing
        smol::Timer::after(std::time::Duration::from_millis(100)).await;

        // Since background processing just processes and drops, we can't check the result
        // But we can check that the processor was called by using a side effect
        // Wait a bit more
        smol::Timer::after(std::time::Duration::from_millis(100)).await;

        // The counter should have been incremented
        assert_eq!(*counter.lock().unwrap(), 15);
    });
}

#[test]
pub fn test_buffered_async_reader_eof() {
    smol::block_on(async {
        use futures_lite::io::Cursor;
        let data = b""; // Empty data to simulate EOF
        let cursor = Cursor::new(data.to_vec());
        let mut reader = streams::BufferedAsyncReader::new(cursor, 20);

        let buffer = reader.read_buffer().await.unwrap();
        assert_eq!(buffer.len(), 0);
    });
}

#[test]
pub fn test_buffered_async_reader_buffer_size() {
    smol::block_on(async {
        use futures_lite::io::Cursor;
        let data = b"Hello";
        let cursor = Cursor::new(data.to_vec());
        let reader = streams::BufferedAsyncReader::new(cursor, 42);

        assert_eq!(reader.buffer_size(), 42);
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
pub fn test_copy_file_async() {
    let temp_source = tempfile::NamedTempFile::new().unwrap();
    let temp_dest = tempfile::NamedTempFile::new().unwrap();
    let source_path = temp_source.path().to_str().unwrap();
    let dest_path = temp_dest.path().to_str().unwrap();
    let content = "Content to copy";

    std::fs::write(source_path, content).unwrap();

    let bytes_copied = copy_file_async(source_path, dest_path).unwrap();
    assert_eq!(bytes_copied, content.len() as u64);

    let copied_content = std::fs::read_to_string(dest_path).unwrap();
    assert_eq!(copied_content, content);
}

#[test]
pub fn test_write_file_bytes_async() {
    smol::block_on(async {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let data = b"Hello, bytes world!";

        write_file_bytes_async(path, data).await.unwrap();

        let read_data = std::fs::read(path).unwrap();
        assert_eq!(read_data, data);
    });
}

#[test]
pub fn test_create_dir_async() {
    smol::block_on(async {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();
        let new_dir_path = format!("{base_path}/new_test_dir");

        create_dir_async(&new_dir_path).await.unwrap();

        assert!(std::fs::metadata(&new_dir_path).unwrap().is_dir());
    });
}

#[test]
pub fn test_read_dir_async() {
    smol::block_on(async {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap();

        // Create some test files
        std::fs::write(format!("{dir_path}/file1.txt"), "content1").unwrap();
        std::fs::write(format!("{dir_path}/file2.txt"), "content2").unwrap();

        let entries = read_dir_async(dir_path).await.unwrap();

        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&"file1.txt".to_string()));
        assert!(entries.contains(&"file2.txt".to_string()));
    });
}

#[test]
pub fn test_atomic_counter_default() {
    let counter = AtomicCounter::default();
    assert_eq!(counter.get(), 0);
}

#[test]
pub fn test_string_interner_default() {
    let interner = StringInterner::default();
    assert_eq!(interner.len(), 0);
    assert!(interner.is_empty());
}

#[test]
pub fn test_io() {
    test_read_file_async();
    test_write_file_async();
    test_copy_file_async();
    test_write_file_bytes_async();
    test_create_dir_async();
    test_read_dir_async();
    test_atomic_counter_default();
    test_string_interner_default();
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
    test_async_file_writer_with_config();
    test_async_file_writer_write_json();
    test_async_file_writer_multiple_flushes();
    test_streaming_file_writer_compressed();
    test_streaming_file_writer_large_chunk();
    test_advanced_file_writer_with_progress();
    test_advanced_file_writer_compressed();
    test_async_file_processor_with_progress_callback();
    test_async_file_processor_with_progress_callback_async();
    test_async_file_processor_default();
    test_async_file_processor_builder();
    test_process_file_async_function();
    test_channel_stream_processor_start_background_processing();
    test_buffered_async_reader_eof();
    test_buffered_async_reader_buffer_size();
}
