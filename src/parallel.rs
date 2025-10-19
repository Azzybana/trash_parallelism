//! # Parallel Processing Utilities
//!
//! This module provides a comprehensive set of parallel processing utilities designed
//! for high-performance computing scenarios. It offers both immediate sequential
//! implementations and extensible interfaces for future parallel execution backends.
//!
//! ## Architecture Overview
//!
//! The module follows a **parallel-ready design pattern** where functions provide:
//! - **Immediate usability**: All functions work correctly with sequential processing
//! - **Future extensibility**: Interfaces designed to easily integrate parallel backends
//! - **Consistent API**: Unified function signatures across all operations
//! - **Performance monitoring**: Built-in timing and statistics collection
//!
//! ## Current Implementation
//!
//! Currently, all functions use **sequential processing** with the following characteristics:
//! - **Thread-safe**: All operations are safe to call from any thread context
//! - **Memory efficient**: Minimal overhead beyond standard library operations
//! - **Composable**: Functions can be chained and combined naturally
//! - **Observable**: Performance monitoring and logging built-in
//!
//! ## Function Categories
//!
//! ### Core Operations
//! - [`parallel_map`] - Transform each element with a function
//! - [`parallel_filter`] - Filter elements based on predicates
//! - [`parallel_for_each`] - Execute side effects on each element
//! - [`parallel_fold`] - Reduce elements to a single value
//!
//! ### Data Organization
//! - [`parallel_partition`] - Split data into matching/non-matching groups
//! - [`parallel_group_by`] - Group elements by computed keys
//! - [`parallel_chunks`] - Divide data into fixed-size chunks
//! - [`parallel_windows`] - Create sliding windows over data
//!
//! ### Data Cleaning
//! - [`parallel_dedup`] - Remove consecutive duplicates
//! - [`parallel_sort`] - Sort data (currently sequential)
//! - [`parallel_search`] - Search for elements (currently sequential)
//!
//! ### Advanced Features
//! - [`parallel_map_async`] - Asynchronous parallel mapping with concurrency control
//! - [`parallel_map_with_cancellation`] - Cancellable parallel operations
//! - [`monitored_execute`] - Performance-monitored function execution
//! - [`create_work_queue`] - Work-stealing task queues
//! - [`distribute_work`] - Load-balanced work distribution
//!
//! ### Monitoring & Observability
//! - [`ThreadPoolMonitor`] - Performance statistics and monitoring
//! - [`ThreadPoolStats`] - Detailed execution metrics
//! - [`OperationTimer`] - Automatic operation timing
//!
//! ## Usage Patterns
//!
//! ### Basic Parallel Processing
//! ```rust
//! use trash_analyzer::parallel::*;
//!
//! // Transform data
//! let data = vec![1, 2, 3, 4, 5];
//! let doubled = parallel_map(data, |x| x * 2);
//!
//! // Filter and process
//! let filtered = parallel_filter(doubled, |&x| x > 5);
//!
//! // Group by criteria
//! let groups = parallel_group_by(filtered, |&x| x % 3);
//! ```
//!
//! ### Performance Monitoring
//! ```rust
//! use trash_analyzer::parallel::{ThreadPoolMonitor, monitored_execute};
//!
//! let monitor = ThreadPoolMonitor::new();
//!
//! let result = monitored_execute(&monitor, "expensive_operation", || {
//!     // Your expensive computation here
//!     42
//! });
//!
//! println!("Stats: {:?}", monitor.stats());
//! ```
//!
//! ### Asynchronous Processing
//! ```rust,no_run
//! use trash_analyzer::parallel::parallel_map_async;
//!
//! async fn process_async() {
//!     let data = vec![1, 2, 3, 4, 5];
//!
//!     // Process up to 3 items concurrently
//!     let results = parallel_map_async(data, |x| async move {
//!         // Simulate async work
//!         x * 2
//!     }, 3).await;
//! }
//! ```
//!
//! ### Work Distribution
//! ```rust
//! use trash_analyzer::parallel::distribute_work;
//!
//! let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//!
//! // Distribute work across chunks
//! let results = distribute_work(&data, |chunk| {
//!     chunk.iter().sum::<i32>()
//! });
//! ```
//!
//! ## Performance Considerations
//!
//! ### When to Use Sequential vs Parallel
//! - **Small datasets**: Sequential processing often faster due to overhead
//! - **Large datasets**: Parallel processing scales better with data size
//! - **I/O bound operations**: Async variants provide better concurrency
//! - **CPU bound operations**: Future parallel backends will excel here
//!
//! ### Memory Usage
//! - **Streaming**: Most operations process data in a single pass
//! - **Allocation**: Results collected into new containers
//! - **Cloning**: Some operations require `Clone` for windowing operations
//!
//! ### Thread Safety
//! - **Send + Sync**: All data types must be thread-safe for future parallelization
//! - **Interior mutability**: Use appropriate synchronization primitives
//! - **Shared state**: Avoid unless explicitly designed for concurrency
//!
//! ## Future Extensions
//!
//! The module is designed for easy integration of parallel execution backends:
//!
//! ### Planned Parallel Backends
//! - **Rayon integration**: Drop-in parallel iterators
//! - **Fork-join pools**: Work-stealing thread pools
//! - **GPU acceleration**: CUDA/OpenCL for compute-intensive operations
//! - **Distributed processing**: Multi-node parallel execution
//!
//! ### Extension Points
//! - **Backend traits**: Pluggable execution strategies
//! - **Configuration**: Runtime backend selection
//! - **Fallbacks**: Automatic fallback to sequential on errors
//! - **Metrics**: Detailed performance profiling
//!
//! ## Error Handling
//!
//! Functions follow Rust's error handling patterns:
//! - **Result types**: Operations that can fail return `Result<T, E>`
//! - **Option types**: Operations that may not produce results return `Option<T>`
//! - **Cancellation**: Graceful handling of cancellation tokens
//! - **Logging**: Comprehensive error logging
//!
//! ## Dependencies
//!
//! The module relies on several key crates:
//! - `fork_union`: Thread pool and parallel execution (imported but not yet used)
//! - `parking_lot`: Efficient synchronization primitives
//! - `smol`: Async runtime for channel-based operations
//! - `smol_cancellation_token`: Cancellation support for async operations
//!
//! ## Testing
//!
//! The module includes comprehensive tests covering:
//! - **Functional correctness**: All operations produce expected results
//! - **Edge cases**: Empty inputs, single elements, large datasets
//! - **Performance**: Benchmarks comparing sequential vs future parallel
//! - **Concurrency**: Thread safety and race condition testing
//! - **Error handling**: Proper error propagation and recovery
//!
//! ## Examples
//!
//! See the individual function documentation for detailed examples.
//! For more complex usage patterns, check the integration tests.

pub mod advanced;
pub mod core;
pub mod data;
pub mod organize;

pub use advanced::*;
pub use core::*;
pub use data::*;
pub use organize::*;
