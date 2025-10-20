/// Advanced async patterns for reliability, concurrency, and resource management.
///
/// This module provides robust async utilities including timeout wrappers, retry mechanisms,
/// circuit breakers for fault tolerance, parallel processing, resource pools, streaming processors,
/// and performance monitoring to build resilient async applications.
///
/// # Examples
///
/// Basic timeout usage:
/// ```rust
/// use trash_utilities::async::patterns::{with_timeout, TimeoutError};
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// let result = with_timeout(Duration::from_secs(1), async {
///     smol::Timer::after(Duration::from_millis(500)).await;
///     42
/// }).await;
/// assert_eq!(result, Ok(42));
/// # });
/// ```
// Standard library imports
use std::{marker::PhantomData, sync::Arc, time::Duration};

// External crate imports
use futures_lite;
use parking_lot::Mutex;
use smol::Timer;

/// Async timeout wrapper (non-blocking).
///
/// Wraps an async operation with a timeout, ensuring it completes within the specified duration.
/// If the operation takes longer than the timeout, it returns a `TimeoutError`.
///
/// # Parameters
///
/// * `duration` - The maximum time to wait for the operation to complete.
/// * `future` - The async operation to execute.
///
/// # Type Parameters
///
/// * `T` - The type of the result returned by the future.
/// * `F` - The type of the future.
///
/// # Returns
///
/// Returns `Ok(result)` if the operation completes within the timeout, or `Err(TimeoutError::Timeout)` if it times out.
///
/// # Errors
///
/// Returns `TimeoutError::Timeout` if the operation exceeds the specified duration.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::patterns::{with_timeout, TimeoutError};
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// // This will succeed
/// let result = with_timeout(Duration::from_secs(1), async {
///     smol::Timer::after(Duration::from_millis(100)).await;
///     "success"
/// }).await;
/// assert_eq!(result, Ok("success"));
///
/// // This will timeout
/// let result = with_timeout(Duration::from_millis(50), async {
///     smol::Timer::after(Duration::from_millis(100)).await;
///     "too slow"
/// }).await;
/// assert_eq!(result, Err(TimeoutError::Timeout));
/// # });
/// ```
pub async fn with_timeout<T, F>(duration: Duration, future: F) -> Result<T, TimeoutError>
where
    F: std::future::Future<Output = T>,
{
    match futures_lite::future::or(
        async {
            Timer::after(duration).await;
            Err::<T, TimeoutError>(TimeoutError::Timeout)
        },
        async { Ok(future.await) },
    )
    .await
    {
        Ok(result) => Ok(result),
        Err(e) => Err(e),
    }
}

/// Timeout error.
///
/// Represents the possible errors that can occur when using timeout wrappers.
#[derive(Debug, Clone)]
pub enum TimeoutError {
    /// The operation timed out before completing.
    Timeout,
}

/// Async retry mechanism (non-blocking).
///
/// Retries an async operation up to 3 times with a 100ms delay between attempts.
/// This is a convenience function that uses default retry configuration.
///
/// # Parameters
///
/// * `operation` - A closure that returns the async operation to retry.
///
/// # Type Parameters
///
/// * `F` - The type of the operation closure.
/// * `Fut` - The type of the future returned by the operation.
/// * `T` - The type of the successful result.
/// * `E` - The type of the error.
///
/// # Returns
///
/// Returns `Ok(result)` if the operation eventually succeeds, or `Err(error)` if all attempts fail.
///
/// # Errors
///
/// Returns the last error encountered if all retry attempts fail.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::patterns::retry_async;
/// use smol;
///
/// # smol::block_on(async {
/// let result = retry_async(|| async {
///     // Simulate an operation that might fail
///     Ok::<_, std::io::Error>("success")
/// }).await;
/// assert_eq!(result, Ok("success"));
/// # });
/// ```
pub async fn retry_async<F, Fut, T, E>(operation: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    retry_async_with_config(3, Duration::from_millis(100), operation).await
}

/// Retries an async operation with custom configuration.
///
/// Allows specifying the number of retry attempts and the delay between them.
///
/// # Parameters
///
/// * `attempts` - The maximum number of attempts (including the initial one).
/// * `delay` - The duration to wait between retry attempts.
/// * `operation` - A closure that returns the async operation to retry.
///
/// # Type Parameters
///
/// * `F` - The type of the operation closure.
/// * `Fut` - The type of the future returned by the operation.
/// * `T` - The type of the successful result.
/// * `E` - The type of the error.
///
/// # Returns
///
/// Returns `Ok(result)` if the operation eventually succeeds, or `Err(error)` if all attempts fail.
///
/// # Errors
///
/// Returns the last error encountered if all retry attempts fail.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::patterns::retry_async_with_config;
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// let result = retry_async_with_config(5, Duration::from_millis(50), || async {
///     // Custom retry logic
///     Ok::<_, std::io::Error>("success")
/// }).await;
/// assert_eq!(result, Ok("success"));
/// # });
/// ```
pub async fn retry_async_with_config<F, Fut, T, E>(
    mut attempts: usize,
    delay: Duration,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts -= 1;
                if attempts == 0 {
                    return Err(e);
                }
                Timer::after(delay).await;
            }
        }
    }
}

/// Async circuit breaker (non-blocking).
///
/// A circuit breaker prevents cascading failures by temporarily stopping calls to a failing service.
/// When failures exceed a threshold, the circuit "opens" and rejects further calls for a timeout period.
/// After the timeout, it allows limited calls to test if the service has recovered.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::patterns::{AsyncCircuitBreaker, CircuitBreakerError};
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// let breaker = AsyncCircuitBreaker::new();
/// let result = breaker.execute(|| async { Ok::<_, std::io::Error>("success") }).await;
/// assert_eq!(result, Ok("success"));
/// # });
/// ```
pub struct AsyncCircuitBreaker {
    failures: Mutex<u32>,
    threshold: u32,
    timeout: Duration,
    last_failure: Mutex<Option<std::time::Instant>>,
}

impl AsyncCircuitBreaker {
    /// Create a new circuit breaker with defaults.
    ///
    /// Uses a failure threshold of 5 and a timeout of 60 seconds.
    ///
    /// # Returns
    ///
    /// A new `AsyncCircuitBreaker` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::patterns::AsyncCircuitBreaker;
    ///
    /// let breaker = AsyncCircuitBreaker::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(5, Duration::from_secs(60))
    }

    /// Create a new circuit breaker with custom config.
    ///
    /// # Parameters
    ///
    /// * `threshold` - Number of consecutive failures before opening the circuit.
    /// * `timeout` - Duration to wait before attempting to close the circuit again.
    ///
    /// # Returns
    ///
    /// A new `AsyncCircuitBreaker` instance with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::patterns::AsyncCircuitBreaker;
    /// use std::time::Duration;
    ///
    /// let breaker = AsyncCircuitBreaker::with_config(3, Duration::from_secs(30));
    /// ```
    #[must_use]
    pub fn with_config(threshold: u32, timeout: Duration) -> Self {
        Self {
            failures: Mutex::new(0),
            threshold,
            timeout,
            last_failure: Mutex::new(None),
        }
    }

    /// Create a builder for advanced configuration.
    ///
    /// Use the builder for fluent configuration of the circuit breaker.
    ///
    /// # Returns
    ///
    /// An `AsyncCircuitBreakerBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::patterns::AsyncCircuitBreaker;
    /// use std::time::Duration;
    ///
    /// let breaker = AsyncCircuitBreaker::builder()
    ///     .threshold(10)
    ///     .timeout(Duration::from_secs(120))
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> AsyncCircuitBreakerBuilder {
        AsyncCircuitBreakerBuilder::new()
    }

    /// Execute an operation with circuit breaker protection (non-blocking).
    ///
    /// If the circuit is open, the operation is not executed and `CircuitOpen` is returned.
    /// If the operation succeeds, the failure count is reset. If it fails, the failure count increases,
    /// and if it exceeds the threshold, the circuit opens.
    ///
    /// # Parameters
    ///
    /// * `operation` - A closure that returns the async operation to protect.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The type of the operation closure.
    /// * `Fut` - The type of the future.
    /// * `T` - The type of the successful result.
    /// * `E` - The type of the error.
    ///
    /// # Returns
    ///
    /// Returns `Ok(result)` if the operation succeeds, or an error if the circuit is open or the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `CircuitBreakerError::CircuitOpen` if the circuit is open, or `CircuitBreakerError::OperationError` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::patterns::{AsyncCircuitBreaker, CircuitBreakerError};
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let breaker = AsyncCircuitBreaker::new();
    /// let result = breaker.execute(|| async { Ok::<_, std::io::Error>("success") }).await;
    /// assert_eq!(result, Ok("success"));
    /// # });
    /// ```
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        // Check if circuit is open
        if let Some(last_failure) = *self.last_failure.lock()
            && last_failure.elapsed() < self.timeout
        {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        match operation().await {
            Ok(result) => {
                // Reset on success
                *self.failures.lock() = 0;
                Ok(result)
            }
            Err(e) => {
                let failures = {
                    let mut f = self.failures.lock();
                    *f += 1;
                    *f
                };

                if failures >= self.threshold {
                    *self.last_failure.lock() = Some(std::time::Instant::now());
                }

                Err(CircuitBreakerError::OperationError(e))
            }
        }
    }
}

impl Default for AsyncCircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `AsyncCircuitBreaker` with ergonomic configuration
pub struct AsyncCircuitBreakerBuilder {
    threshold: u32,
    timeout: Duration,
}

impl AsyncCircuitBreakerBuilder {
    /// Create a new builder with defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            threshold: 5,
            timeout: Duration::from_secs(60),
        }
    }

    /// Set the failure threshold
    #[must_use]
    pub fn threshold(mut self, threshold: u32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Set the timeout duration
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the `AsyncCircuitBreaker`
    #[must_use]
    pub fn build(self) -> AsyncCircuitBreaker {
        AsyncCircuitBreaker::with_config(self.threshold, self.timeout)
    }
}

impl Default for AsyncCircuitBreakerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Circuit breaker error.
///
/// Represents the possible errors that can occur when using the circuit breaker.
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// The circuit is open and operations are not allowed.
    CircuitOpen,
    /// The operation failed with the wrapped error.
    OperationError(E),
}

/// Async parallel processing (non-blocking, high-throughput).
///
/// Processes a collection of items in parallel using async tasks.
/// Each item is processed by the provided function, and results are collected in order.
///
/// # Parameters
///
/// * `data` - The vector of items to process.
/// * `processor` - A function that takes an item and returns a processed result.
///
/// # Type Parameters
///
/// * `T` - The type of input items.
/// * `U` - The type of output results.
/// * `F` - The type of the processor function.
///
/// # Returns
///
/// A vector of processed results in the same order as the input.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::patterns::parallel_process_async;
/// use smol;
///
/// # smol::block_on(async {
/// let data = vec![1, 2, 3, 4, 5];
/// let results = parallel_process_async(data, |x| x * 2).await;
/// assert_eq!(results, vec![2, 4, 6, 8, 10]);
/// # });
/// ```
pub async fn parallel_process_async<T, U, F>(data: Vec<T>, processor: F) -> Vec<U>
where
    T: Send + 'static,
    U: Send + 'static,
    F: Fn(T) -> U + Send + Sync + 'static,
{
    let processor = Arc::new(processor);
    let mut tasks = Vec::new();

    for item in data {
        let processor = processor.clone();
        let task = smol::spawn(async move { smol::unblock(move || processor(item)).await });
        tasks.push(task);
    }

    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await);
    }

    results
}

/// Async resource pool (non-blocking).
///
/// Manages a pool of reusable resources, creating them on demand and recycling them.
/// Resources are acquired via guards that automatically return them to the pool when dropped.
///
/// # Type Parameters
///
/// * `T` - The type of resources in the pool.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::patterns::AsyncResourcePool;
///
/// let pool = AsyncResourcePool::new(|| String::from("resource"));
/// let guard = pool.acquire();
/// assert_eq!(*guard, "resource");
/// // Resource is automatically returned when guard is dropped
/// ```
pub struct AsyncResourcePool<T> {
    resources: Mutex<Vec<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> AsyncResourcePool<T> {
    /// Create a new resource pool with defaults
    pub fn new<F>(factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self::with_config(factory, 10)
    }

    /// Create a new resource pool with custom config
    pub fn with_config<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            resources: Mutex::new(Vec::new()),
            factory: Box::new(factory),
            max_size,
        }
    }

    /// Create a builder for advanced configuration
    pub fn builder<F>(factory: F) -> AsyncResourcePoolBuilder<T, F>
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        AsyncResourcePoolBuilder::new(factory)
    }

    /// Acquire a resource from the pool (non-blocking)
    pub fn acquire(&self) -> ResourceGuard<'_, T> {
        let resource = {
            let mut resources = self.resources.lock();
            resources.pop().unwrap_or_else(|| (self.factory)())
        };

        ResourceGuard {
            resource: Some(resource),
            pool: self,
        }
    }

    /// Return a resource to the pool
    fn release(&self, resource: T) {
        let mut resources = self.resources.lock();
        if resources.len() < self.max_size {
            resources.push(resource);
        }
    }
}

/// Builder for `AsyncResourcePool` with ergonomic configuration
pub struct AsyncResourcePoolBuilder<T, F> {
    factory: F,
    max_size: usize,
    _phantom: PhantomData<T>,
}

impl<T, F> AsyncResourcePoolBuilder<T, F>
where
    F: Fn() -> T + Send + Sync + 'static,
{
    /// Create a new builder
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            max_size: 10,
            _phantom: PhantomData,
        }
    }

    /// Set the maximum pool size
    #[must_use]
    pub fn max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// Build the `AsyncResourcePool`
    pub fn build(self) -> AsyncResourcePool<T> {
        AsyncResourcePool::with_config(self.factory, self.max_size)
    }
}

/// Resource guard for pool resources
pub struct ResourceGuard<'a, T> {
    resource: Option<T>,
    pool: &'a AsyncResourcePool<T>,
}

impl<T> std::ops::Deref for ResourceGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource
            .as_ref()
            .expect("resource must exist in guard")
    }
}

impl<T> std::ops::DerefMut for ResourceGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource
            .as_mut()
            .expect("resource must exist in guard")
    }
}

impl<T> Drop for ResourceGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            self.pool.release(resource);
        }
    }
}

/// Async streaming processor (non-blocking, high-throughput).
///
/// Buffers items and processes them in batches when the buffer is full or flushed.
/// Useful for efficient batch processing of streaming data.
///
/// # Type Parameters
///
/// * `T` - The type of items to process.
/// * `F` - The type of the batch processor function.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::patterns::AsyncStreamProcessor;
/// use smol;
///
/// # smol::block_on(async {
/// let processor = AsyncStreamProcessor::new(|batch: Vec<i32>| async move {
///     println!("Processing batch of {} items", batch.len());
/// });
/// processor.push(1).await;
/// processor.push(2).await;
/// processor.flush().await; // Process remaining items
/// # });
/// ```
pub struct AsyncStreamProcessor<T, F> {
    processor: F,
    buffer: Mutex<Vec<T>>,
    buffer_size: usize,
}

impl<T, F, Fut> AsyncStreamProcessor<T, F>
where
    F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
    T: Send + 'static,
{
    /// Create a new stream processor with defaults
    pub fn new(processor: F) -> Self {
        Self::with_config(processor, 100)
    }

    /// Create a new stream processor with custom config
    pub fn with_config(processor: F, buffer_size: usize) -> Self {
        Self {
            processor,
            buffer: Mutex::new(Vec::new()),
            buffer_size,
        }
    }

    /// Create a builder for advanced configuration
    pub fn builder(processor: F) -> AsyncStreamProcessorBuilder<T, F> {
        AsyncStreamProcessorBuilder::new(processor)
    }

    /// Add an item to the stream (non-blocking)
    pub async fn push(&self, item: T) {
        let should_process = {
            let mut buffer = self.buffer.lock();
            buffer.push(item);
            buffer.len() >= self.buffer_size
        };

        if should_process {
            self.process_batch().await;
        }
    }

    /// Flush remaining items (non-blocking)
    pub async fn flush(&self) {
        self.process_batch().await;
    }

    async fn process_batch(&self) {
        let batch = {
            let mut buffer = self.buffer.lock();
            std::mem::take(&mut *buffer)
        };

        if !batch.is_empty() {
            (self.processor)(batch).await;
        }
    }
}

/// Builder for `AsyncStreamProcessor` with ergonomic configuration
pub struct AsyncStreamProcessorBuilder<T, F> {
    processor: F,
    buffer_size: usize,
    _phantom: PhantomData<T>,
}

impl<T, F> AsyncStreamProcessorBuilder<T, F> {
    /// Create a new builder
    pub fn new(processor: F) -> Self {
        Self {
            processor,
            buffer_size: 100,
            _phantom: PhantomData,
        }
    }

    /// Set the buffer size
    #[must_use]
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Build the `AsyncStreamProcessor`
    pub fn build<Fut>(self) -> AsyncStreamProcessor<T, F>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
        T: Send + 'static,
    {
        AsyncStreamProcessor::with_config(self.processor, self.buffer_size)
    }
}

/// Async performance monitor (non-blocking).
///
/// Tracks the execution time of async operations for performance analysis.
/// Provides statistics and averages for monitoring and optimization.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::patterns::AsyncPerformanceMonitor;
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// let monitor = AsyncPerformanceMonitor::new();
/// let result = monitor.time_operation("test_op", || async {
///     smol::Timer::after(Duration::from_millis(10)).await;
///     42
/// }).await;
/// assert_eq!(result, 42);
/// let stats = monitor.stats();
/// assert_eq!(stats.len(), 1);
/// # });
/// ```
#[derive(Debug)]
pub struct AsyncPerformanceMonitor {
    operations: Mutex<Vec<(String, Duration)>>,
}

impl AsyncPerformanceMonitor {
    /// Create a new monitor
    #[must_use]
    pub fn new() -> Self {
        Self {
            operations: Mutex::new(Vec::new()),
        }
    }

    /// Time an async operation (non-blocking)
    pub async fn time_operation<F, Fut, T>(&self, name: &str, operation: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = operation().await;
        let duration = start.elapsed();

        self.operations.lock().push((name.to_string(), duration));

        result
    }

    /// Get performance statistics
    pub fn stats(&self) -> Vec<(String, Duration)> {
        self.operations.lock().clone()
    }

    /// Clear statistics
    pub fn clear(&self) -> &Self {
        self.operations.lock().clear();
        self
    }

    /// Get average duration for an operation
    pub fn avg_duration(&self, name: &str) -> Option<Duration> {
        let operations = self.operations.lock();
        let matching: Vec<_> = operations
            .iter()
            .filter(|(op_name, _)| op_name == name)
            .map(|(_, duration)| *duration)
            .collect();

        if matching.is_empty() {
            None
        } else {
            let total: Duration = matching.iter().sum();
            let len = u32::try_from(matching.len()).unwrap_or(1);
            Some(total / len)
        }
    }

    /// Get total operations count
    pub fn operation_count(&self) -> usize {
        self.operations.lock().len()
    }

    /// Get operations by name
    pub fn operations_for(&self, name: &str) -> Vec<Duration> {
        self.operations
            .lock()
            .iter()
            .filter(|(op_name, _)| op_name == name)
            .map(|(_, duration)| *duration)
            .collect()
    }
}

impl Default for AsyncPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Traced async operation (non-blocking)
pub async fn traced_async_operation<F, Fut, T>(_name: &str, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    f().await
}
