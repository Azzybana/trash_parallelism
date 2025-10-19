/// Comprehensive async utilities with advanced concurrency and processing capabilities.
///
/// This module provides high-performance async operations using smol runtime
/// and crossfire MPMC channels for optimal non-blocking throughput and zero-copy message passing.
// Standard library imports
use std::{
    io::{Read, Write},
    marker::PhantomData,
    sync::Arc,
    time::Duration,
};

// External crate imports
use base64::{Engine, engine::general_purpose::STANDARD};
use brotli::{CompressorWriter, Decompressor};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json;
use smol::Timer;
use smol_cancellation_token::CancellationToken;

/// Async sleep helper
pub async fn sleep_for(duration: Duration) {
    Timer::after(duration).await;
}

/// Futures-lite helper: race two futures
pub async fn race<T, F1, F2>(f1: F1, f2: F2) -> T
where
    F1: std::future::Future<Output = T>,
    F2: std::future::Future<Output = T>,
{
    futures_lite::future::race(f1, f2).await
}

/// Futures-lite helper: join two futures
pub async fn join<T1, T2, F1, F2>(f1: F1, f2: F2) -> (T1, T2)
where
    F1: std::future::Future<Output = T1>,
    F2: std::future::Future<Output = T2>,
{
    futures_lite::future::zip(f1, f2).await
}

/// Cancellation token helper
#[must_use]
pub fn create_cancellation_token() -> CancellationToken {
    CancellationToken::new()
}

/// Run a future with cancellation
pub async fn with_cancellation<T, F>(token: &CancellationToken, future: F) -> Option<T>
where
    F: std::future::Future<Output = T>,
{
    futures_lite::future::or(
        async {
            token.cancelled().await;
            None
        },
        async { Some(future.await) },
    )
    .await
}

/// Parking lot mutex helper
pub fn create_mutex<T>(value: T) -> Mutex<T> {
    Mutex::new(value)
}

/// Async task spawner with error handling
pub struct AsyncTaskSpawner {
    token: CancellationToken,
    handles: Mutex<Vec<smol::Task<()>>>,
}

impl AsyncTaskSpawner {
    /// Create a new task spawner
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Create a builder for more complex configuration
    #[must_use]
    pub fn builder() -> AsyncTaskSpawnerBuilder {
        AsyncTaskSpawnerBuilder::new()
    }

    /// Spawn an async task (non-blocking)
    pub fn spawn<F, Fut>(&self, task: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let token = self.token.clone();
        let handle = smol::spawn(async move {
            if !token.is_cancelled() {
                task().await;
            }
        });

        self.handles.lock().push(handle);
    }

    /// Cancel all tasks
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Wait for all tasks to complete (non-blocking)
    pub async fn wait_all(&self) {
        let handles = std::mem::take(&mut *self.handles.lock());
        for handle in handles {
            let () = handle.await;
        }
    }

    /// Chain method to spawn a task (consumes self)
    #[must_use]
    pub fn with_task<F, Fut>(self, task: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.spawn(task);
        self
    }

    /// Chain method to cancel tasks (consumes self)
    #[must_use]
    pub fn with_cancel(self) -> Self {
        self.cancel();
        self
    }
}

impl Default for AsyncTaskSpawner {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `AsyncTaskSpawner` with ergonomic configuration
pub struct AsyncTaskSpawnerBuilder {
    token: Option<CancellationToken>,
}

impl AsyncTaskSpawnerBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self { token: None }
    }

    /// Set a custom cancellation token
    #[must_use]
    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.token = Some(token);
        self
    }

    /// Build the `AsyncTaskSpawner`
    #[must_use]
    pub fn build(self) -> AsyncTaskSpawner {
        AsyncTaskSpawner {
            token: self.token.unwrap_or_default(),
            handles: Mutex::new(Vec::new()),
        }
    }
}

impl Default for AsyncTaskSpawnerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Async compression utilities (non-blocking)
pub async fn compress_data_async(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    let data = data.to_vec();
    smol::unblock(move || {
        let mut output = Vec::new();
        {
            let mut compressor = CompressorWriter::new(&mut output, 4096, level, level);
            compressor.write_all(&data)?;
            compressor.flush()?;
        }
        Ok(output)
    })
    .await
}

pub async fn decompress_data_async(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let data = data.to_vec();
    smol::unblock(move || {
        let mut decompressor = Decompressor::new(&data[..], 4096);
        let mut output = Vec::new();
        decompressor.read_to_end(&mut output)?;
        Ok(output)
    })
    .await
}

/// Async serialization (non-blocking)
pub async fn serialize_async<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let json_result = serde_json::to_string(value);
    match json_result {
        Ok(json) => Ok(json),
        Err(e) => Err(e),
    }
}

pub async fn deserialize_async<'a, T: Deserialize<'a>>(
    json: &'a str,
) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

/// Async cryptographic operations (non-blocking)
pub async fn hash_data_async(data: &[u8]) -> u64 {
    let data = data.to_vec();
    smol::unblock(move || {
        use ahash::AHasher;
        use std::hash::Hasher;
        let mut hasher = AHasher::default();
        hasher.write(&data);
        hasher.finish()
    })
    .await
}

pub async fn encode_base64_async(data: &[u8]) -> String {
    let data = data.to_vec();
    smol::unblock(move || STANDARD.encode(&data)).await
}

pub async fn decode_base64_async(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let data = data.to_string();
    smol::unblock(move || STANDARD.decode(&data)).await
}

/// Async parallel processing (non-blocking, high-throughput)
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

/// Async timeout wrapper (non-blocking)
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

/// Timeout error
#[derive(Debug, Clone)]
pub enum TimeoutError {
    Timeout,
}

/// Async retry mechanism (non-blocking)
pub async fn retry_async<F, Fut, T, E>(operation: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    retry_async_with_config(3, Duration::from_millis(100), operation).await
}

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

/// Async circuit breaker (non-blocking)
pub struct AsyncCircuitBreaker {
    failures: Mutex<u32>,
    threshold: u32,
    timeout: Duration,
    last_failure: Mutex<Option<std::time::Instant>>,
}

impl AsyncCircuitBreaker {
    /// Create a new circuit breaker with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(5, Duration::from_secs(60))
    }

    /// Create a new circuit breaker with custom config
    #[must_use]
    pub fn with_config(threshold: u32, timeout: Duration) -> Self {
        Self {
            failures: Mutex::new(0),
            threshold,
            timeout,
            last_failure: Mutex::new(None),
        }
    }

    /// Create a builder for advanced configuration
    #[must_use]
    pub fn builder() -> AsyncCircuitBreakerBuilder {
        AsyncCircuitBreakerBuilder::new()
    }

    /// Execute an operation with circuit breaker protection (non-blocking)
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

/// Circuit breaker error
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    OperationError(E),
}

/// Async resource pool (non-blocking)
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
    pub async fn acquire(&self) -> ResourceGuard<'_, T> {
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

/// Async streaming processor (non-blocking, high-throughput)
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

/// Async performance monitor (non-blocking)
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
            Some(total / matching.len() as u32)
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

/// Async task group for managing multiple related tasks (non-blocking)
pub struct AsyncTaskGroup {
    token: CancellationToken,
    tasks: Mutex<Vec<smol::Task<()>>>,
}

impl AsyncTaskGroup {
    /// Create a new task group
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            tasks: Mutex::new(Vec::new()),
        }
    }

    /// Add a task to the group (non-blocking)
    pub fn add_task<F, Fut>(&self, task: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let token = self.token.clone();
        let task_handle = smol::spawn(async move {
            if !token.is_cancelled() {
                task().await;
            }
        });

        self.tasks.lock().push(task_handle);
    }

    /// Cancel all tasks in the group
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Wait for all tasks to complete (non-blocking)
    pub async fn wait_all(&self) {
        let tasks = std::mem::take(&mut *self.tasks.lock());
        for task in tasks {
            let () = task.await;
        }
    }
}

impl Default for AsyncTaskGroup {
    fn default() -> Self {
        Self::new()
    }
}
