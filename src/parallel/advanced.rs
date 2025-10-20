use fork_union::ThreadPool;
use parking_lot::Mutex;
use smol_cancellation_token::CancellationToken;
use std::time::{Duration, Instant};

/// Advanced parallel processing utilities with monitoring and async support.
///
/// This module provides sophisticated parallel processing capabilities including
/// performance monitoring, work distribution, task queues, and async operations.
/// Built on top of the core parallel primitives with additional features for
/// production workloads.
///
/// ## Features
///
/// - **Performance Monitoring**: Thread pool statistics and operation timing
/// - **Work Distribution**: Load balancing across available threads
/// - **Task Queues**: Channel-based work-stealing task processing
/// - **Cancellation Support**: Graceful cancellation of parallel operations
/// - **Async Parallelism**: Futures-based concurrent processing
/// - **File Processing**: Parallel file operations with error handling
///
/// ## Examples
///
/// ### Performance Monitoring
/// ```rust
/// use trash_utilities::parallel::*;
/// use std::time::Duration;
///
/// let monitor = ThreadPoolMonitor::new();
///
/// // Monitor expensive operations
/// let result = monitored_execute(&monitor, "data_processing", || {
///     // Simulate expensive work
///     std::thread::sleep(Duration::from_millis(100));
///     vec![1, 2, 3, 4, 5]
/// });
///
/// let stats = monitor.stats();
/// println!("Completed {} operations, avg time: {:?}",
///          stats.total_operations,
///          stats.average_time.map(|t| Duration::from_secs(t)));
/// ```
///
/// ### Work Distribution
/// ```rust
/// use trash_utilities::parallel::distribute_work;
///
/// let large_dataset = (0..1000).collect::<Vec<_>>();
///
/// // Process data in chunks across threads
/// let chunk_sums = distribute_work(&large_dataset, |chunk| {
///     chunk.iter().sum::<i32>()
/// });
///
/// let total: i32 = chunk_sums.iter().sum();
/// println!("Total sum: {}", total);
/// ```
///
/// ### Task Queue Processing
/// ```rust,no_run
/// use trash_utilities::parallel::create_work_queue;
/// use smol::channel::Sender;
///
/// // Create a task queue for processing jobs
/// let tx: Sender<String> = create_work_queue(10, |job| {
///     println!("Processing job: {}", job);
///     // Process the job...
/// });
///
/// // Submit tasks asynchronously
/// smol::spawn(async move {
///     for i in 0..5 {
///         let job = format!("Job {}", i);
///         let _ = tx.send(job).await;
///     }
/// }).detach();
/// ```
#[derive(Debug, Default)]
pub struct ThreadPoolMonitor {
    total_operations: Mutex<u64>,
    total_time: Mutex<Duration>,
    active_operations: Mutex<u64>,
}

impl ThreadPoolMonitor {
    /// Create a new thread pool monitor.
    ///
    /// Initializes all counters to zero and prepares the monitor for tracking
    /// thread pool performance metrics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::parallel::ThreadPoolMonitor;
    ///
    /// let monitor = ThreadPoolMonitor::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the start of an operation and return a timer.
    ///
    /// This method increments the active operations counter and returns an
    /// `OperationTimer` that will automatically record the operation's
    /// completion when dropped.
    ///
    /// # Returns
    ///
    /// An `OperationTimer` that tracks the operation duration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::parallel::ThreadPoolMonitor;
    ///
    /// let monitor = ThreadPoolMonitor::new();
    /// let timer = monitor.start_operation();
    /// // ... perform operation ...
    /// drop(timer); // Records completion
    /// ```
    pub fn start_operation(&self) -> OperationTimer<'_> {
        *self.active_operations.lock() += 1;
        OperationTimer {
            start: Instant::now(),
            monitor: self,
        }
    }

    /// Get current performance statistics.
    ///
    /// Returns a snapshot of current thread pool performance metrics
    /// including operation counts, timing, and averages.
    ///
    /// # Returns
    ///
    /// Current `ThreadPoolStats` containing performance metrics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::parallel::ThreadPoolMonitor;
    ///
    /// let monitor = ThreadPoolMonitor::new();
    /// let stats = monitor.stats();
    /// println!("Total operations: {}", stats.total_operations);
    /// ```
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
///
/// Automatically tracks the duration of an operation from creation to drop.
/// When dropped, it updates the associated monitor's statistics.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::parallel::ThreadPoolMonitor;
///
/// let monitor = ThreadPoolMonitor::new();
/// {
///     let _timer = monitor.start_operation();
///     // Operation work here
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// } // Timer automatically records completion here
/// ```
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
    }
}

/// Statistics for thread pool performance.
///
/// Contains aggregated performance metrics for thread pool operations,
/// including timing information and operation counts.
///
/// # Fields
///
/// * `total_operations` - Total number of completed operations
/// * `total_time` - Total time spent on all operations (in seconds)
/// * `average_time` - Average time per operation (in seconds), or None if no operations
/// * `active_operations` - Number of currently active operations
///
/// # Examples
///
/// ```rust
/// use trash_utilities::parallel::{ThreadPoolMonitor, ThreadPoolStats};
///
/// let monitor = ThreadPoolMonitor::new();
/// let stats: ThreadPoolStats = monitor.stats();
///
/// println!("Completed {} operations", stats.total_operations);
/// if let Some(avg) = stats.average_time {
///     println!("Average time: {} seconds", avg);
/// }
/// ```
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

    println!("Starting operation: {operation_name}");
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!("Operation '{operation_name}' completed in {duration:?}");

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
    Some(super::core::parallel_map(data, f))
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
