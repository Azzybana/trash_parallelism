/// Comprehensive async utilities with advanced concurrency and processing capabilities.
///
/// This module provides high-performance async operations using smol runtime
/// and crossfire MPMC channels for optimal non-blocking throughput and zero-copy message passing.
///
/// # Submodules
///
/// - [`core`] - Fundamental async building blocks (sleep, race, join, cancellation, mutex)
/// - [`data`] - Async data processing (compression, serialization, cryptography)
/// - [`patterns`] - Advanced patterns (timeouts, retries, circuit breakers, parallel processing)
/// - [`tasks`] - Task management (spawners, groups, coordination)
///
/// # Examples
///
/// Using multiple utilities together:
/// ```rust
/// use trash_utilities::async::{sleep_for, join, compress_data_async, decompress_data_async};
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// // Compress data asynchronously
/// let data = b"Hello, world! This is a test message.";
/// let compressed = compress_data_async(data, 6).await.unwrap();
///
/// // Decompress while sleeping concurrently
/// let (decompressed, _) = join(
///     decompress_data_async(&compressed),
///     sleep_for(Duration::from_millis(10))
/// ).await;
///
/// assert_eq!(decompressed.unwrap(), data);
/// # });
/// ```
// Submodule declarations
pub mod core;
pub mod data;
pub mod patterns;
pub mod tasks;

// Re-exports for backward compatibility
pub use core::*;
pub use data::*;
pub use patterns::*;
pub use tasks::*;
