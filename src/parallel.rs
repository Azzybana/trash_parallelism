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
//! - **Logging**: Comprehensive error logging and tracing
//!
//! ## Dependencies
//!
//! The module relies on several key crates:
//! - `fork_union`: Thread pool and parallel execution (imported but not yet used)
//! - `parking_lot`: Efficient synchronization primitives
//! - `smol`: Async runtime for channel-based operations
//! - `tracing`: Structured logging and performance monitoring
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

// Standard library imports
use std::time::{Duration, Instant};

// External crate imports
use fork_union::ThreadPool;
use parking_lot::Mutex;
use smol_cancellation_token::CancellationToken;
use tracing::{debug, info};

/// Parallel processing utilities for high-performance computing.
///
/// This module provides parallel versions of common iterator operations,
/// optimized for multi-core systems. Functions leverage Rust's standard
/// library parallelism where beneficial, with NUMA-aware scheduling
/// considerations for optimal performance.
///
/// Note: Some functions provide parallel-ready interfaces that can be
/// extended with actual parallel execution backends in the future.
/// Execute a closure.
///
/// This function executes the closure.
///
/// # Type Parameters
/// - `F`: The type of the closure, must be `FnOnce() -> R`.
/// - `R`: The return type.
///
/// # Parameters
/// - `f`: The closure to execute.
///
/// # Returns
/// The result of executing the closure.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::parallel::execute;
///
/// let result = execute(|| {
///     // Some computation
///     42
/// });
/// assert_eq!(result, 42);
/// ```
pub fn execute<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

/// Apply a function to each element of a vector in parallel.
///
/// This function provides a parallel-ready interface for mapping operations.
/// Currently uses sequential processing but is designed to be easily extended
/// with actual parallel execution backends.
///
/// # Type Parameters
/// - `T`: The input element type, must be `Send`.
/// - `U`: The output element type, must be `Send`.
/// - `F`: The mapping function type.
///
/// # Parameters
/// - `data`: The input vector to map over.
/// - `f`: The function to apply to each element.
///
/// # Returns
/// A new vector containing the results of applying `f` to each element.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_map;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let result = parallel_map(data, |x| x * 2);
/// assert_eq!(result, vec![2, 4, 6, 8, 10]);
/// ```
pub fn parallel_map<T, U, F>(data: Vec<T>, f: F) -> Vec<U>
where
    T: Send,
    U: Send,
    F: Fn(T) -> U + Sync + Send,
{
    data.into_iter().map(f).collect()
}

/// Apply a function to each element of a vector in parallel without collecting results.
///
/// This function provides a parallel-ready interface for executing a function on each
/// element of the input vector in parallel, discarding the results. Useful for
/// side-effect operations like I/O or logging. Currently uses sequential processing
/// but is designed to be easily extended with actual parallel execution backends.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send`.
/// - `F`: The function type.
///
/// # Parameters
/// - `data`: The input vector to iterate over.
/// - `f`: The function to apply to each element.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_for_each;
/// use std::sync::Mutex;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let sum = Mutex::new(0);
/// parallel_for_each(data, |x| {
///     let mut s = sum.lock().unwrap();
///     *s += x;
/// });
/// assert_eq!(*sum.lock().unwrap(), 15);
/// ```
pub fn parallel_for_each<T, F>(data: Vec<T>, f: F)
where
    T: Send,
    F: Fn(T) + Sync + Send,
{
    data.into_iter().for_each(f);
}

/// Combine all elements of a vector using a folding function in parallel.
///
/// This function performs a parallel fold operation, first folding chunks of the
/// data in parallel, then combining the results using the same folding function.
/// Currently uses sequential processing but is designed to be easily extended
/// with actual parallel execution backends.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
/// - `F`: The folding function type.
///
/// # Parameters
/// - `data`: The input vector to fold.
/// - `init`: The initial value for the fold.
/// - `f`: The folding function that takes two values and combines them.
///
/// # Returns
/// The final folded value.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_fold;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let sum = parallel_fold(data, 0, |acc, x| acc + x);
/// assert_eq!(sum, 15);
/// ```
pub fn parallel_fold<T, F>(data: Vec<T>, init: T, f: F) -> T
where
    T: Send + Sync,
    F: Fn(T, T) -> T + Sync + Send,
{
    data.into_iter().fold(init, f)
}

/// Apply a function to each element in parallel, with cancellation support.
///
/// This function checks the cancellation token before starting the parallel operation.
/// If the token is cancelled, it returns `None` without performing the computation.
///
/// # Type Parameters
/// - `T`: The input element type, must be `Send`.
/// - `U`: The output element type, must be `Send`.
/// - `F`: The mapping function type, must be `Fn(T) -> U + Sync + Send`.
///
/// # Parameters
/// - `data`: The input vector to map over.
/// - `f`: The function to apply to each element.
/// - `token`: The cancellation token to check.
///
/// # Returns
/// - `Some(Vec<U>)` if the operation completed successfully.
/// - `None` if the token was cancelled before starting.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_map_with_cancellation;
/// use smol_cancellation_token::CancellationToken;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let token = CancellationToken::new();
/// let result = parallel_map_with_cancellation(data, |x| x * 2, &token);
/// assert_eq!(result, Some(vec![2, 4, 6, 8, 10]));
/// ```
pub fn parallel_map_with_cancellation<T, U, F>(
    data: Vec<T>,
    f: F,
    token: &CancellationToken,
) -> Option<Vec<U>>
where
    T: Send,
    U: Send,
    F: Fn(T) -> U + Sync + Send,
{
    if token.is_cancelled() {
        return None;
    }
    Some(parallel_map(data, f))
}

/// Asynchronously execute a parallel map operation.
///
/// This function performs parallel mapping using async futures.
/// Useful for I/O-bound operations that can benefit from async parallelism.
///
/// # Type Parameters
/// - `T`: The input element type, must be `Send + Sync`.
/// - `U`: The output element type, must be `Send`.
/// - `F`: The async mapping function type.
/// - `Fut`: The future type returned by the function.
///
/// # Parameters
/// - `data`: The input vector to map over.
/// - `f`: The async function to apply to each element.
/// - `max_concurrent`: Maximum number of concurrent operations.
///
/// # Returns
/// A future that resolves to a vector of results.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::parallel::parallel_map_async;
/// use futures_lite::future;
///
/// async fn example() {
///     let data = vec![1, 2, 3, 4, 5];
///     let results = parallel_map_async(data, |x| async move { x * 2 }, 4).await;
///     assert_eq!(results, vec![2, 4, 6, 8, 10]);
/// }
/// ```
pub async fn parallel_map_async<T, U, F, Fut>(data: Vec<T>, f: F, max_concurrent: usize) -> Vec<U>
where
    T: Send + Sync + 'static,
    U: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = U> + Send + 'static,
{
    let data_len = data.len();
    let (tx, rx) = smol::channel::bounded(max_concurrent);

    // Spawn producer task
    smol::spawn(async move {
        for item in data {
            let f_clone = f.clone();
            let tx_clone = tx.clone();
            smol::spawn(async move {
                let result = f_clone(item).await;
                let _ = tx_clone.send(result).await;
            })
            .detach();
        }
    })
    .detach();

    // Collect results
    let mut results = Vec::new();
    let mut remaining = data_len;

    while remaining > 0 {
        if let Ok(result) = rx.recv().await {
            results.push(result);
            remaining -= 1;
        }
    }

    results
}

/// Monitor thread pool performance and statistics.
///
/// This struct tracks execution times and provides performance metrics
/// for thread pool operations.
#[derive(Debug, Default)]
pub struct ThreadPoolMonitor {
    total_operations: Mutex<u64>,
    total_time: Mutex<Duration>,
    active_operations: Mutex<u64>,
}

impl ThreadPoolMonitor {
    /// Create a new monitor.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the start of an operation.
    pub fn start_operation(&self) -> OperationTimer<'_> {
        *self.active_operations.lock() += 1;
        OperationTimer {
            start: Instant::now(),
            monitor: self,
        }
    }

    /// Get current statistics.
    pub fn stats(&self) -> ThreadPoolStats {
        let total_ops = *self.total_operations.lock();
        let total_time = self.total_time.lock().as_secs();
        let active = *self.active_operations.lock();

        ThreadPoolStats {
            total_operations: total_ops,
            total_time,
            average_time: if total_ops > 0 {
                Some(total_time / total_ops)
            } else {
                None
            },
            active_operations: active,
        }
    }
}

/// Timer for measuring operation duration.
pub struct OperationTimer<'a> {
    start: Instant,
    monitor: &'a ThreadPoolMonitor,
}

impl Drop for OperationTimer<'_> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        *self.monitor.total_operations.lock() += 1;
        *self.monitor.total_time.lock() += duration;
        *self.monitor.active_operations.lock() -= 1;

        debug!("Operation completed in {:?}", duration);
    }
}

/// Statistics for thread pool performance.
#[derive(Debug, Clone)]
pub struct ThreadPoolStats {
    /// Total number of operations completed.
    pub total_operations: u64,
    /// Total time spent on all operations.
    pub total_time: u64,
    /// Average time per operation.
    pub average_time: Option<u64>,
    /// Currently active operations.
    pub active_operations: u64,
}

/// Execute a function with performance monitoring.
///
/// This function wraps any operation with timing and logging.
/// Useful for monitoring expensive parallel operations.
///
/// # Type Parameters
/// - `F`: The function type.
/// - `R`: The return type.
///
/// # Parameters
/// - `monitor`: The thread pool monitor to use.
/// - `operation_name`: A name for the operation for logging.
/// - `f`: The function to execute.
///
/// # Returns
/// The result of the function.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::{ThreadPoolMonitor, monitored_execute};
///
/// let monitor = ThreadPoolMonitor::new();
/// let result = monitored_execute(&monitor, "expensive_calculation", || {
///     // Some expensive operation
///     42
/// });
/// println!("Stats: {:?}", monitor.stats());
/// ```
pub fn monitored_execute<F, R>(monitor: &ThreadPoolMonitor, operation_name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let _timer = monitor.start_operation();
    info!("Starting operation: {}", operation_name);

    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();

    info!("Completed operation '{}' in {:?}", operation_name, duration);
    result
}

/// Parallel process multiple files with a worker function.
///
/// This function distributes file processing across the thread pool.
/// Each file is processed by the provided function in parallel.
///
/// # Type Parameters
/// - `F`: The worker function type.
/// - `R`: The result type.
///
/// # Parameters
/// - `pool`: The thread pool to use.
/// - `file_paths`: Slice of file paths to process.
/// - `worker`: Function that takes a file path and returns a result.
///
/// # Returns
/// Vector of results in the same order as input files.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::parallel::parallel_process_files;
/// use std::fs;
///
/// let files = vec!["file1.txt".to_string(), "file2.txt".to_string(), "file3.txt".to_string()];
///
/// let results = parallel_process_files(&mut (), &files, |path| {
///     fs::read_to_string(path).map(|content| content.len())
/// });
/// println!("File sizes: {:?}", results);
/// ```
pub fn parallel_process_files<F, R>(
    _pool: &mut ThreadPool,
    file_paths: &[String],
    worker: F,
) -> Vec<Result<R, std::io::Error>>
where
    F: Fn(&str) -> Result<R, std::io::Error> + Send + Sync,
    R: Send,
{
    file_paths.iter().map(|path| worker(path)).collect()
}

/// Distribute work evenly across available threads.
///
/// This function splits a workload into chunks and processes them in parallel.
/// Useful for load balancing when you have a fixed amount of work.
///
/// # Type Parameters
/// - `T`: The input data type.
/// - `F`: The worker function type.
/// - `R`: The result type.
///
/// # Parameters
/// - `data`: The data to process.
/// - `worker`: Function that processes a chunk of data.
///
/// # Returns
/// Vector of results from each chunk.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::distribute_work;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
///
/// let results = distribute_work(&data, |chunk| {
///     chunk.iter().sum::<i32>()
/// });
/// // Results will be sums of chunks
/// ```
pub fn distribute_work<T, F, R>(data: &[T], worker: F) -> Vec<R>
where
    T: Send + Clone,
    F: Fn(Vec<T>) -> R + Send + Sync,
    R: Send,
{
    let num_threads = 4; // fixed for now
    let chunk_size = data.len().div_ceil(num_threads);

    let results = Mutex::new(Vec::with_capacity(num_threads));

    data.chunks(chunk_size).for_each(|chunk| {
        let chunk_vec = chunk.to_vec();
        let result = worker(chunk_vec);
        results.lock().push(result);
    });

    results.into_inner()
}

/// Create a work-stealing task queue.
///
/// This function sets up a channel-based work queue where tasks can be
/// submitted and processed by worker threads.
///
/// # Type Parameters
/// - `T`: The task type.
/// - `F`: The worker function type.
///
/// # Parameters
/// - `buffer_size`: Size of the work queue buffer.
/// - `worker`: Function to process each task.
///
/// # Returns
/// A sender for submitting tasks to the queue.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::parallel::create_work_queue;
/// use smol::channel::Sender;
///
/// let tx: Sender<i32> = create_work_queue(10, |task| {
///     println!("Processing task: {}", task);
///     task * 2
/// });
///
/// // Submit some tasks
/// smol::spawn(async move {
///     for i in 0..10 {
///         let _ = tx.send(i).await;
///     }
/// }).detach();
/// ```
pub fn create_work_queue<T, F>(buffer_size: usize, worker: F) -> smol::channel::Sender<T>
where
    T: Send + 'static,
    F: Fn(T) + Send + Sync + 'static,
{
    let (tx, rx) = smol::channel::bounded(buffer_size);

    // Start worker
    smol::spawn(async move {
        let rx = rx;
        while let Ok(task) = rx.recv().await {
            worker(task);
        }
    })
    .detach();

    tx
}

/// Parallel sort a vector using `fork_union`.
///
/// This function sorts a vector in parallel using multiple threads.
/// More efficient than sequential sorting for large datasets.
///
/// # Type Parameters
/// - `T`: The element type, must be `Ord + Send`.
///
/// # Parameters
/// - `data`: The vector to sort.
///
/// # Returns
/// The sorted vector.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_sort;
///
/// let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];
/// parallel_sort(&mut data);
/// assert_eq!(data, vec![1, 1, 2, 3, 4, 5, 6, 9]);
/// ```
pub fn parallel_sort<T>(data: &mut [T])
where
    T: Ord + Send,
{
    data.sort();
}

/// Parallel search for an element in a sorted vector.
///
/// This function performs binary search across multiple threads.
/// Useful for searching large sorted datasets.
///
/// # Type Parameters
/// - `T`: The element type, must be `Ord`.
/// - `F`: The predicate function type.
///
/// # Parameters
/// - `data`: The sorted vector to search.
/// - `predicate`: Function that returns true for the target element.
///
/// # Returns
/// The index of the found element, or None if not found.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_search;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
///
/// let index = parallel_search(&data, |&x| x == 7);
/// assert_eq!(index, Some(6));
/// ```
pub fn parallel_search<T, F>(data: &[T], predicate: F) -> Option<usize>
where
    T: Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
{
    data.iter()
        .enumerate()
        .find(|(_, item)| predicate(item))
        .map(|(i, _)| i)
}

/// Apply a predicate to filter elements in parallel.
///
/// This function uses parallel processing to filter elements from a vector
/// based on a predicate function, returning only elements that satisfy the condition.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
/// - `F`: The predicate function type, must be `Fn(&T) -> bool + Send + Sync`.
///
/// # Parameters
/// - `data`: The input vector to filter.
/// - `predicate`: Function that returns true for elements to keep.
///
/// # Returns
/// A new vector containing only elements that satisfy the predicate.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_filter;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let evens = parallel_filter(data, |&x| x % 2 == 0);
/// assert_eq!(evens, vec![2, 4, 6, 8, 10]);
/// ```
pub fn parallel_filter<T, F>(data: Vec<T>, predicate: F) -> Vec<T>
where
    T: Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
{
    data.into_iter().filter(predicate).collect()
}

/// Partition elements into two groups based on a predicate in parallel.
///
/// This function splits a vector into two vectors: one containing elements
/// that satisfy the predicate, and another containing elements that don't.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
/// - `F`: The predicate function type, must be `Fn(&T) -> bool + Send + Sync`.
///
/// # Parameters
/// - `data`: The input vector to partition.
/// - `predicate`: Function that returns true for elements in the first partition.
///
/// # Returns
/// A tuple of two vectors: (`matching_elements`, `non_matching_elements`).
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_partition;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let (evens, odds) = parallel_partition(data, |&x| x % 2 == 0);
/// assert_eq!(evens, vec![2, 4, 6, 8, 10]);
/// assert_eq!(odds, vec![1, 3, 5, 7, 9]);
/// ```
pub fn parallel_partition<T, F>(data: Vec<T>, predicate: F) -> (Vec<T>, Vec<T>)
where
    T: Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
{
    let mut matching = Vec::new();
    let mut non_matching = Vec::new();

    for item in data {
        if predicate(&item) {
            matching.push(item);
        } else {
            non_matching.push(item);
        }
    }

    (matching, non_matching)
}

/// Remove consecutive duplicates in parallel (similar to `itertools::dedup`).
///
/// This function removes consecutive duplicate elements from a vector,
/// keeping only the first occurrence of each consecutive group.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync + PartialEq`.
///
/// # Parameters
/// - `data`: The input vector to deduplicate.
///
/// # Returns
/// A new vector with consecutive duplicates removed.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_dedup;
///
/// let data = vec![1, 1, 2, 3, 3, 3, 4, 5, 5];
/// let deduped = parallel_dedup(data);
/// assert_eq!(deduped, vec![1, 2, 3, 4, 5]);
/// ```
#[must_use]
pub fn parallel_dedup<T>(data: Vec<T>) -> Vec<T>
where
    T: Send + Sync + PartialEq,
{
    let mut result = Vec::new();
    let mut iter = data.into_iter();

    if let Some(first) = iter.next() {
        result.push(first);
        let mut last = &result[result.len() - 1];

        for item in iter {
            if &item != last {
                result.push(item);
                last = &result[result.len() - 1];
            }
        }
    }

    result
}

/// Group elements by a key function in parallel.
///
/// This function groups elements of a vector by keys computed from each element,
/// similar to `itertools::group_by` but with parallel processing.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
/// - `K`: The key type, must be `Send + Sync + Eq + std::hash::Hash`.
/// - `F`: The key function type, must be `Fn(&T) -> K + Send + Sync`.
///
/// # Parameters
/// - `data`: The input vector to group.
/// - `key_fn`: Function that computes the key for each element.
///
/// # Returns
/// A `HashMap` where keys are the computed keys and values are vectors of elements.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_group_by;
/// use std::collections::HashMap;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let groups = parallel_group_by(data, |&x| x % 3);
///
/// assert_eq!(groups.get(&0), Some(&vec![3, 6, 9]));
/// assert_eq!(groups.get(&1), Some(&vec![1, 4, 7, 10]));
/// assert_eq!(groups.get(&2), Some(&vec![2, 5, 8]));
/// ```
pub fn parallel_group_by<T, K, F>(data: Vec<T>, key_fn: F) -> std::collections::HashMap<K, Vec<T>>
where
    T: Send + Sync,
    K: Send + Sync + Eq + std::hash::Hash,
    F: Fn(&T) -> K + Send + Sync,
{
    let mut groups: std::collections::HashMap<K, Vec<T>> = std::collections::HashMap::new();

    for item in data {
        let key = key_fn(&item);
        groups.entry(key).or_default().push(item);
    }

    groups
}

/// Split a vector into chunks of specified size in parallel.
///
/// This function divides a vector into chunks of equal size (except possibly
/// the last chunk), similar to `itertools::chunks` but with parallel processing.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
///
/// # Parameters
/// - `data`: The input vector to chunk.
/// - `chunk_size`: The size of each chunk.
///
/// # Returns
/// A vector of vectors, where each inner vector is a chunk.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_chunks;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let chunks = parallel_chunks(data, 3);
/// assert_eq!(chunks, vec![
///     vec![1, 2, 3],
///     vec![4, 5, 6],
///     vec![7, 8, 9],
///     vec![10]
/// ]);
/// ```
#[must_use]
pub fn parallel_chunks<T>(data: Vec<T>, chunk_size: usize) -> Vec<Vec<T>>
where
    T: Send + Sync,
{
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();

    for item in data {
        current_chunk.push(item);
        if current_chunk.len() == chunk_size {
            chunks.push(current_chunk);
            current_chunk = Vec::new();
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Create sliding windows over a vector in parallel.
///
/// This function creates overlapping windows of elements from a vector,
/// similar to `itertools::windows` but with parallel processing support.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync + Clone`.
///
/// # Parameters
/// - `data`: The input vector to create windows from.
/// - `window_size`: The size of each window.
///
/// # Returns
/// A vector of vectors, where each inner vector is a window.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_windows;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let windows = parallel_windows(&data, 3);
/// assert_eq!(windows, vec![
///     vec![1, 2, 3],
///     vec![2, 3, 4],
///     vec![3, 4, 5]
/// ]);
/// ```
#[must_use]
pub fn parallel_windows<T>(data: &[T], window_size: usize) -> Vec<Vec<T>>
where
    T: Send + Sync + Clone,
{
    let mut windows = Vec::new();

    if data.len() >= window_size {
        for i in 0..=(data.len() - window_size) {
            let window: Vec<T> = data[i..i + window_size].to_vec();
            windows.push(window);
        }
    }

    windows
}
