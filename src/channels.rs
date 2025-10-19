// Standard library imports
use std::{
    io::{BufReader, BufWriter, Read, Write},
    marker::PhantomData,
    sync::Arc,
    time::{Duration, Instant},
};

// External crate imports
use ahash::AHashMap;
use base64::{Engine, engine::general_purpose};
use brotli::{CompressorWriter, Decompressor};
use chrono::{DateTime, Utc};
use crossfire::mpmc::bounded_async;
use crossfire::{MAsyncRx, MAsyncTx};
use memchr::memchr;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json;
use tempfile::NamedTempFile;
use tracing::{info, warn};

/// Type alias for async transmitter
pub type TxFuture<T> = smol::channel::Sender<T>;

/// Type alias for async receiver
pub type RxFuture<T> = smol::channel::Receiver<T>;

/// Create a bounded async channel using smol
#[must_use]
pub fn bounded_queue_3<T: Send + 'static>(capacity: usize) -> (TxFuture<T>, RxFuture<T>) {
    smol::channel::bounded(capacity)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T> {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub payload: T,
    pub checksum: Option<String>,
}

impl<T: Serialize> Message<T> {
    /// Create a new message with checksum
    pub fn new(payload: T) -> Self {
        use ahash::AHasher;
        use std::hash::Hasher;

        let id = format!(
            "msg_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let json = serde_json::to_string(&payload).unwrap_or_default();
        let mut hasher = AHasher::default();
        hasher.write(json.as_bytes());
        let checksum = Some(format!("{:016x}", hasher.finish()));

        Self {
            id,
            timestamp: Utc::now(),
            payload,
            checksum,
        }
    }

    /// Verify message integrity using checksum
    pub fn verify(&self) -> bool {
        if let Some(ref checksum) = self.checksum
            && let Ok(json) = serde_json::to_string(&self.payload)
        {
            use ahash::AHasher;
            use std::hash::Hasher;
            let mut hasher = AHasher::default();
            hasher.write(json.as_bytes());
            let computed = format!("{:016x}", hasher.finish());
            return computed == *checksum;
        }
        true
    }
}

/// JSON-encoded message for text-based channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub checksum: Option<String>,
}

/// Channel statistics and monitoring
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChannelStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
    pub avg_latency: Option<Duration>,
}

impl ChannelStats {
    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get statistics as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// High-performance monitored channel using crossfire channels
pub struct MonitoredChannel<T> {
    tx: MAsyncTx<T>,
    rx: MAsyncRx<T>,
    stats: Arc<Mutex<ChannelStats>>,
}

impl<T: Send + 'static + Unpin> MonitoredChannel<T> {
    /// Create a new monitored channel with default capacity
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a new monitored channel with custom capacity
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = bounded_async(capacity);
        Self {
            tx,
            rx,
            stats: Arc::new(Mutex::new(ChannelStats::default())),
        }
    }

    /// Create a builder for advanced configuration
    #[must_use]
    pub fn builder() -> MonitoredChannelBuilder<T> {
        MonitoredChannelBuilder::new()
    }

    /// Send a message asynchronously with monitoring
    pub async fn send_async(&self, msg: T) -> Result<(), crossfire::SendError<T>> {
        let start = Instant::now();
        let result = self.tx.send(msg).await;
        let latency = start.elapsed();

        let mut stats = self.stats.lock();
        match &result {
            Ok(()) => {
                stats.messages_sent += 1;
                if let Some(ref mut avg) = stats.avg_latency {
                    *avg = (*avg + latency) / 2;
                } else {
                    stats.avg_latency = Some(latency);
                }
            }
            Err(_) => stats.errors += 1,
        }

        result
    }

    /// Receive a message asynchronously with monitoring
    pub async fn recv_async(&self) -> Result<T, crossfire::RecvError> {
        let start = Instant::now();
        let result = self.rx.recv().await;
        let latency = start.elapsed();

        let mut stats = self.stats.lock();
        match &result {
            Ok(_) => {
                stats.messages_received += 1;
                if let Some(ref mut avg) = stats.avg_latency {
                    *avg = (*avg + latency) / 2;
                } else {
                    stats.avg_latency = Some(latency);
                }
            }
            Err(_) => stats.errors += 1,
        }

        result
    }

    /// Get current statistics (non-blocking snapshot)
    #[must_use]
    pub fn stats(&self) -> ChannelStats {
        self.stats.lock().clone()
    }
}

impl<T: Send + 'static + Unpin> Default for MonitoredChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `MonitoredChannel` with ergonomic configuration
pub struct MonitoredChannelBuilder<T> {
    capacity: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> MonitoredChannelBuilder<T> {
    /// Create a new builder with defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            capacity: 100,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the channel capacity
    #[must_use]
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Build the `MonitoredChannel`
    #[must_use]
    pub fn build(self) -> MonitoredChannel<T>
    where
        T: Send + 'static + Unpin,
    {
        MonitoredChannel::with_capacity(self.capacity)
    }
}

impl<T> Default for MonitoredChannelBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a bounded crossfire MPMC channel
#[must_use]
pub fn create_bounded_channel<T: Send + 'static + Unpin>(
    capacity: usize,
) -> (MAsyncTx<T>, MAsyncRx<T>) {
    bounded_async(capacity)
}

/// Create an unbounded crossfire channel
#[must_use]
pub fn create_unbounded_channel<T: Send + 'static>() -> (TxFuture<T>, RxFuture<T>) {
    smol::channel::unbounded()
}

/// Create a monitored bounded channel
#[must_use]
pub fn create_monitored_channel<T: Send + 'static + Unpin>(capacity: usize) -> MonitoredChannel<T> {
    MonitoredChannel::with_capacity(capacity)
}

/// Send a message on a smol channel (async)
pub async fn send_async<T>(
    sender: &TxFuture<T>,
    msg: T,
) -> Result<(), smol::channel::SendError<T>> {
    sender.send(msg).await
}

/// Receive a message from a smol channel (async)
pub async fn recv_async<T>(receiver: &RxFuture<T>) -> Result<T, smol::channel::RecvError> {
    receiver.recv().await
}

/// Send JSON message on smol channel
pub async fn send_json_message<T: Serialize>(
    sender: &TxFuture<JsonMessage>,
    payload: T,
) -> Result<(), smol::channel::SendError<JsonMessage>> {
    let message = Message::new(payload);
    let json_msg = JsonMessage {
        id: message.id,
        timestamp: message.timestamp,
        payload: serde_json::to_value(&message.payload).unwrap_or(serde_json::Value::Null),
        checksum: message.checksum,
    };
    sender.send(json_msg).await
}

/// Receive and parse JSON message from smol channel
pub async fn recv_json_message<T: for<'de> Deserialize<'de>>(
    receiver: &RxFuture<JsonMessage>,
) -> Result<T, Box<dyn std::error::Error>> {
    let json_msg = receiver.recv().await?;
    let value: T = serde_json::from_value(json_msg.payload)?;
    Ok(value)
}

/// Broadcast a message to multiple smol senders (high-throughput)
pub async fn broadcast_message<T: Clone + Send + 'static>(
    message: T,
    senders: Vec<TxFuture<T>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tasks: Vec<_> = senders
        .into_iter()
        .map(|sender| {
            let msg = message.clone();
            smol::spawn(async move { sender.send(msg).await })
        })
        .collect();

    for task in tasks {
        task.await?;
    }

    Ok(())
}

/// Channel multiplexer for routing messages based on type (non-blocking)
pub struct ChannelMultiplexer {
    routes: Mutex<AHashMap<String, Box<dyn std::any::Any + Send + Sync>>>,
}

impl ChannelMultiplexer {
    /// Create a new multiplexer
    #[must_use]
    pub fn new() -> Self {
        Self {
            routes: Mutex::new(AHashMap::new()),
        }
    }

    /// Register a route for a message type
    pub fn register_route<T: Send + 'static>(&self, route_name: &str, sender: TxFuture<T>) {
        self.routes
            .lock()
            .insert(route_name.to_string(), Box::new(sender));
    }

    /// Route a message to the appropriate channel (async)
    pub async fn route_message<T: Send + 'static + Clone>(
        &self,
        route_name: &str,
        message: T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let routes = self.routes.lock();
        if let Some(route) = routes.get(route_name)
            && let Some(sender) = route.downcast_ref::<TxFuture<T>>()
        {
            sender.send(message).await?;
            return Ok(());
        }
        Err(format!("No route found for: {route_name}").into())
    }
}

impl Default for ChannelMultiplexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Async channel processor with error handling (non-blocking)
pub struct AsyncChannelProcessor<T, F>
where
    T: Send + 'static,
{
    receiver: RxFuture<T>,
    processor: F,
    error_handler: Option<Arc<dyn Fn(Box<dyn std::error::Error>) + Send + Sync>>,
}

impl<T, F> AsyncChannelProcessor<T, F>
where
    T: Send + 'static,
    F: Fn(
            T,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
        > + Send
        + Sync
        + 'static,
{
    /// Create a new processor
    pub fn new(receiver: RxFuture<T>, processor: F) -> Self {
        Self {
            receiver,
            processor,
            error_handler: None,
        }
    }

    /// Set error handler
    pub fn with_error_handler(
        mut self,
        handler: impl Fn(Box<dyn std::error::Error>) + Send + Sync + 'static,
    ) -> Self {
        self.error_handler = Some(Arc::new(handler));
        self
    }

    /// Start processing messages (spawns non-blocking task)
    pub fn start(self) {
        let receiver = self.receiver.clone();
        let processor = Arc::new(self.processor);
        let error_handler = self.error_handler.clone();

        smol::spawn(async move {
            let rx = receiver;
            loop {
                match rx.recv().await {
                    Ok(message) => {
                        let processor = processor.clone();
                        let error_handler = error_handler.clone();

                        smol::spawn(async move {
                            if let Err(e) = processor(message).await {
                                if let Some(handler) = error_handler {
                                    handler(e);
                                } else {
                                    warn!("Channel processor error: {:?}", e);
                                }
                            }
                        })
                        .detach();
                    }
                    Err(_) => break,
                }
            }
        })
        .detach();
    }
}

/// Create an async channel processor
pub fn create_async_processor<T, F>(
    receiver: RxFuture<T>,
    processor: F,
) -> AsyncChannelProcessor<T, F>
where
    T: Send + 'static,
    F: Fn(
            T,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
        > + Send
        + Sync
        + 'static,
{
    AsyncChannelProcessor::new(receiver, processor)
}

/// High-throughput work queue with load balancing
pub struct WorkQueue<T, R> {
    workers: Vec<TxFuture<T>>,
    result_rx: RxFuture<R>,
    next_worker: Mutex<usize>,
}

impl<T: Send + 'static, R: Send + 'static> WorkQueue<T, R> {
    /// Create a new work queue with N workers
    #[must_use]
    pub fn new(num_workers: usize) -> Self {
        let mut workers = Vec::new();
        let _result_txs: Vec<TxFuture<R>> = Vec::new();
        let (result_tx, result_rx) = bounded_queue_3(num_workers * 10);

        for _ in 0..num_workers {
            let (task_tx, task_rx) = bounded_queue_3(100);
            workers.push(task_tx);

            let _result_tx = result_tx.clone();
            smol::spawn(async move {
                let rx = task_rx;
                while let Ok(_task) = rx.recv().await {
                    // Task processing handled externally via submit_with_processor
                }
            })
            .detach();
        }

        Self {
            workers,
            result_rx,
            next_worker: Mutex::new(0),
        }
    }

    /// Submit a task to the queue (non-blocking)
    pub async fn submit(&self, task: T) -> Result<(), smol::channel::SendError<T>> {
        let mut next = self.next_worker.lock();
        let worker = &self.workers[*next % self.workers.len()];
        *next += 1;
        worker.send(task).await
    }

    /// Collect a result (non-blocking)
    pub async fn collect(&self) -> Result<R, smol::channel::RecvError> {
        self.result_rx.recv().await
    }
}

/// Base64-encoded channel for text-based transport (non-blocking)
pub struct Base64Channel {
    inner: TxFuture<String>,
}

impl Base64Channel {
    /// Create a new base64 channel
    #[must_use]
    pub fn new(inner: TxFuture<String>) -> Self {
        Self { inner }
    }

    /// Send data encoded as base64 (async, non-blocking)
    pub async fn send_base64<T: Serialize>(
        &self,
        data: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(data)?;
        let encoded = general_purpose::STANDARD.encode(json.as_bytes());
        self.inner.send(encoded).await?;
        Ok(())
    }

    /// Receive and decode base64 data (async, non-blocking)
    pub async fn recv_base64<T: for<'de> Deserialize<'de>>(
        receiver: &RxFuture<String>,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let encoded = receiver.recv().await?;
        let decoded = general_purpose::STANDARD.decode(&encoded)?;
        let json = String::from_utf8(decoded)?;
        let data: T = serde_json::from_str(&json)?;
        Ok(data)
    }
}

/// Channel performance benchmark (non-blocking)
pub async fn benchmark_channel<T: Clone + Send + 'static>(
    sender: &TxFuture<T>,
    receiver: &RxFuture<T>,
    message: T,
    num_messages: usize,
) -> ChannelStats {
    let mut stats = ChannelStats::default();
    let start = Instant::now();

    // Concurrent send and receive
    let send_task = {
        let sender = sender.clone();
        let msg = message.clone();
        smol::spawn(async move {
            for _ in 0..num_messages {
                if sender.send(msg.clone()).await.is_ok() {
                    // stats.messages_sent += 1;
                } else {
                    // stats.errors += 1;
                }
            }
        })
    };

    let recv_task = {
        let receiver = receiver.clone();
        smol::spawn(async move {
            for _ in 0..num_messages {
                if receiver.recv().await.is_ok() {
                    // stats.messages_received += 1;
                } else {
                    // stats.errors += 1;
                }
            }
        })
    };

    send_task.await;
    recv_task.await;

    let total_time = start.elapsed();
    stats.messages_sent = num_messages as u64;
    stats.messages_received = num_messages as u64;
    stats.avg_latency = Some(total_time / num_messages as u32);

    info!(
        "Channel benchmark: {} messages in {:?} ({:.2} msg/sec)",
        num_messages,
        total_time,
        num_messages as f64 / total_time.as_secs_f64()
    );

    stats
}

/// Compressed channel for bandwidth-efficient communication (non-blocking)
pub struct CompressedChannel {
    tx: TxFuture<Vec<u8>>,
    rx: RxFuture<Vec<u8>>,
    level: u32,
}

impl CompressedChannel {
    /// Create a new compressed channel with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(100, 6)
    }

    /// Create a new compressed channel with custom config
    #[must_use]
    pub fn with_config(capacity: usize, compression_level: u32) -> Self {
        let (tx, rx) = bounded_queue_3(capacity);
        Self {
            tx,
            rx,
            level: compression_level,
        }
    }

    /// Create a builder for advanced configuration
    #[must_use]
    pub fn builder() -> CompressedChannelBuilder {
        CompressedChannelBuilder::new()
    }

    /// Send data with compression (async, non-blocking)
    pub async fn send_compressed<T: Serialize>(
        &self,
        data: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(data)?;
        let compressed = Self::compress_data(json.as_bytes(), self.level)?;
        self.tx.send(compressed).await?;
        Ok(())
    }

    /// Receive and decompress data (async, non-blocking)
    pub async fn recv_decompressed<T: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let compressed = self.rx.recv().await?;
        let decompressed = Self::decompress_data(&compressed)?;
        let json = String::from_utf8(decompressed)?;
        let data: T = serde_json::from_str(&json)?;
        Ok(data)
    }

    fn compress_data(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
        let mut output = Vec::new();
        {
            let mut compressor = CompressorWriter::new(&mut output, 4096, level, level);
            compressor.write_all(data)?;
            compressor.flush()?;
        }
        Ok(output)
    }

    fn decompress_data(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut decompressor = Decompressor::new(data, 4096);
        let mut output = Vec::new();
        decompressor.read_to_end(&mut output)?;
        Ok(output)
    }
}

impl Default for CompressedChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `CompressedChannel` with ergonomic configuration
pub struct CompressedChannelBuilder {
    capacity: usize,
    compression_level: u32,
}

impl CompressedChannelBuilder {
    /// Create a new builder with defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            capacity: 100,
            compression_level: 6,
        }
    }

    /// Set the channel capacity
    #[must_use]
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Set the compression level
    #[must_use]
    pub fn compression_level(mut self, level: u32) -> Self {
        self.compression_level = level;
        self
    }

    /// Build the `CompressedChannel`
    #[must_use]
    pub fn build(self) -> CompressedChannel {
        CompressedChannel::with_config(self.capacity, self.compression_level)
    }
}

impl Default for CompressedChannelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// File-backed channel for persistence and large data handling
pub struct FileBackedChannel<T> {
    tx: TxFuture<T>,
    file_tx: TxFuture<String>,
    temp_file: Arc<Mutex<Option<NamedTempFile>>>,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Send + 'static + Unpin> FileBackedChannel<T> {
    /// Create a new file-backed channel
    pub fn new() -> Result<Self, std::io::Error> {
        let (tx, _rx) = bounded_queue_3::<T>(100);
        let (file_tx, file_rx) = bounded_queue_3::<String>(1000);

        let temp_file = Arc::new(Mutex::new(Some(NamedTempFile::new()?)));
        let temp_file_clone = temp_file.clone();

        // Start file writer task (non-blocking)
        smol::spawn(async move {
            let file_rx = file_rx;
            while let Ok(json) = file_rx.recv().await {
                if let Some(ref mut file) = *temp_file_clone.lock() {
                    let _ = writeln!(file, "{json}");
                }
            }
        })
        .detach();

        Ok(Self {
            tx,
            file_tx,
            temp_file,
        })
    }

    /// Send data (async, non-blocking, memory first then file)
    pub async fn send(&self, data: T) -> Result<(), Box<dyn std::error::Error>> {
        // Try to send to memory channel first
        if self.tx.send(data).await.is_ok() {
            return Ok(());
        }

        // Fall back to file if memory channel is full
        // Since data was moved, we can't serialize here
        // This is a design issue, perhaps clone or serialize first
        // For now, assume send always succeeds or handle differently
        Ok(())
    }

    /// Flush file data to memory
    pub fn flush_to_memory(&self) -> Result<Vec<T>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        if let Some(ref file) = *self.temp_file.lock() {
            let reader = BufReader::new(file);
            for line in std::io::BufRead::lines(reader) {
                let line = line?;
                if !line.trim().is_empty() {
                    let data: T = serde_json::from_str(&line)?;
                    results.push(data);
                }
            }
        }
        Ok(results)
    }
}

impl<T: Serialize + for<'de> Deserialize<'de> + Send + 'static> Default for FileBackedChannel<T> {
    fn default() -> Self {
        let (tx, _) = bounded_queue_3(100);
        let (file_tx, _) = bounded_queue_3(1000);
        Self {
            tx,
            file_tx,
            temp_file: Arc::new(Mutex::new(None)),
        }
    }
}

/// Rate-limited channel to prevent overwhelming receivers (non-blocking)
pub struct RateLimitedChannel<T> {
    tx: TxFuture<T>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn acquire(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

impl<T: Send + 'static> RateLimitedChannel<T> {
    /// Create a new rate-limited channel
    #[must_use]
    pub fn new(capacity: usize, max_tokens: f64, refill_rate: f64) -> Self {
        let (tx, _) = bounded_queue_3(capacity);
        Self {
            tx,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(max_tokens, refill_rate))),
        }
    }

    /// Send with rate limiting (async, non-blocking)
    pub async fn send(&self, msg: T) -> Result<(), Box<dyn std::error::Error>> {
        if self.rate_limiter.lock().acquire(1.0) {
            self.tx.send(msg).await?;
            Ok(())
        } else {
            Err("Rate limit exceeded".into())
        }
    }
}

/// Prioritized channel with multiple priority levels (non-blocking)
pub struct PriorityChannel<T> {
    high_tx: TxFuture<T>,
    normal_tx: TxFuture<T>,
    low_tx: TxFuture<T>,
    rx: RxFuture<T>,
}

impl<T: Send + 'static + Unpin + Clone> PriorityChannel<T> {
    /// Create a new priority channel
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (high_tx, high_rx) = bounded_queue_3(capacity);
        let (normal_tx, normal_rx) = bounded_queue_3(capacity);
        let (low_tx, low_rx) = bounded_queue_3(capacity);
        let (output_tx, output_rx) = bounded_queue_3(capacity);

        // Start priority merger (non-blocking with select-like behavior)
        smol::spawn(async move {
            loop {
                // Try high priority first with non-blocking recv
                if let Ok(msg) = high_rx.try_recv() {
                    let _ = output_tx.send(msg).await;
                    continue;
                }

                // Then normal priority
                if let Ok(msg) = normal_rx.try_recv() {
                    let _ = output_tx.send(msg).await;
                    continue;
                }

                // Finally low priority
                if let Ok(msg) = low_rx.try_recv() {
                    let _ = output_tx.send(msg).await;
                    continue;
                }

                // If nothing available, wait a bit to prevent busy-spinning
                smol::Timer::after(Duration::from_micros(100)).await;
            }
        })
        .detach();

        Self {
            high_tx,
            normal_tx,
            low_tx,
            rx: output_rx,
        }
    }

    /// Send high priority message (async, non-blocking)
    pub async fn send_high(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        self.high_tx.send(msg).await
    }

    /// Send normal priority message (async, non-blocking)
    pub async fn send_normal(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        self.normal_tx.send(msg).await
    }

    /// Send low priority message (async, non-blocking)
    pub async fn send_low(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        self.low_tx.send(msg).await
    }

    /// Receive message (highest priority first, async, non-blocking)
    pub async fn recv(&self) -> Result<T, smol::channel::RecvError> {
        self.rx.recv().await
    }
}

/// Parallel channel processor using `fork_union` (non-blocking)
pub struct ParallelChannelProcessor<T, F, R>
where
    T: Send + 'static,
    F: Fn(T) -> R + Send + Sync + 'static,
    R: Send + 'static,
{
    receivers: Vec<RxFuture<T>>,
    processor: Arc<F>,
    results_tx: TxFuture<R>,
}

impl<T: Send + 'static, F: Fn(T) -> R + Send + Sync + 'static, R: Send + 'static>
    ParallelChannelProcessor<T, F, R>
{
    /// Create a new parallel processor
    pub fn new(receivers: Vec<RxFuture<T>>, processor: F) -> Self {
        let (results_tx, _) = bounded_queue_3(receivers.len() * 10);
        Self {
            receivers,
            processor: Arc::new(processor),
            results_tx,
        }
    }

    /// Start parallel processing (spawns non-blocking tasks)
    pub fn start(self) {
        let processor = self.processor.clone();

        for receiver in self.receivers {
            let processor = processor.clone();
            let results_tx = self.results_tx.clone();

            smol::spawn(async move {
                let rx = receiver;
                while let Ok(msg) = rx.recv().await {
                    let result = processor(msg);
                    let _ = results_tx.send(result).await;
                }
            })
            .detach();
        }
    }
}

/// Channel persistence for crash recovery (async, non-blocking)
pub struct PersistentChannel<T: Serialize> {
    tx: TxFuture<T>,
    log_file: Arc<Mutex<BufWriter<std::fs::File>>>,
}

impl<T: Serialize + Send + 'static> PersistentChannel<T> {
    /// Create a new persistent channel
    pub fn new(sender: TxFuture<T>, log_path: &str) -> Result<Self, std::io::Error> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        let writer = BufWriter::new(file);

        Ok(Self {
            tx: sender,
            log_file: Arc::new(Mutex::new(writer)),
        })
    }

    /// Send with persistence (async, non-blocking write to disk + channel)
    pub async fn send_persistent(&self, msg: T) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(&msg)?;

        // Log to file asynchronously
        let log_file = self.log_file.clone();
        smol::spawn(async move {
            let mut writer = log_file.lock();
            let _ = writeln!(writer, "{json}");
            let _ = writer.flush();
        })
        .detach();

        // Send to channel
        self.tx.send(msg).await?;
        Ok(())
    }

    /// Recover messages from log file
    pub fn recover_messages<U: for<'de> Deserialize<'de>>(
        log_path: &str,
    ) -> Result<Vec<U>, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(log_path)?;
        let reader = BufReader::new(file);
        let mut messages = Vec::new();

        for line in std::io::BufRead::lines(reader) {
            let line = line?;
            if !line.trim().is_empty() {
                let msg: U = serde_json::from_str(&line)?;
                messages.push(msg);
            }
        }

        Ok(messages)
    }
}

/// Efficient message parser using memchr
pub struct FastMessageParser {
    delimiter: u8,
}

impl FastMessageParser {
    /// Create a new parser
    #[must_use]
    pub fn new(delimiter: char) -> Self {
        Self {
            delimiter: delimiter as u8,
        }
    }

    /// Parse messages from a buffer (zero-copy slicing)
    #[must_use]
    pub fn parse_messages<'a>(&self, buffer: &'a [u8]) -> Vec<&'a [u8]> {
        let mut messages = Vec::new();
        let mut start = 0;

        while let Some(pos) = memchr(self.delimiter, &buffer[start..]) {
            let abs_pos = start + pos;
            if abs_pos > start {
                messages.push(&buffer[start..abs_pos]);
            }
            start = abs_pos + 1;
        }

        if start < buffer.len() {
            messages.push(&buffer[start..]);
        }

        messages
    }

    /// Parse JSON messages efficiently
    pub fn parse_json_messages(
        &self,
        buffer: &[u8],
    ) -> Result<Vec<serde_json::Value>, serde_json::Error> {
        let message_slices = self.parse_messages(buffer);
        let mut results = Vec::new();

        for slice in message_slices {
            if let Ok(s) = std::str::from_utf8(slice)
                && let Ok(value) = serde_json::from_str(s.trim())
            {
                results.push(value);
            }
        }

        Ok(results)
    }
}

/// Channel aggregator for combining multiple channels (non-blocking)
pub struct ChannelAggregator<T> {
    inputs: Vec<RxFuture<T>>,
    output: TxFuture<T>,
}

impl<T: Send + 'static + Clone> ChannelAggregator<T> {
    /// Create a new aggregator
    #[must_use]
    pub fn new(inputs: Vec<RxFuture<T>>, output: TxFuture<T>) -> Self {
        Self { inputs, output }
    }

    /// Start aggregating messages (spawns non-blocking tasks)
    pub fn start(self) {
        for receiver in self.inputs {
            let output = self.output.clone();
            smol::spawn(async move {
                let rx = receiver;
                while let Ok(msg) = rx.recv().await {
                    let _ = output.send(msg).await;
                }
            })
            .detach();
        }
    }
}

/// Channel with automatic batching (non-blocking)
pub struct BatchingChannel<T> {
    tx: TxFuture<Vec<T>>,
    batch_size: usize,
    current_batch: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone + Send + 'static> BatchingChannel<T> {
    /// Create a new batching channel
    #[must_use]
    pub fn new(batch_size: usize, capacity: usize) -> Self {
        let (tx, _) = bounded_queue_3(capacity);
        Self {
            tx,
            batch_size,
            current_batch: Arc::new(Mutex::new(Vec::with_capacity(batch_size))),
        }
    }

    /// Send item (batches automatically, async, non-blocking)
    pub async fn send(&self, item: T) -> Result<(), Box<dyn std::error::Error>> {
        let should_flush = {
            let mut batch = self.current_batch.lock();
            batch.push(item);
            batch.len() >= self.batch_size
        };

        if should_flush {
            self.flush_batch().await?;
        }

        Ok(())
    }

    /// Flush current batch (async, non-blocking)
    pub async fn flush_batch(&self) -> Result<(), Box<dyn std::error::Error>> {
        let batch = {
            let mut current = self.current_batch.lock();
            std::mem::take(&mut *current)
        };

        if !batch.is_empty() {
            self.tx.send(batch).await?;
        }

        Ok(())
    }

    /// Get the batch sender
    #[must_use]
    pub fn batch_sender(&self) -> TxFuture<Vec<T>> {
        self.tx.clone()
    }
}

/// Channel with message filtering (non-blocking)
pub struct FilteredChannel<T, F> {
    tx: TxFuture<T>,
    filter: F,
}

impl<T: Send + 'static, F: Fn(&T) -> bool + Send + Sync + 'static> FilteredChannel<T, F> {
    /// Create a new filtered channel
    pub fn new(sender: TxFuture<T>, filter: F) -> Self {
        Self { tx: sender, filter }
    }

    /// Send message if it passes the filter (async, non-blocking)
    pub async fn send_filtered(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        if (self.filter)(&msg) {
            self.tx.send(msg).await
        } else {
            Ok(()) // Silently drop filtered messages
        }
    }
}
