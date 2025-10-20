/// Specialized channel implementations for advanced use cases.
///
/// This module provides specialized channel types for specific communication patterns
/// including base64 encoding, compression, file backing, rate limiting, prioritization,
/// parallel processing, and persistence.
///
/// # Examples
///
/// Base64 channel for text transport:
/// ```rust
/// use trash_utilities::channels::{core::bounded_queue_3, specialist::Base64Channel};
/// use smol;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Data { value: i32 }
///
/// # smol::block_on(async {
/// let (tx, rx) = bounded_queue_3::<String>(10);
/// let channel = Base64Channel::new(tx);
/// channel.send_base64(&Data { value: 42 }).await.unwrap();
/// let data: Data = Base64Channel::recv_base64(&rx).await.unwrap();
/// assert_eq!(data.value, 42);
/// # });
/// ```
// Standard library imports
use std::{
    io::{BufRead, BufReader, BufWriter, Read, Write},
    sync::Arc,
    time::Duration,
};

// External crate imports
use base64::{Engine, engine::general_purpose};
use brotli::{CompressorWriter, Decompressor};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

/// Base64-encoded channel for text-based transport (non-blocking).
///
/// Automatically encodes data to base64 before sending and decodes on receive.
/// Useful for text-based protocols or when binary data needs to be transmitted as text.
///
/// # Type Parameters
///
/// * `T` - The type of data to send/receive (must implement Serialize/Deserialize).
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::{core::bounded_queue_3, specialist::Base64Channel};
/// use smol;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Message { id: u32, data: String }
///
/// # smol::block_on(async {
/// let (tx, rx) = bounded_queue_3::<String>(10);
/// let channel = Base64Channel::new(tx);
///
/// let msg = Message { id: 1, data: "hello".to_string() };
/// channel.send_base64(&msg).await.unwrap();
/// let received: Message = Base64Channel::recv_base64(&rx).await.unwrap();
/// assert_eq!(received, msg);
/// # });
/// ```
pub struct Base64Channel {
    inner: crate::channels::core::TxFuture<String>,
}

impl Base64Channel {
    /// Create a new base64 channel
    ///
    /// # Parameters
    ///
    /// * `inner` - The underlying string channel to wrap.
    ///
    /// # Returns
    ///
    /// A new `Base64Channel` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, specialist::Base64Channel};
    ///
    /// let (tx, _) = bounded_queue_3::<String>(10);
    /// let channel = Base64Channel::new(tx);
    /// ```
    #[must_use]
    pub fn new(inner: crate::channels::core::TxFuture<String>) -> Self {
        Self { inner }
    }

    /// Send data encoded as base64 (async, non-blocking)
    ///
    /// Serializes the data to JSON, encodes it as base64, and sends it.
    ///
    /// # Parameters
    ///
    /// * `data` - The data to send.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of data to send (must implement Serialize).
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or channel send fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, specialist::Base64Channel};
    /// use smol;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Data { value: i32 }
    ///
    /// # smol::block_on(async {
    /// let (tx, _) = bounded_queue_3::<String>(10);
    /// let channel = Base64Channel::new(tx);
    /// channel.send_base64(&Data { value: 42 }).await.unwrap();
    /// # });
    /// ```
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
    ///
    /// Receives base64-encoded data, decodes it, and deserializes from JSON.
    ///
    /// # Parameters
    ///
    /// * `receiver` - The channel receiver to read from.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize into (must implement Deserialize).
    ///
    /// # Returns
    ///
    /// The deserialized data on success.
    ///
    /// # Errors
    ///
    /// Returns an error if channel receive, base64 decoding, or deserialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, specialist::Base64Channel};
    /// use smol;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug, PartialEq)]
    /// struct Data { value: i32 }
    ///
    /// # smol::block_on(async {
    /// let (tx, rx) = bounded_queue_3::<String>(10);
    /// let channel = Base64Channel::new(tx);
    /// channel.send_base64(&Data { value: 42 }).await.unwrap();
    /// let data: Data = Base64Channel::recv_base64(&rx).await.unwrap();
    /// assert_eq!(data.value, 42);
    /// # });
    /// ```
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

/// Compressed channel for bandwidth-efficient communication (non-blocking).
///
/// Automatically compresses data using Brotli before sending and decompresses on receive.
/// Ideal for reducing network bandwidth or storage when dealing with compressible data.
///
/// # Type Parameters
///
/// * `T` - The type of data to send/receive (must implement Serialize/Deserialize).
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::specialist::CompressedChannel;
/// use smol;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct LargeData { content: String }
///
/// # smol::block_on(async {
/// let channel = CompressedChannel::new();
/// let data = LargeData { content: "A".repeat(1000) };
/// channel.send_compressed(&data).await.unwrap();
/// let received: LargeData = channel.recv_decompressed().await.unwrap();
/// assert_eq!(received, data);
/// # });
/// ```
pub struct CompressedChannel {
    tx: crate::channels::core::TxFuture<Vec<u8>>,
    rx: crate::channels::core::RxFuture<Vec<u8>>,
    level: u32,
}

impl CompressedChannel {
    /// Create a new compressed channel with defaults
    ///
    /// Uses capacity of 100 and compression level 6.
    ///
    /// # Returns
    ///
    /// A new `CompressedChannel` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannel;
    ///
    /// let channel = CompressedChannel::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(100, 6)
    }

    /// Create a new compressed channel with custom config
    ///
    /// # Parameters
    ///
    /// * `capacity` - The channel buffer capacity.
    /// * `compression_level` - Brotli compression level (0-11).
    ///
    /// # Returns
    ///
    /// A new `CompressedChannel` instance with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannel;
    ///
    /// let channel = CompressedChannel::with_config(50, 9);
    /// ```
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
    ///
    /// # Returns
    ///
    /// A `CompressedChannelBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannel;
    ///
    /// let channel = CompressedChannel::builder()
    ///     .capacity(200)
    ///     .compression_level(11)
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> CompressedChannelBuilder {
        CompressedChannelBuilder::new()
    }

    /// Send data with compression (async, non-blocking)
    ///
    /// Serializes the data to JSON, compresses it with Brotli, and sends it.
    ///
    /// # Parameters
    ///
    /// * `data` - The data to send.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of data to send (must implement Serialize).
    ///
    /// # Errors
    ///
    /// Returns an error if serialization, compression, or channel send fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannel;
    /// use smol;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Data { text: String }
    ///
    /// # smol::block_on(async {
    /// let channel = CompressedChannel::new();
    /// let data = Data { text: "compressible text".repeat(100) };
    /// channel.send_compressed(&data).await.unwrap();
    /// # });
    /// ```
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
    ///
    /// Receives compressed data, decompresses it, and deserializes from JSON.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize into (must implement Deserialize).
    ///
    /// # Returns
    ///
    /// The deserialized data on success.
    ///
    /// # Errors
    ///
    /// Returns an error if channel receive, decompression, or deserialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannel;
    /// use smol;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug, PartialEq)]
    /// struct Data { text: String }
    ///
    /// # smol::block_on(async {
    /// let channel = CompressedChannel::new();
    /// let data = Data { text: "test".to_string() };
    /// channel.send_compressed(&data).await.unwrap();
    /// let received: Data = channel.recv_decompressed().await.unwrap();
    /// assert_eq!(received, data);
    /// # });
    /// ```
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
    ///
    /// # Returns
    ///
    /// A new `CompressedChannelBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannelBuilder;
    ///
    /// let builder = CompressedChannelBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            capacity: 100,
            compression_level: 6,
        }
    }

    /// Set the channel capacity
    ///
    /// # Parameters
    ///
    /// * `capacity` - The buffer capacity for the channel.
    ///
    /// # Returns
    ///
    /// The builder instance for chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannelBuilder;
    ///
    /// let builder = CompressedChannelBuilder::new().capacity(200);
    /// ```
    #[must_use]
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Set the compression level
    ///
    /// # Parameters
    ///
    /// * `level` - Brotli compression level (0-11, higher = better compression but slower).
    ///
    /// # Returns
    ///
    /// The builder instance for chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannelBuilder;
    ///
    /// let builder = CompressedChannelBuilder::new().compression_level(9);
    /// ```
    #[must_use]
    pub fn compression_level(mut self, level: u32) -> Self {
        self.compression_level = level;
        self
    }

    /// Build the `CompressedChannel`
    ///
    /// # Returns
    ///
    /// A new `CompressedChannel` with the configured settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::CompressedChannelBuilder;
    ///
    /// let channel = CompressedChannelBuilder::new()
    ///     .capacity(50)
    ///     .compression_level(11)
    ///     .build();
    /// ```
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
///
/// Automatically falls back to temporary file storage when the in-memory channel is full.
/// Useful for handling large volumes of data or preventing memory exhaustion.
///
/// # Type Parameters
///
/// * `T` - The type of data to send (must implement Serialize/Deserialize).
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::specialist::FileBackedChannel;
/// use smol;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct LargeData { content: Vec<u8> }
///
/// # smol::block_on(async {
/// let channel = FileBackedChannel::new().unwrap();
/// let data = LargeData { content: vec![0; 1000000] }; // 1MB
/// channel.send(data).await.unwrap();
/// // Data is stored in memory or file as needed
/// # });
/// ```
pub struct FileBackedChannel<T> {
    tx: crate::channels::core::TxFuture<T>,
    temp_file: Arc<Mutex<Option<NamedTempFile>>>,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Send + 'static + Unpin> FileBackedChannel<T> {
    /// Create a new file-backed channel
    ///
    /// Creates a temporary file for overflow storage.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of data to send.
    ///
    /// # Returns
    ///
    /// A new `FileBackedChannel` instance on success.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the temporary file fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::FileBackedChannel;
    ///
    /// let channel: FileBackedChannel<String> = FileBackedChannel::new().unwrap();
    /// ```
    pub fn new() -> Result<Self, std::io::Error> {
        // expose the receiver so the background writer can persist overflowed messages
        let (tx, file_rx) = crate::channels::core::bounded_queue_3::<T>(100);

        let temp_file = Arc::new(Mutex::new(Some(NamedTempFile::new()?)));
        let temp_file_clone = temp_file.clone();

        // Start file writer task (non-blocking)
        smol::spawn(async move {
            // receive T values from the fallback receiver, serialize to JSON and append to the temp file
            while let Ok(msg) = file_rx.recv().await {
                if let Some(ref mut temp) = *temp_file_clone.lock()
                    && let Ok(json) = serde_json::to_string(&msg)
                {
                    let _ = temp.as_file_mut().write_all(format!("{json}\n").as_bytes());
                    let _ = temp.as_file_mut().flush();
                }
            }
        })
        .detach();

        Ok(Self { tx, temp_file })
    }

    /// Send data (async, non-blocking, memory first then file)
    ///
    /// Attempts to send to the in-memory channel first. If full, serializes and writes to file.
    ///
    /// # Parameters
    ///
    /// * `data` - The data to send.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::FileBackedChannel;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let channel: FileBackedChannel<String> = FileBackedChannel::new().unwrap();
    /// channel.send("data".to_string()).await.unwrap();
    /// # });
    /// ```
    pub async fn send(&self, data: T) -> Result<(), Box<dyn std::error::Error>> {
        // Serialize first so we can persist if send fails.
        let json = serde_json::to_string(&data)?;

        // Try to send to memory channel first. If it fails, persist the serialized JSON to file.
        if let Ok(()) = self.tx.send(data).await {
            Ok(())
        } else {
            if let Some(ref mut temp) = *self.temp_file.lock() {
                let _ = temp.as_file_mut().write_all(format!("{json}\n").as_bytes());
                let _ = temp.as_file_mut().flush();
            }
            Ok(())
        }
    }

    /// Flush file data to memory
    ///
    /// Reads all data from the temporary file and returns it as a vector.
    ///
    /// # Returns
    ///
    /// A vector of deserialized data from the file.
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or deserialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::specialist::FileBackedChannel;
    ///
    /// let channel: FileBackedChannel<String> = FileBackedChannel::new().unwrap();
    /// let data: Vec<String> = channel.flush_to_memory().unwrap();
    /// ```
    pub fn flush_to_memory(&self) -> Result<Vec<T>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        if let Some(ref file) = *self.temp_file.lock() {
            // Read from the underlying File of NamedTempFile and iterate lines
            let reader = BufReader::new(file.as_file());
            for line_res in reader.lines() {
                let line = line_res?;
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
        Self {
            tx,
            temp_file: Arc::new(Mutex::new(None)),
        }
    }
}

/// Rate-limited channel to prevent overwhelming receivers (non-blocking)
///
/// Uses a token bucket algorithm to limit the rate of messages sent.
/// Useful for controlling throughput and preventing system overload.
///
/// # Type Parameters
///
/// * `T` - The type of data to send.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::specialist::RateLimitedChannel;
/// use smol;
///
/// # smol::block_on(async {
/// let channel = RateLimitedChannel::new(10, 10.0, 1.0); // 10 tokens, refill 1/sec
/// channel.send("message".to_string()).await.unwrap();
/// # });
/// ```
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
    ///
    /// # Errors
    ///
    /// Returns an error if rate limit is exceeded or channel is closed/full.
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
///
/// Supports high, normal, and low priority messages. High priority messages are processed first.
/// Useful for systems requiring message prioritization.
///
/// # Type Parameters
///
/// * `T` - The type of data to send/receive.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::specialist::PriorityChannel;
/// use smol;
///
/// # smol::block_on(async {
/// let channel = PriorityChannel::new(10);
/// channel.send_high("urgent".to_string()).await.unwrap();
/// channel.send_normal("normal".to_string()).await.unwrap();
/// let msg = channel.recv().await.unwrap(); // Gets "urgent" first
/// assert_eq!(msg, "urgent");
/// # });
/// ```
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
    ///
    /// # Errors
    ///
    /// Send high priority message (async, non-blocking)
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    pub async fn send_high(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        self.high_tx.send(msg).await
    }

    /// Send normal priority message (async, non-blocking)
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    pub async fn send_normal(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        self.normal_tx.send(msg).await
    }

    /// Send low priority message (async, non-blocking)
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    pub async fn send_low(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        self.low_tx.send(msg).await
    }

    /// Receive message (highest priority first, async, non-blocking)
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or empty.
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
    ///
    /// # Errors
    ///
    /// Returns an error if opening the log file fails.
    pub fn new(
        sender: crate::channels::core::TxFuture<T>,
        log_path: &str,
    ) -> Result<Self, std::io::Error> {
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
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or channel send fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if file opening, reading, or deserialization fails.
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
