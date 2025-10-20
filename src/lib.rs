//! # Trash Utilities
//!
//! A high-performance Rust library providing comprehensive async, threading,
//! memory management, and utility functions for building efficient applications.
//!
//! ## Overview
//!
//! Trash Utilities is a batteries-included Rust library designed for high-performance
//! applications requiring efficient async operations, parallel processing, memory management,
//! and system-level utilities. Built with performance and ergonomics in mind.
//!
//! ## Key Features
//!
//! - **🚀 Async Operations**: High-performance async utilities with smol, futures-lite, and crossfire
//! - **⚡ Threading**: Parallel processing with work-stealing schedulers and fork-join patterns
//! - **💾 Memory Management**: Efficient allocation with mimalloc and custom memory pools
//! - **📡 Channels**: Advanced channel communication with monitoring and serialization
//! - **🖥️ System Utilities**: Time handling, environment variables, file system operations
//! - **📊 Data Processing**: Parsing, serialization, and data manipulation
//! - **🔄 I/O Operations**: File and network I/O with async support
//! - **📝 Logging**: Structured logging with performance monitoring
//! - **🛠️ Utilities**: Compression, hashing, JSON handling, and more
//!
//! ## Quick Start
//!
//! ### Basic Usage
//! ```rust
//! use trash_utilities::*;
//!
//! // Async task spawning
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! spawn_task!("my_task", async {
//!     println!("Hello from async task!");
//!     Ok(())
//! });
//! # Ok(())
//! # }
//!
//! // Parallel processing
//! let data = vec![1, 2, 3, 4, 5];
//! let result = parallel_map(data, |x| x * 2);
//! assert_eq!(result, vec![2, 4, 6, 8, 10]);
//!
//! // System utilities
//! let now = current_utc_time();
//! let home = read_env_var("HOME").unwrap_or_default();
//! println!("Current time: {}, Home: {}", now, home);
//! # }
//! ```
//!
//! ### Advanced Example: High-Performance Data Processing
//! ```rust,no_run
//! use trash_utilities::*;
//! use smol;
//!
//! # smol::block_on(async {
//! // Parallel data processing pipeline
//! let raw_data = vec![1u32; 10000];
//!
//! // Process in parallel
//! let processed = parallel_map(raw_data, |x| x * x + 1);
//!
//! // Serialize to JSON asynchronously
//! let json_data = serde::serialize_to_json(&processed).unwrap();
//!
//! // Compress and save
//! let compressed = io::utils::compress_data_brotli(json_data.as_bytes(), 6).await.unwrap();
//! io::utils::write_file_async("processed_data.gz", &compressed).await.unwrap();
//!
//! println!("Processed {} elements", processed.len());
//! # });
//! ```
//!
//! ### Memory Management Example
//! ```rust
//! use trash_utilities::memory::*;
//!
//! // Custom memory pool allocation
//! let pool_name = "processing_pool";
//! create_memory_pool(pool_name, 1024 * 1024).unwrap(); // 1MB pool
//!
//! // Allocate from pool
//! let ptr = alloc_from_pool!(pool_name, 1024).unwrap();
//!
//! // Use allocated memory safely
//! unsafe {
//!     std::ptr::write_bytes(ptr, 0, 1024); // Initialize to zero
//! }
//!
//! // Pool automatically manages cleanup
//! ```
//!
//! ### Channel-Based Communication
//! ```rust,no_run
//! use trash_utilities::channels::*;
//! use smol;
//!
//! # smol::block_on(async {
//! // Create monitored channel
//! let (tx, rx, monitor) = create_monitored_channel::<String>(10);
//!
//! // Send messages
//! for i in 0..5 {
//!     send_async(&tx, format!("Message {}", i)).await.unwrap();
//! }
//!
//! // Receive and process
//! for _ in 0..5 {
//!     let msg = recv_async(&rx).await.unwrap();
//!     println!("Received: {}", msg);
//! }
//!
//! // Check performance stats
//! let stats = monitor.get_stats();
//! println!("Processed {} messages", stats.messages_sent);
//! # });
//! ```
//!
//! ## Module Organization
//!
//! ### Core Functionality
//! - **[`async`]**: Comprehensive async utilities with smol runtime
//! - **[`parallel`]**: Threading and parallelism with work-stealing
//! - **[`memory`]**: Memory management and custom pools
//! - **[`channels`]**: Advanced channel communication
//!
//! ### Data & I/O
//! - **[`serde`]**: Serialization utilities (JSON, base64, etc.)
//! - **[`io`]**: Async file operations and compression
//! - **[`data`]**: Data parsing and manipulation
//! - **[`chars`]**: String processing and encoding
//!
//! ### System Integration
//! - **[`sys`]**: System utilities (time, env vars, filesystem)
//! - **[`common`]**: Shared utilities across modules
//!
//! ### Development Tools
//! - **`logging`**: Structured logging (not shown in docs)
//! - **`macros`**: Ergonomic macros (not shown in docs)
//! - **`utils`**: Additional utilities (not shown in docs)
//!
//! ## Performance Characteristics
//!
//! - **Zero-Copy Operations**: Where possible, avoids unnecessary allocations
//! - **Async-First Design**: Built for non-blocking I/O and concurrency
//! - **Memory Efficient**: Custom allocators and pooling reduce overhead
//! - **Parallel Processing**: Automatic parallelization for CPU-bound tasks
//! - **Monitoring**: Built-in performance tracking and statistics
//!
//! ## Safety & Reliability
//!
//! - **Memory Safe**: All operations are memory-safe with no undefined behavior
//! - **Thread Safe**: Concurrent operations are properly synchronized
//! - **Error Handling**: Comprehensive error propagation with context
//! - **Resource Management**: Automatic cleanup and RAII patterns
//! - **Testing**: Extensive test coverage for reliability
//!
//! ## Platform Support
//!
//! - **Linux**: Full support with optimized system calls
//! - **macOS**: Full support with native optimizations
//! - **Windows**: Full support with Windows-specific implementations
//! - **Cross-Platform**: Consistent API across all platforms
//!
//! ## Contributing
//!
//! Contributions are welcome! Please see the repository for:
//! - Development setup and guidelines
//! - Code style and conventions
//! - Testing requirements
//! - Performance benchmarking
//!
//! ## License
//!
//! This project is licensed under the MIT License - see the LICENSE file for details.

#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/121582001?v=4&size=64")]
#![doc(html_playground_url = "https://play.rust-lang.org/")]

pub mod r#async;
pub mod channels;
pub mod chars;
pub mod common;
pub mod data;
pub mod io;
pub mod memory;
pub mod parallel;
pub mod serde;
pub mod sys;

// Re-exports
pub use parallel::{parallel_for_each, parallel_map};
pub use sys::{current_utc_time, read_env_var};
