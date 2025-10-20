//! High-performance I/O utilities for asynchronous file operations, compression, and caching.
//!
//! This module provides comprehensive async I/O operations using smol runtime,
//! compression algorithms (Brotli, Gzip), cryptographic hashing, and thread-safe
//! data structures for caching and concurrency. Designed for high-throughput
//! applications requiring efficient data processing and storage.
//!
//! ## Features
//!
//! - **Async File Operations**: Non-blocking file I/O with smol runtime
//! - **Compression Support**: Brotli and Gzip compression/decompression
//! - **Parallel Processing**: Concurrent file operations and batch processing
//! - **Streaming I/O**: Efficient stream processing and channel utilities
//! - **Advanced Writers**: Buffered and compressed file writing
//! - **Thread-Safe Caching**: LRU caches and atomic counters
//! - **Memory Efficient**: Optimized memory usage with string interning
//! - **Cryptographic Operations**: Hashing and integrity checking
//!
//! ## Architecture
//!
//! ### Core Components
//! - **`utils`**: Basic file operations, compression, counters, and caching
//! - **`parallelism`**: Parallel file processing using existing parallel module
//! - **`writers`**: Async file writers with buffering and compression
//! - **`streams`**: Streaming I/O operations and channel utilities
//!
//! ## Examples
//!
//! ### Basic Async File Operations
//! ```rust,no_run
//! use trash_utilities::io::utils::*;
//! use smol;
//!
//! # smol::block_on(async {
//! // Read and write files asynchronously
//! let content = read_file_async("input.txt").await.unwrap();
//! write_file_async("output.txt", &content).await.unwrap();
//!
//! // Directory operations
//! create_dir_async("backup").await.unwrap();
//! let entries = read_dir_async(".").await.unwrap();
//! println!("Found {} files/directories", entries.len());
//!
//! // File metadata
//! let size = get_file_size_async("input.txt").await.unwrap();
//! let modified = get_file_modified_async("input.txt").await.unwrap();
//! println!("File size: {} bytes, modified: {:?}", size, modified);
//! # });
//! ```
//!
//! ### Compression and Archiving
//! ```rust,no_run
//! use trash_utilities::io::utils::*;
//! use smol;
//!
//! # smol::block_on(async {
//! // Compress data
//! let data = b"This is some data to compress efficiently.";
//! let compressed = compress_data_brotli(data, 6).await.unwrap();
//!
//! // Write compressed data to file
//! write_file_async("data.compressed", &compressed).await.unwrap();
//!
//! // Read and decompress
//! let compressed_data = read_file_async("data.compressed").await.unwrap();
//! let decompressed = decompress_data_brotli(&compressed_data).await.unwrap();
//!
//! assert_eq!(data.to_vec(), decompressed);
//! println!("Compression ratio: {:.2}", compressed.len() as f64 / data.len() as f64);
//! # });
//! ```
//!
//! ### Parallel File Processing
//! ```rust,no_run
//! use trash_utilities::io::parallelism::*;
//! use smol;
//!
//! # smol::block_on(async {
//! // Process multiple files in parallel
//! let files = vec!["file1.txt", "file2.txt", "file3.txt"];
//!
//! // Read all files concurrently
//! let contents = read_files_parallel(&files).await.unwrap();
//!
//! // Process contents (e.g., count lines)
//! let line_counts = process_files_parallel(contents, |content| {
//!     content.lines().count()
//! }).await;
//!
//! for (i, count) in line_counts.iter().enumerate() {
//!     println!("File {}: {} lines", i + 1, count);
//! }
//!
//! // Write results in parallel
//! let results: Vec<String> = line_counts.iter()
//!     .enumerate()
//!     .map(|(i, &count)| format!("File {}: {} lines\n", i + 1, count))
//!     .collect();
//!
//! write_files_parallel(&files, &results).await.unwrap();
//! # });
//! ```
//!
//! ### Streaming Data Processing
//! ```rust,no_run
//! use trash_utilities::io::streams::*;
//! use smol;
//!
//! # smol::block_on(async {
//! // Create a streaming processor
//! let mut processor = create_stream_processor();
//!
//! // Process data in chunks
//! let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//! let chunks = data.chunks(3);
//!
//! for chunk in chunks {
//!     processor.process_chunk(chunk).await.unwrap();
//! }
//!
//! // Get final result
//! let result = processor.finalize().await.unwrap();
//! println!("Processed {} elements", result);
//! # });
//! ```
//!
//! ### Advanced File Writing
//! ```rust,no_run
//! use trash_utilities::io::writers::*;
//! use smol;
//!
//! # smol::block_on(async {
//! // Create a buffered, compressed writer
//! let writer = create_compressed_writer("output.gz", 6).await.unwrap();
//!
//! // Write data efficiently
//! let data = b"Large amounts of data to write efficiently...";
//! writer.write_chunk(data).await.unwrap();
//! writer.write_chunk(data).await.unwrap();
//!
//! // Flush and close
//! writer.flush().await.unwrap();
//! writer.close().await.unwrap();
//!
//! // Verify file was created and compressed
//! let compressed_size = get_file_size_async("output.gz").await.unwrap();
//! println!("Compressed file size: {} bytes", compressed_size);
//! # });
//! ```
//!
//! ### Thread-Safe Caching and Counters
//! ```rust
//! use trash_utilities::io::utils::*;
//! use std::sync::Arc;
//! use std::thread;
//!
//! // Shared cache across threads
//! let cache = Arc::new(LruCache::new(1000));
//!
//! // Atomic counter for statistics
//! let counter = Arc::new(AtomicCounter::new());
//!
//! let mut handles = vec![];
//!
//! for i in 0..10 {
//!     let cache_clone = Arc::clone(&cache);
//!     let counter_clone = Arc::clone(&counter);
//!
//!     let handle = thread::spawn(move || {
//!         // Use cache and counter from multiple threads
//!         cache_clone.insert(format!("key_{}", i), format!("value_{}", i));
//!         counter_clone.increment();
//!     });
//!
//!     handles.push(handle);
//! }
//!
//! // Wait for all threads
//! for handle in handles {
//!     handle.join().unwrap();
//! }
//!
//! assert_eq!(counter.get(), 10);
//! assert_eq!(cache.len(), 10);
//! ```
//!
//! ### String Interning for Memory Efficiency
//! ```rust
//! use trash_utilities::io::utils::*;
//!
//! // Create string interner for memory efficiency
//! let interner = StringInterner::new();
//!
//! // Intern frequently used strings
//! let user1 = interner.intern("frequently_used_username");
//! let user2 = interner.intern("frequently_used_username");
//!
//! // Both references point to the same memory location
//! assert_eq!(user1.as_ptr(), user2.as_ptr());
//! assert_eq!(interner.len(), 1); // Only one copy in memory
//!
//! // Useful for caching, configuration, etc.
//! let config_keys = vec![
//!     "database.host",
//!     "database.port",
//!     "cache.size",
//!     "cache.ttl",
//! ];
//!
//! let interned_keys: Vec<_> = config_keys.iter()
//!     .map(|key| interner.intern(key))
//!     .collect();
//!
//! println!("Interned {} unique strings", interner.len());
//! ```

// Submodule declarations
pub mod parallelism;
pub mod streams;
pub mod utils;
pub mod writers;

// Re-exports for backward compatibility and convenience
pub use parallelism::*;
pub use streams::*;
pub use utils::*;
pub use writers::*;
