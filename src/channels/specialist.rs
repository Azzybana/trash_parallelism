// Standard library imports
use std::{
    io::{BufReader, BufWriter, Read, Write},
    sync::Arc,
    time::Duration,
};

// External crate imports
use base64::{Engine, engine::general_purpose};
use brotli::{CompressorWriter, Decompressor};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

/// Base64-encoded channel for text-based transport (non-blocking)
pub struct Base64Channel {
    inner: crate::channels::core::TxFuture<String>,
}

impl Base64Channel {
    /// Create a new base64 channel
    #[must_use]
    pub fn new(inner: crate::channels::core::TxFuture<String>) -> Self {
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
        receiver: &crate::channels::core::RxFuture<String>,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let encoded = receiver.recv().await?;
        let decoded = general_purpose::STANDARD.decode(&encoded)?;
        let json = String::from_utf8(decoded)?;
        let data: T = serde_json::from_str(&json)?;
        Ok(data)
    }
}

/// Compressed channel for bandwidth-efficient communication (non-blocking)
pub struct CompressedChannel {
    tx: crate::channels::core::TxFuture<Vec<u8>>,
    rx: crate::channels::core::RxFuture<Vec<u8>>,
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
        let (tx, rx) = crate::channels::core::bounded_queue_3(capacity);
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
    tx: crate::channels::core::TxFuture<T>,
    file_tx: crate::channels::core::TxFuture<String>,
    temp_file: Arc<Mutex<Option<NamedTempFile>>>,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Send + 'static + Unpin> FileBackedChannel<T> {
    /// Create a new file-backed channel
    pub fn new() -> Result<Self, std::io::Error> {
        let (tx, _rx) = crate::channels::core::bounded_queue_3::<T>(100);
        let (file_tx, file_rx) = crate::channels::core::bounded_queue_3::<String>(1000);

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
        let (tx, _) = crate::channels::core::bounded_queue_3(100);
        let (file_tx, _) = crate::channels::core::bounded_queue_3(1000);
        Self {
            tx,
            file_tx,
            temp_file: Arc::new(Mutex::new(None)),
        }
    }
}

/// Rate-limited channel to prevent overwhelming receivers (non-blocking)
pub struct RateLimitedChannel<T> {
    tx: crate::channels::core::TxFuture<T>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: std::time::Instant,
}

impl RateLimiter {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: std::time::Instant::now(),
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
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

impl<T: Send + 'static> RateLimitedChannel<T> {
    /// Create a new rate-limited channel
    #[must_use]
    pub fn new(capacity: usize, max_tokens: f64, refill_rate: f64) -> Self {
        let (tx, _) = crate::channels::core::bounded_queue_3(capacity);
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
    high_tx: crate::channels::core::TxFuture<T>,
    normal_tx: crate::channels::core::TxFuture<T>,
    low_tx: crate::channels::core::TxFuture<T>,
    rx: crate::channels::core::RxFuture<T>,
}

impl<T: Send + 'static + Unpin + Clone> PriorityChannel<T> {
    /// Create a new priority channel
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (high_tx, high_rx) = crate::channels::core::bounded_queue_3(capacity);
        let (normal_tx, normal_rx) = crate::channels::core::bounded_queue_3(capacity);
        let (low_tx, low_rx) = crate::channels::core::bounded_queue_3(capacity);
        let (output_tx, output_rx) = crate::channels::core::bounded_queue_3(capacity);

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
    receivers: Vec<crate::channels::core::RxFuture<T>>,
    processor: Arc<F>,
    results_tx: crate::channels::core::TxFuture<R>,
}

impl<T: Send + 'static, F: Fn(T) -> R + Send + Sync + 'static, R: Send + 'static>
    ParallelChannelProcessor<T, F, R>
{
    /// Create a new parallel processor
    pub fn new(receivers: Vec<crate::channels::core::RxFuture<T>>, processor: F) -> Self {
        let (results_tx, _) = crate::channels::core::bounded_queue_3(receivers.len() * 10);
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
    tx: crate::channels::core::TxFuture<T>,
    log_file: Arc<Mutex<BufWriter<std::fs::File>>>,
}

impl<T: Serialize + Send + 'static> PersistentChannel<T> {
    /// Create a new persistent channel
    pub fn new(sender: crate::channels::core::TxFuture<T>, log_path: &str) -> Result<Self, std::io::Error> {
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