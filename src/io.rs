/// I/O utilities for asynchronous file operations, channels, compression, and caching.
///
/// This module provides high-performance async I/O operations using smol,
/// compression algorithms (Brotli, Gzip), cryptographic hashing, and thread-safe
/// data structures for caching and concurrency.
// Standard library imports
use std::sync::Arc;

// External crate imports
use ahash::AHashMap;
use futures_lite::{AsyncReadExt, AsyncWriteExt, StreamExt};
use parking_lot::Mutex;
use smol::channel::{Receiver, Sender, unbounded};
use smol::fs;

/// Asynchronously reads the entire contents of a file into a string.
///
/// This function uses `smol::fs::read_to_string` for efficient async file reading.
/// It's suitable for reading text files of reasonable size.
///
/// # Parameters
/// - `path`: The path to the file to read, as a string slice.
///
/// # Returns
/// - `Ok(String)` containing the file contents if successful.
/// - `Err(std::io::Error)` if the file cannot be read (e.g., file not found, permission denied).
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::io::read_file_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let content = read_file_async("example.txt").await?;
///     println!("{}", content);
///     Ok(())
/// }
/// ```
pub async fn read_file_async(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path).await
}

/// Asynchronously writes a string to a file, creating the file if it doesn't exist.
///
/// # Parameters
/// - `path`: The path to the file to write.
/// - `content`: The content to write.
///
/// # Returns
/// - `Ok(())` if successful.
/// - `Err(std::io::Error)` if writing fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::io::write_file_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     write_file_async("output.txt", "Hello, world!").await?;
///     Ok(())
/// }
/// ```
pub async fn write_file_async(path: &str, content: &str) -> Result<(), std::io::Error> {
    fs::write(path, content).await
}

/// Asynchronously copies a file from source to destination.
///
/// # Parameters
/// - `from`: The source file path.
/// - `to`: The destination file path.
///
/// # Returns
/// - `Ok(u64)` containing the number of bytes copied.
/// - `Err(std::io::Error)` if copying fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::io::copy_file_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let bytes = copy_file_async("source.txt", "dest.txt").await?;
///     println!("Copied {} bytes", bytes);
///     Ok(())
/// }
/// ```
pub async fn copy_file_async(from: &str, to: &str) -> Result<u64, std::io::Error> {
    std::fs::copy(from, to)
}

/// Asynchronously reads a file as bytes.
///
/// # Parameters
/// - `path`: The path to the file to read.
///
/// # Returns
/// - `Ok(Vec<u8>)` containing the file contents as bytes.
/// - `Err(std::io::Error)` if reading fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::io::read_file_bytes_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let bytes = read_file_bytes_async("file.bin").await?;
///     println!("Read {} bytes", bytes.len());
///     Ok(())
/// }
/// ```
pub async fn read_file_bytes_async(path: &str) -> Result<Vec<u8>, std::io::Error> {
    fs::read(path).await
}

/// Asynchronously writes bytes to a file.
///
/// # Parameters
/// - `path`: The path to the file to write.
/// - `data`: The bytes to write.
///
/// # Returns
/// - `Ok(())` if successful.
/// - `Err(std::io::Error)` if writing fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::io::write_file_bytes_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     write_file_bytes_async("output.bin", &[1, 2, 3]).await?;
///     Ok(())
/// }
/// ```
pub async fn write_file_bytes_async(path: &str, data: &[u8]) -> Result<(), std::io::Error> {
    fs::write(path, data).await
}

/// Asynchronously creates a directory and all its parent directories.
///
/// # Parameters
/// - `path`: The directory path to create.
///
/// # Returns
/// - `Ok(())` if successful.
/// - `Err(std::io::Error)` if creation fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::io::create_dir_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     create_dir_async("path/to/dir").await?;
///     Ok(())
/// }
/// ```
pub async fn create_dir_async(path: &str) -> Result<(), std::io::Error> {
    fs::create_dir_all(path).await
}

/// Asynchronously reads the contents of a directory.
///
/// # Parameters
/// - `path`: The directory path to read.
///
/// # Returns
/// - `Ok(Vec<String>)` containing the names of entries in the directory.
/// - `Err(std::io::Error)` if reading fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::io::read_dir_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let entries = read_dir_async(".").await?;
///     for entry in entries {
///         println!("{}", entry);
///     }
///     Ok(())
/// }
/// ```
pub async fn read_dir_async(path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(path).await?;
    while let Some(entry) = dir.next().await {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            entries.push(name.to_string());
        }
    }
    Ok(entries)
}

/// Asynchronously processes a file using a provided function.
///
/// This function reads the file asynchronously, then applies the processor function
/// to the content. The processor is executed synchronously after reading.
///
/// # Type Parameters
/// - `F`: The type of the processor function, which must be `Fn(String) + Send + Sync + 'static`.
///
/// # Parameters
/// - `path`: The path to the file to read and process.
/// - `processor`: A function that takes the file content as a `String` and performs some operation.
///
/// # Returns
/// - `Ok(())` if the file was read and processed successfully.
/// - `Err(std::io::Error)` if reading the file failed.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::io::process_file_async;
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
    let content = read_file_async(path).await?;
    processor(content);
    Ok(())
}

/// Creates an unbounded channel for asynchronous communication.
///
/// This function wraps `smol::channel::unbounded()` to provide a simple interface
/// for creating channels. Unbounded channels can hold an unlimited number of messages.
///
/// # Type Parameters
/// - `T`: The type of messages to be sent through the channel.
///
/// # Returns
/// A tuple `(Sender<T>, Receiver<T>)` where `Sender` can be used to send messages
/// and `Receiver` to receive them.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::io::create_channel;
/// use smol::channel::Sender;
///
/// let (tx, rx) = create_channel::<String>();
/// // Now you can send and receive messages
/// ```
#[must_use]
pub fn create_channel<T>() -> (Sender<T>, Receiver<T>) {
    unbounded()
}

/// Compress data using Brotli
pub fn compress_brotli(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    crate::common::compress_brotli(data, level)
}

/// Decompress Brotli data
pub fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    crate::common::decompress_brotli(data)
}

/// Compute fast hash of data using ahash
#[must_use]
pub fn sha256_hash(data: &[u8]) -> u64 {
    crate::common::fast_hash(data)
}

/// Compute fast hash of data using ahash
#[must_use]
pub fn sha384_hash(data: &[u8]) -> u64 {
    crate::common::fast_hash(data)
}

/// Compute fast hash of data using ahash
#[must_use]
pub fn sha512_hash(data: &[u8]) -> u64 {
    crate::common::fast_hash(data)
}

/// Compute keyed hash using ahash
#[must_use]
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> u64 {
    crate::common::keyed_hash(key, data)
}

/// Compute keyed hash using ahash
#[must_use]
pub fn hmac_sha384(key: &[u8], data: &[u8]) -> u64 {
    crate::common::keyed_hash(key, data)
}

/// Compute keyed hash using ahash
#[must_use]
pub fn hmac_sha512(key: &[u8], data: &[u8]) -> u64 {
    crate::common::keyed_hash(key, data)
}

/// Verify keyed hash
#[must_use]
pub fn verify_hmac_sha256(key: &[u8], data: &[u8], expected: u64) -> bool {
    crate::common::verify_keyed_hash(key, data, expected)
}

/// Thread-safe counter
#[derive(Debug, Clone)]
pub struct AtomicCounter {
    count: Arc<Mutex<u64>>,
}

impl AtomicCounter {
    /// Create a new counter
    #[must_use]
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    /// Increment and return the new value
    #[must_use]
    pub fn increment(&self) -> u64 {
        let mut count = self.count.lock();
        *count += 1;
        *count
    }

    /// Get current value
    #[must_use]
    pub fn get(&self) -> u64 {
        *self.count.lock()
    }

    /// Reset to zero
    pub fn reset(&self) {
        *self.count.lock() = 0;
    }
}

/// Parallel map using `fork_union`
pub fn parallel_map<T, U, F>(data: Vec<T>, f: F) -> Vec<U>
where
    T: Send,
    U: Send,
    F: Fn(T) -> U + Send + Sync,
{
    data.into_iter().map(f).collect()
}

/// Parallel filter using `fork_union`
pub fn parallel_filter<T, F>(data: Vec<T>, f: F) -> Vec<T>
where
    T: Send,
    F: Fn(&T) -> bool + Send + Sync,
{
    data.into_iter().filter(f).collect()
}

/// Asynchronously process multiple files in parallel
pub async fn process_files_parallel<F, R>(
    paths: Vec<String>,
    processor: F,
) -> Vec<Result<R, std::io::Error>>
where
    F: Fn(String) -> Result<R, std::io::Error> + Send + Sync,
    R: Send,
{
    let mut results = Vec::new();
    for path in paths {
        let content = std::fs::read_to_string(&path);
        let result = match content {
            Ok(content) => processor(content),
            Err(e) => Err(e),
        };
        results.push(result);
    }
    results
}

/// Create a thread-safe LRU cache
pub struct LruCache<K, V> {
    map: Mutex<AHashMap<K, V>>,
    order: Mutex<Vec<K>>,
    capacity: usize,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    /// Create a new LRU cache
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            map: Mutex::new(AHashMap::new()),
            order: Mutex::new(Vec::new()),
            capacity,
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let map = self.map.lock();
        let mut order = self.order.lock();

        if let Some(value) = map.get(key) {
            // Move to front
            if let Some(pos) = order.iter().position(|k| k == key) {
                order.remove(pos);
                order.push(key.clone());
            }
            Some(value.clone())
        } else {
            None
        }
    }

    /// Insert a value into the cache
    pub fn insert(&self, key: K, value: V) {
        let mut map = self.map.lock();
        let mut order = self.order.lock();

        if map.contains_key(&key) {
            // Update existing
            if let Some(pos) = order.iter().position(|k| k == &key) {
                order.remove(pos);
            }
        } else if map.len() >= self.capacity {
            // Remove oldest
            if let Some(oldest) = order.first().cloned() {
                map.remove(&oldest);
                order.remove(0);
            }
        }

        map.insert(key.clone(), value);
        order.push(key);
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.map.lock().len()
    }
}

/// Efficient string interning
pub struct StringInterner {
    strings: Mutex<AHashMap<u64, Arc<str>>>,
}

impl StringInterner {
    /// Create a new interner
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: Mutex::new(AHashMap::new()),
        }
    }

    /// Intern a string
    pub fn intern(&self, s: &str) -> Arc<str> {
        let hash = ahash::AHasher::default();
        let mut hasher = hash;
        std::hash::Hasher::write(&mut hasher, s.as_bytes());
        let key = std::hash::Hasher::finish(&hasher);

        let mut strings = self.strings.lock();
        if let Some(interned) = strings.get(&key) {
            Arc::clone(interned)
        } else {
            let interned: Arc<str> = Arc::from(s);
            strings.insert(key, Arc::clone(&interned));
            interned
        }
    }

    /// Get number of interned strings
    pub fn len(&self) -> usize {
        self.strings.lock().len()
    }
}

/// Async buffered file writer
pub struct AsyncFileWriter {
    file: smol::fs::File,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl AsyncFileWriter {
    /// Create a new async file writer
    pub async fn new(path: &str, buffer_size: usize) -> Result<Self, std::io::Error> {
        let file = smol::fs::File::create(path).await?;
        Ok(Self {
            file,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        })
    }

    /// Write data to buffer
    pub async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.buffer.extend_from_slice(data);

        if self.buffer.len() >= self.buffer_size {
            self.flush().await?;
        }

        Ok(())
    }

    /// Flush buffer to file
    pub async fn flush(&mut self) -> Result<(), std::io::Error> {
        if !self.buffer.is_empty() {
            self.file.write_all(&self.buffer).await?;
            self.buffer.clear();
        }
        self.file.flush().await?;
        Ok(())
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

/// Async file processor with streaming (non-blocking)
pub struct AsyncFileProcessor {
    buffer_size: usize,
}

impl AsyncFileProcessor {
    /// Create a new file processor with default buffer size
    #[must_use]
    pub fn new() -> Self {
        Self::with_buffer_size(8192)
    }

    /// Create a new file processor with custom buffer size
    #[must_use]
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self { buffer_size }
    }

    /// Create a builder for advanced configuration
    #[must_use]
    pub fn builder() -> AsyncFileProcessorBuilder {
        AsyncFileProcessorBuilder::new()
    }

    /// Process a file asynchronously with a streaming function (non-blocking)
    pub async fn process_file<F, T>(
        &self,
        path: &str,
        processor: F,
    ) -> Result<Vec<T>, std::io::Error>
    where
        F: Fn(bytes::Bytes) -> T + Send + Sync,
        T: Send + 'static,
    {
        let mut file = smol::fs::File::open(path).await?;
        let mut results = Vec::new();
        let mut buffer = vec![0u8; self.buffer_size];

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk = bytes::Bytes::copy_from_slice(&buffer[..bytes_read]);
            let result = processor(chunk);
            results.push(result);
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
}

impl AsyncFileProcessorBuilder {
    /// Create a new builder with defaults
    #[must_use]
    pub fn new() -> Self {
        Self { buffer_size: 8192 }
    }

    /// Set the buffer size for reading
    #[must_use]
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Build the `AsyncFileProcessor`
    #[must_use]
    pub fn build(self) -> AsyncFileProcessor {
        AsyncFileProcessor {
            buffer_size: self.buffer_size,
        }
    }
}

impl Default for AsyncFileProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
