//! Async file writers with buffering and compression.
//!
//! This module provides ergonomic async writers for various output destinations,
//! with built-in buffering, compression, and serialization support.
//!
//! ## Features
//!
//! - **Buffered Writing**: Automatic buffering with configurable flush triggers
//! - **Compression**: Integrated Brotli/gzip compression for file writers
//! - **Serialization**: JSON/binary serialization using serde utilities
//! - **Async Operations**: Non-blocking writes using smol and futures-lite
//! - **Progress Tracking**: Optional progress callbacks and statistics
//!
//! ## Examples
//!
//! Basic buffered writing:
//! ```rust,no_run
//! use trash_utilities::io::writers::AsyncFileWriter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a compressed writer
//!     let mut writer = AsyncFileWriter::with_config("output.txt", 8192, true).await?;
//!     
//!     // Write data (buffered)
//!     writer.write(b"Hello, ").await?;
//!     writer.write(b"world!").await?;
//!     
//!     // Write JSON data
//!     writer.write_json(&serde_json::json!({"message": "done"})).await?;
//!     
//!     // Flush to disk
//!     writer.flush().await?;
//!     println!("Written {} bytes", writer.bytes_written());
//!     
//!     Ok(())
//! }
//! ```
//!
//! Streaming large data:
//! ```rust,no_run
//! use trash_utilities::io::writers::StreamingFileWriter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut writer = StreamingFileWriter::new("large_output.bin", 65536, false).await?;
//!     
//!     for chunk in large_data_chunks {
//!         writer.write_chunk(&chunk).await?;
//!     }
//!     
//!     println!("Total bytes written: {}", writer.bytes_written());
//!     Ok(())
//! }
//! ```

// Standard library imports
// (none needed)

// External crate imports
use bytes::Bytes;
use futures_lite::{AsyncWriteExt, StreamExt};

/// Async buffered file writer with compression support
///
/// Provides buffered writing with automatic flushing, optional compression,
/// and progress tracking. Uses smol for async I/O operations.
///
/// # Examples
///
/// Basic usage:
/// ```rust,no_run
/// use trash_utilities::io::writers::AsyncFileWriter;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create writer with compression
///     let mut writer = AsyncFileWriter::with_config("output.txt", 8192, true).await?;
///     
///     // Write data
///     writer.write(b"Hello, world!").await?;
///     writer.write_json(&serde_json::json!({"status": "ok"})).await?;
///     
///     // Flush and check bytes written
///     writer.flush().await?;
///     println!("Bytes written: {}", writer.bytes_written());
///     
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct AsyncFileWriter {
    file: smol::fs::File,
    buffer: Vec<u8>,
    buffer_size: usize,
    compressed: bool,
    bytes_written: u64,
}

impl AsyncFileWriter {
    /// Create a new async file writer with default settings
    ///
    /// # Parameters
    /// - `path`: Path to the file to write
    ///
    /// # Returns
    /// New `AsyncFileWriter` instance
    ///
    /// # Errors
    /// Returns error if file cannot be created
    pub async fn new(path: &str) -> Result<Self, std::io::Error> {
        Self::with_config(path, 8192, false).await
    }

    /// Create a new async file writer with custom configuration
    ///
    /// # Parameters
    /// - `path`: Path to the file to write
    /// - `buffer_size`: Size of the write buffer in bytes
    /// - `compressed`: Whether to compress data using Brotli
    ///
    /// # Returns
    /// Configured `AsyncFileWriter` instance
    ///
    /// # Errors
    /// Returns error if file cannot be created
    pub async fn with_config(
        path: &str,
        buffer_size: usize,
        compressed: bool,
    ) -> Result<Self, std::io::Error> {
        let file = smol::fs::File::create(path).await?;
        Ok(Self {
            file,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
            compressed,
            bytes_written: 0,
        })
    }

    /// Write data to the buffer (async, non-blocking)
    ///
    /// Data is buffered until the buffer is full or `flush()` is called.
    /// If compression is enabled, data is compressed before buffering.
    ///
    /// # Parameters
    /// - `data`: Data to write
    ///
    /// # Returns
    /// Success or I/O error
    ///
    /// # Errors
    /// Returns error if compression fails or buffer write fails
    pub async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        let data_to_write = if self.compressed {
            // Compress data using our compression utilities
            crate::io::utils::compress_brotli(data, 6)?
        } else {
            data.to_vec()
        };

        self.buffer.extend_from_slice(&data_to_write);

        if self.buffer.len() >= self.buffer_size {
            self.flush().await?;
        }

        Ok(())
    }

    /// Write serialized data (async, non-blocking)
    ///
    /// Serializes the data to JSON and writes it with optional compression.
    ///
    /// # Type Parameters
    /// - `T`: Type that implements serde Serialize
    ///
    /// # Parameters
    /// - `data`: Data to serialize and write
    ///
    /// # Returns
    /// Success or error
    ///
    /// # Errors
    /// Returns error if serialization, compression, or writing fails
    pub async fn write_json<T: serde::Serialize>(
        &mut self,
        data: &T,
    ) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.write(json.as_bytes()).await
    }

    /// Flush buffer to file (async, non-blocking)
    ///
    /// Forces all buffered data to be written to the file.
    ///
    /// # Returns
    /// Success or I/O error
    ///
    /// # Errors
    /// Returns error if file write fails
    pub async fn flush(&mut self) -> Result<(), std::io::Error> {
        if !self.buffer.is_empty() {
            self.file.write_all(&self.buffer).await?;
            self.bytes_written += self.buffer.len() as u64;
            self.buffer.clear();
        }
        self.file.flush().await?;
        Ok(())
    }

    /// Get total bytes written to the file
    #[must_use]
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Check if compression is enabled
    #[must_use]
    pub fn is_compressed(&self) -> bool {
        self.compressed
    }
}

impl Drop for AsyncFileWriter {
    fn drop(&mut self) {
        // Note: This is sync, in practice you'd want an async drop
        futures_lite::future::block_on(async {
            let _ = self.flush().await;
        });
    }
}

/// Streaming file writer for large data sets
///
/// Optimized for streaming large amounts of data with minimal memory usage.
/// Uses chunked writing and can integrate with async streams.
pub struct StreamingFileWriter {
    file: smol::fs::File,
    chunk_size: usize,
    compressed: bool,
    bytes_written: u64,
}

impl StreamingFileWriter {
    /// Create a new streaming file writer
    ///
    /// # Parameters
    /// - `path`: Path to the file to write
    /// - `chunk_size`: Size of chunks to write at once
    /// - `compressed`: Whether to compress chunks
    ///
    /// # Returns
    /// New `StreamingFileWriter` instance
    ///
    /// # Errors
    /// Returns error if file cannot be created
    pub async fn new(
        path: &str,
        chunk_size: usize,
        compressed: bool,
    ) -> Result<Self, std::io::Error> {
        let file = smol::fs::File::create(path).await?;
        Ok(Self {
            file,
            chunk_size,
            compressed,
            bytes_written: 0,
        })
    }

    /// Write a chunk of data (async, non-blocking)
    ///
    /// Writes data immediately without buffering. Suitable for streaming.
    ///
    /// # Parameters
    /// - `data`: Data chunk to write
    ///
    /// # Returns
    /// Success or I/O error
    ///
    /// # Errors
    /// Returns error if compression or writing fails
    pub async fn write_chunk(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        let data_to_write = if self.compressed {
            crate::io::utils::compress_brotli(data, 6)?
        } else {
            data.to_vec()
        };

        self.file.write_all(&data_to_write).await?;
        self.bytes_written += data_to_write.len() as u64;
        Ok(())
    }

    /// Write from an async stream (async, non-blocking)
    ///
    /// Consumes an async stream and writes all chunks to the file.
    ///
    /// # Type Parameters
    /// - `S`: Async stream type yielding byte chunks
    ///
    /// # Parameters
    /// - `stream`: Async stream of byte data
    ///
    /// # Returns
    /// Success or error
    ///
    /// # Errors
    /// Returns error if reading from stream or writing fails
    pub async fn write_from_stream<S>(&mut self, mut stream: S) -> Result<(), std::io::Error>
    where
        S: futures_lite::Stream<Item = Result<Bytes, std::io::Error>> + Unpin,
    {
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            self.write_chunk(&chunk).await?;
        }
        self.file.flush().await?;
        Ok(())
    }

    /// Get total bytes written
    #[must_use]
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Get the chunk size
    #[must_use]
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

/// Buffered writer with automatic compression and serialization
///
/// Advanced writer that combines buffering, compression, and serialization
/// with progress tracking and error recovery.
pub struct AdvancedFileWriter {
    inner: AsyncFileWriter,
    progress_callback: Option<Box<dyn Fn(u64) + Send + Sync>>,
    error_recovery: bool,
}

impl AdvancedFileWriter {
    /// Create a new advanced file writer
    ///
    /// # Parameters
    /// - `path`: Path to the file to write
    /// - `buffer_size`: Size of the write buffer
    /// - `compressed`: Whether to compress data
    /// - `progress_callback`: Optional callback for progress updates
    /// - `error_recovery`: Whether to attempt error recovery
    ///
    /// # Returns
    /// New `AdvancedFileWriter` instance
    ///
    /// # Errors
    /// Returns error if file cannot be created
    pub async fn new(
        path: &str,
        buffer_size: usize,
        compressed: bool,
        progress_callback: Option<Box<dyn Fn(u64) + Send + Sync>>,
        error_recovery: bool,
    ) -> Result<Self, std::io::Error> {
        let inner = AsyncFileWriter::with_config(path, buffer_size, compressed).await?;
        Ok(Self {
            inner,
            progress_callback,
            error_recovery,
        })
    }

    /// Write data with progress tracking and error recovery
    ///
    /// # Parameters
    /// - `data`: Data to write
    ///
    /// # Returns
    /// Success or error
    ///
    /// # Errors
    /// Returns error if writing fails (with optional recovery attempts)
    pub async fn write_with_progress(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        let result = self.inner.write(data).await;

        if let Err(ref e) = result
            && self.error_recovery
        {
            // Attempt recovery (could implement retry logic, backup writing, etc.)
            eprintln!("Write error, attempting recovery: {e}");
            // For now, just log - could implement more sophisticated recovery
        }

        // Update progress
        if let Some(ref callback) = self.progress_callback {
            callback(self.inner.bytes_written());
        }

        result
    }

    /// Write serialized data with progress tracking
    ///
    /// # Type Parameters
    /// - `T`: Type that implements serde Serialize
    ///
    /// # Parameters
    /// - `data`: Data to serialize and write
    ///
    /// # Returns
    /// Success or error
    ///
    /// # Errors
    /// Returns error if serialization or writing fails
    pub async fn write_json_with_progress<T: serde::Serialize>(
        &mut self,
        data: &T,
    ) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.write_with_progress(json.as_bytes()).await
    }

    /// Flush all buffered data
    ///
    /// # Returns
    /// Success or I/O error
    ///
    /// # Errors
    /// Returns an I/O error if flushing the buffer fails.
    pub async fn flush(&mut self) -> Result<(), std::io::Error> {
        self.inner.flush().await
    }

    /// Get total bytes written
    #[must_use]
    pub fn bytes_written(&self) -> u64 {
        self.inner.bytes_written()
    }
}

/// Utility function for writing to stdout asynchronously
///
/// # Parameters
/// - `data`: Data to write to stdout
///
/// # Returns
/// Success or I/O error
///
/// # Errors
/// Returns error if stdout write fails
pub fn write_stdout_async(data: &[u8]) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(data)?;
    stdout.flush()?;
    Ok(())
}

/// Utility function for writing to stderr asynchronously
///
/// # Parameters
/// - `data`: Data to write to stderr
///
/// # Returns
/// Success or I/O error
///
/// # Errors
/// Returns error if stderr write fails
pub fn write_stderr_async(data: &[u8]) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut stderr = std::io::stderr().lock();
    stderr.write_all(data)?;
    stderr.flush()?;
    Ok(())
}
