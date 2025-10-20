//! Async stream processing utilities.
//!
//! This module provides streaming I/O operations, channel-based communication,
//! and async file processing with chunked reading and processing.
//!
//! ## Features
//!
//! - **Async File Processing**: Streaming file readers with chunked processing
//! - **Channel Integration**: Async channel creation and communication
//! - **Stream Processing**: Async stream utilities using futures-lite
//! - **Memory Efficient**: Chunked reading to handle large files
//! - **Progress Tracking**: Optional progress callbacks during processing

// Standard library imports
use std::sync::Arc;

// External crate imports
use bytes::Bytes;
use futures_lite::{AsyncReadExt, StreamExt};
use smol::channel::{Receiver, Sender, unbounded};

/// Creates an unbounded channel for asynchronous communication
///
/// Leverages smol channels for efficient async communication.
/// This provides a simple interface while using the proven smol implementation.
///
/// # Type Parameters
/// - `T`: The type of messages to be sent through the channel
///
/// # Returns
/// A tuple `(Sender<T>, Receiver<T>)` for sending and receiving messages
///
/// # Examples
/// ```rust
/// use trash_utilities::io::create_channel;
/// use smol::channel::Sender;
///
/// let (tx, rx) = create_channel::<String>();
/// // Now you can send and receive messages asynchronously
/// ```
#[must_use]
pub fn create_channel<T>() -> (Sender<T>, Receiver<T>) {
    unbounded()
}

/// Async file processor with streaming capabilities
///
/// Processes files in chunks to handle large files efficiently.
/// Supports progress tracking and async processing functions.
pub struct AsyncFileProcessor {
    buffer_size: usize,
    progress_callback: Option<Box<dyn Fn(u64) + Send + Sync>>,
}

impl AsyncFileProcessor {
    /// Create a new file processor with default buffer size
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(8192, None)
    }

    /// Create a new file processor with custom configuration
    ///
    /// # Parameters
    /// - `buffer_size`: Size of chunks to read at once
    /// - `progress_callback`: Optional callback for progress updates
    ///
    /// # Returns
    /// Configured `AsyncFileProcessor` instance
    #[must_use]
    pub fn with_config(
        buffer_size: usize,
        progress_callback: Option<Box<dyn Fn(u64) + Send + Sync>>,
    ) -> Self {
        Self {
            buffer_size,
            progress_callback,
        }
    }

    /// Create a builder for advanced configuration
    #[must_use]
    pub fn builder() -> AsyncFileProcessorBuilder {
        AsyncFileProcessorBuilder::new()
    }

    /// Process a file asynchronously with a streaming function
    ///
    /// Reads the file in chunks and applies the processor function to each chunk.
    /// This is memory-efficient for large files.
    ///
    /// # Type Parameters
    /// - `F`: Processor function type
    /// - `T`: Result type from processing
    ///
    /// # Parameters
    /// - `path`: Path to the file to process
    /// - `processor`: Function to process each chunk
    ///
    /// # Returns
    /// Vector of results from processing each chunk
    ///
    /// # Errors
    /// Returns error if file reading fails
    pub async fn process_file<F, T>(
        &self,
        path: &str,
        processor: F,
    ) -> Result<Vec<T>, std::io::Error>
    where
        F: Fn(Bytes) -> T + Send + Sync,
        T: Send + 'static,
    {
        let mut file = smol::fs::File::open(path).await?;
        let mut results = Vec::new();
        let mut buffer = vec![0u8; self.buffer_size];
        let mut bytes_processed = 0u64;

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk = Bytes::copy_from_slice(&buffer[..bytes_read]);
            let result = processor(chunk);
            results.push(result);

            bytes_processed += bytes_read as u64;

            // Update progress if callback provided
            if let Some(ref callback) = self.progress_callback {
                callback(bytes_processed);
            }
        }

        Ok(results)
    }

    /// Process a file with async processor function
    ///
    /// Similar to `process_file` but allows async processing functions.
    ///
    /// # Type Parameters
    /// - `F`: Async processor function type
    /// - `T`: Result type from processing
    ///
    /// # Parameters
    /// - `path`: Path to the file to process
    /// - `processor`: Async function to process each chunk
    ///
    /// # Returns
    /// Vector of results from processing each chunk
    ///
    /// # Errors
    /// Returns error if file reading or processing fails
    pub async fn process_file_async<F, Fut, T>(
        &self,
        path: &str,
        processor: F,
    ) -> Result<Vec<T>, std::io::Error>
    where
        F: Fn(Bytes) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T, std::io::Error>> + Send,
        T: Send + 'static,
    {
        let mut file = smol::fs::File::open(path).await?;
        let mut results = Vec::new();
        let mut buffer = vec![0u8; self.buffer_size];
        let mut bytes_processed = 0u64;

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk = Bytes::copy_from_slice(&buffer[..bytes_read]);
            let result = processor(chunk).await?;
            results.push(result);

            bytes_processed += bytes_read as u64;

            // Update progress if callback provided
            if let Some(ref callback) = self.progress_callback {
                callback(bytes_processed);
            }
        }

        Ok(results)
    }
}

impl Default for AsyncFileProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `AsyncFileProcessor` with ergonomic configuration
pub struct AsyncFileProcessorBuilder {
    buffer_size: usize,
    progress_callback: Option<Box<dyn Fn(u64) + Send + Sync>>,
}

impl AsyncFileProcessorBuilder {
    /// Create a new builder with defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer_size: 8192,
            progress_callback: None,
        }
    }

    /// Set the buffer size for reading chunks
    #[must_use]
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set a progress callback function
    #[must_use]
    pub fn progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Build the `AsyncFileProcessor`
    #[must_use]
    pub fn build(self) -> AsyncFileProcessor {
        AsyncFileProcessor {
            buffer_size: self.buffer_size,
            progress_callback: self.progress_callback,
        }
    }
}

impl Default for AsyncFileProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Asynchronously process a file using a provided function
///
/// Enhanced version that leverages async I/O utilities and provides
/// better error handling than the original.
///
/// # Type Parameters
/// - `F`: The type of the processor function
///
/// # Parameters
/// - `path`: The path to the file to read and process
/// - `processor`: A function that takes the file content and performs some operation
///
/// # Returns
/// - `Ok(())` if the file was read and processed successfully
/// - `Err(std::io::Error)` if reading the file failed
///
/// # Errors
/// Returns an error if the file cannot be read.
///
/// # Examples
/// ```rust,no_run
/// use trash_utilities::io::process_file_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     process_file_async("data.txt", |content| {
///         println!("File length: {}", content.len());
///     }).await?;
///     Ok(())
/// }
/// ```
pub async fn process_file_async<F>(path: &str, processor: F) -> Result<(), std::io::Error>
where
    F: Fn(String) + Send + Sync + 'static,
{
    let content = crate::io::utils::read_file_async(path).await?;
    processor(content);
    Ok(())
}

/// Async stream utilities for processing data streams
///
/// Provides utilities for working with async streams, including
/// transformation, filtering, and collection operations.
pub struct AsyncStreamUtils;

impl AsyncStreamUtils {
    /// Process an async stream with a transformation function
    ///
    /// # Type Parameters
    /// - `S`: Stream type
    /// - `F`: Transformation function type
    /// - `T`: Input item type
    /// - `U`: Output item type
    ///
    /// # Parameters
    /// - `stream`: Input async stream
    /// - `transformer`: Function to transform each item
    ///
    /// # Returns
    /// Vector of transformed results
    pub async fn map_stream<S, F, T, U>(mut stream: S, transformer: F) -> Vec<U>
    where
        S: futures_lite::Stream<Item = T> + Unpin,
        F: Fn(T) -> U,
        U: Send + 'static,
    {
        let mut results = Vec::new();

        while let Some(item) = stream.next().await {
            let transformed_item = transformer(item);
            results.push(transformed_item);
        }

        results
    }

    /// Filter an async stream based on a predicate
    ///
    /// # Type Parameters
    /// - `S`: Stream type
    /// - `F`: Predicate function type
    /// - `T`: Item type
    ///
    /// # Parameters
    /// - `stream`: Input async stream
    /// - `predicate`: Function that returns true for items to keep
    ///
    /// # Returns
    /// Vector of filtered results
    pub async fn filter_stream<S, F, T>(mut stream: S, predicate: F) -> Vec<T>
    where
        S: futures_lite::Stream<Item = T> + Unpin,
        F: Fn(&T) -> bool,
        T: Send + 'static,
    {
        let mut results = Vec::new();

        while let Some(item) = stream.next().await {
            if predicate(&item) {
                results.push(item);
            }
        }

        results
    }

    /// Collect an async stream into a vector
    ///
    /// # Type Parameters
    /// - `S`: Stream type
    /// - `T`: Item type
    ///
    /// # Parameters
    /// - `stream`: Input async stream
    ///
    /// # Returns
    /// Vector containing all stream items
    pub async fn collect_stream<S, T>(mut stream: S) -> Vec<T>
    where
        S: futures_lite::Stream<Item = T> + Unpin,
        T: Send + 'static,
    {
        let mut results = Vec::new();

        while let Some(item) = stream.next().await {
            results.push(item);
        }

        results
    }
}

/// Channel-based stream processor
///
/// Processes data through channels with async streaming capabilities.
/// Useful for producer-consumer patterns and data pipeline processing.
pub struct ChannelStreamProcessor<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
    processor: Arc<dyn Fn(T) -> T + Send + Sync>,
}

impl<T> ChannelStreamProcessor<T>
where
    T: Send + 'static + Clone,
{
    /// Create a new channel stream processor
    ///
    /// # Parameters
    /// - `processor`: Function to process each item
    ///
    /// # Returns
    /// New `ChannelStreamProcessor` instance
    #[must_use]
    pub fn new<F>(processor: F) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'static,
    {
        let (sender, receiver) = create_channel();
        Self {
            sender,
            receiver,
            processor: Arc::new(processor),
        }
    }

    /// Send data to be processed
    ///
    /// # Parameters
    /// - `data`: Data to send for processing
    ///
    /// # Returns
    /// Success or channel error
    ///
    /// # Errors
    /// Returns an error if the channel is closed or full.
    pub async fn send(&self, data: T) -> Result<(), smol::channel::SendError<T>> {
        self.sender.send(data).await
    }

    /// Receive processed data
    ///
    /// # Returns
    /// Processed data or channel error
    ///
    /// # Errors
    /// Returns an error if the channel is closed or empty.
    pub async fn receive(&self) -> Result<T, smol::channel::RecvError> {
        let data = self.receiver.recv().await?;
        Ok((self.processor)(data))
    }

    /// Start processing in the background
    ///
    /// Spawns a background task that continuously processes data from the channel.
    pub fn start_background_processing(&self) {
        let receiver = self.receiver.clone();
        let processor = self.processor.clone();

        smol::spawn(async move {
            while let Ok(data) = receiver.recv().await {
                let processed = processor(data);
                // In a real implementation, you might send processed data to another channel
                // For now, we just process and discard (or log)
                drop(processed);
            }
        })
        .detach();
    }
}

/// Utility for creating buffered async readers
///
/// Provides buffered reading capabilities for async streams.
/// Useful for efficient reading of large data streams.
pub struct BufferedAsyncReader<R> {
    reader: R,
    buffer: Vec<u8>,
    buffer_size: usize,
    position: usize,
    limit: usize,
}

impl<R> BufferedAsyncReader<R>
where
    R: AsyncReadExt + Unpin,
{
    /// Create a new buffered async reader
    ///
    /// # Parameters
    /// - `reader`: The underlying async reader
    /// - `buffer_size`: Size of the read buffer
    ///
    /// # Returns
    /// New `BufferedAsyncReader` instance
    #[must_use]
    pub fn new(reader: R, buffer_size: usize) -> Self {
        Self {
            reader,
            buffer: vec![0u8; buffer_size],
            buffer_size,
            position: 0,
            limit: 0,
        }
    }

    /// Read data into the buffer
    ///
    /// # Returns
    /// Slice of available data or error
    ///
    /// # Errors
    /// Returns error if reading from the underlying reader fails
    pub async fn read_buffer(&mut self) -> Result<&[u8], std::io::Error> {
        if self.position >= self.limit {
            self.limit = self.reader.read(&mut self.buffer).await?;
            self.position = 0;

            if self.limit == 0 {
                return Ok(&self.buffer[0..0]);
            }
        }

        Ok(&self.buffer[self.position..self.limit])
    }

    /// Consume bytes from the buffer
    ///
    /// # Parameters
    /// - `amount`: Number of bytes to consume
    pub fn consume(&mut self, amount: usize) {
        self.position = (self.position + amount).min(self.limit);
    }

    /// Get the buffer size
    ///
    /// # Returns
    /// The size of the internal buffer
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}
