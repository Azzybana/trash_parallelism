//! I/O utility functions and data structures.
//!
//! This module provides basic file operations, compression utilities,
//! thread-safe counters, caching, and string interning for efficient I/O operations.
//!
//! ## Features
//!
//! - **Async File Operations**: Basic read/write operations using smol
//! - **Compression**: Brotli compression/decompression integration
//! - **Thread-Safe Utilities**: Atomic counters and LRU caching
//! - **Memory Efficiency**: String interning for reduced memory usage
//! - **Performance**: Optimized with ahash and `parking_lot`

// Standard library imports
use std::sync::Arc;

// External crate imports
use ahash::AHashMap;
use parking_lot::Mutex;
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
/// # Errors
/// Returns an I/O error if the file cannot be opened or read.
///
/// # Examples
/// ```rust,no_run
/// use trash_utilities::io::read_file_async;
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
/// # Errors
/// Returns an I/O error if the file cannot be created or written to.
///
/// # Examples
/// ```rust,no_run
/// use trash_utilities::io::write_file_async;
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
/// # Errors
/// Returns an I/O error if the source file cannot be read or the destination cannot be written to.
///
/// # Examples
/// ```rust,no_run
/// use trash_utilities::io::copy_file_async;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let bytes = copy_file_async("source.txt", "dest.txt").await?;
///     println!("Copied {} bytes", bytes);
///     Ok(())
/// }
/// ```
pub fn copy_file_async(from: &str, to: &str) -> Result<u64, std::io::Error> {
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
/// # Errors
/// Returns an I/O error if the file cannot be opened or read.
///
/// # Examples
/// ```rust,no_run
/// use trash_utilities::io::read_file_bytes_async;
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
/// # Errors
/// Returns an I/O error if the file cannot be created or written to.
///
/// # Examples
/// ```rust,no_run
/// use trash_utilities::io::write_file_bytes_async;
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
/// # Errors
/// Returns an I/O error if the directory cannot be created.
///
/// # Examples
/// ```rust,no_run
/// use trash_utilities::io::create_dir_async;
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
/// # Errors
/// Returns an I/O error if the directory cannot be opened or read.
///
/// # Examples
/// ```rust,no_run
/// use trash_utilities::io::read_dir_async;
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
    use futures_lite::StreamExt;

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

/// Compress data using Brotli
///
/// Leverages the common module's compression utilities for consistency.
///
/// # Parameters
/// - `data`: The data to compress.
/// - `level`: Compression level (1-11, where 11 is maximum compression).
///
/// # Returns
/// - `Ok(Vec<u8>)` containing compressed data.
/// - `Err(std::io::Error)` if compression fails.
///
/// # Errors
/// Returns an I/O error if compression fails.
pub fn compress_brotli(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    crate::common::crypto::compress_brotli(data, level)
}

/// Decompress Brotli data
///
/// Leverages the common module's decompression utilities.
///
/// # Parameters
/// - `data`: The compressed data to decompress.
///
/// # Returns
/// - `Ok(Vec<u8>)` containing decompressed data.
/// - `Err(std::io::Error)` if decompression fails.
///
/// # Errors
/// Returns an I/O error if decompression fails.
pub fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    crate::common::crypto::decompress_brotli(data)
}

/// Thread-safe counter with atomic operations
///
/// Uses `parking_lot` for efficient locking and arc-swap for potential future optimizations.
#[derive(Debug, Clone)]
pub struct AtomicCounter {
    count: Arc<Mutex<u64>>,
}

impl AtomicCounter {
    /// Create a new counter initialized to zero
    #[must_use]
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    /// Increment the counter and return the new value
    ///
    /// This operation is thread-safe and uses efficient locking.
    #[must_use]
    pub fn increment(&self) -> u64 {
        let mut count = self.count.lock();
        *count += 1;
        *count
    }

    /// Get the current value of the counter
    ///
    /// This operation is thread-safe.
    #[must_use]
    pub fn get(&self) -> u64 {
        *self.count.lock()
    }

    /// Reset the counter to zero
    ///
    /// This operation is thread-safe.
    pub fn reset(&self) {
        *self.count.lock() = 0;
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe LRU cache with configurable capacity
///
/// Uses ahash for fast hashing and `parking_lot` for efficient concurrent access.
/// Implements LRU eviction policy for memory efficiency.
pub struct LruCache<K, V> {
    map: Mutex<AHashMap<K, V>>,
    order: Mutex<Vec<K>>,
    capacity: usize,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    /// Create a new LRU cache with the specified capacity
    ///
    /// # Parameters
    /// - `capacity`: Maximum number of items to store in the cache.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            map: Mutex::new(AHashMap::new()),
            order: Mutex::new(Vec::new()),
            capacity,
        }
    }

    /// Get a value from the cache
    ///
    /// If the key exists, it moves to the front (most recently used).
    /// Returns `None` if the key is not in the cache.
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let map = self.map.lock();
        let mut order = self.order.lock();

        if let Some(value) = map.get(key) {
            // Move to front (most recently used)
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
    ///
    /// If the cache is at capacity, the least recently used item is evicted.
    /// If the key already exists, it updates the value and moves it to the front.
    pub fn insert(&self, key: K, value: V) {
        let mut map = self.map.lock();
        let mut order = self.order.lock();

        if map.contains_key(&key) {
            // Update existing - move to front
            if let Some(pos) = order.iter().position(|k| k == &key) {
                order.remove(pos);
            }
        } else if map.len() >= self.capacity {
            // Remove least recently used (oldest)
            if let Some(oldest) = order.first().cloned() {
                map.remove(&oldest);
                order.remove(0);
            }
        }

        map.insert(key.clone(), value);
        order.push(key);
    }

    /// Get the current number of items in the cache
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.lock().len()
    }

    /// Check if the cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.lock().is_empty()
    }

    /// Clear all items from the cache
    pub fn clear(&self) {
        let mut map = self.map.lock();
        let mut order = self.order.lock();
        map.clear();
        order.clear();
    }
}

/// Efficient string interning for memory optimization
///
/// Uses ahash for fast hashing and `parking_lot` for thread-safe access.
/// Stores strings as Arc<str> to enable efficient sharing and reference counting.
pub struct StringInterner {
    strings: Mutex<AHashMap<u64, Arc<str>>>,
}

impl StringInterner {
    /// Create a new string interner
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: Mutex::new(AHashMap::new()),
        }
    }

    /// Intern a string, returning a shared reference
    ///
    /// If the string has already been interned, returns the existing Arc<str>.
    /// Otherwise, creates a new interned string and returns it.
    ///
    /// # Parameters
    /// - `s`: The string to intern.
    ///
    /// # Returns
    /// An `Arc<str>` containing the interned string.
    pub fn intern(&self, s: &str) -> Arc<str> {
        use std::hash::Hasher;

        let mut hasher = ahash::AHasher::default();
        hasher.write(s.as_bytes());
        let key = hasher.finish();

        let mut strings = self.strings.lock();
        if let Some(interned) = strings.get(&key) {
            Arc::clone(interned)
        } else {
            let interned: Arc<str> = Arc::from(s);
            strings.insert(key, Arc::clone(&interned));
            interned
        }
    }

    /// Get the number of unique interned strings
    #[must_use]
    pub fn len(&self) -> usize {
        self.strings.lock().len()
    }

    /// Check if any strings are interned
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strings.lock().is_empty()
    }

    /// Clear all interned strings
    pub fn clear(&self) {
        self.strings.lock().clear();
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Global string interner instance for application-wide string sharing
///
/// Uses `once_cell` for lazy initialization and arc-swap for potential future
/// concurrent access optimizations.
static GLOBAL_INTERNER: std::sync::LazyLock<StringInterner> =
    std::sync::LazyLock::new(StringInterner::new);

/// Get a reference to the global string interner
///
/// This provides application-wide string interning for memory efficiency.
/// All calls to `global_interner()` return the same instance.
#[must_use]
pub fn global_interner() -> &'static StringInterner {
    &GLOBAL_INTERNER
}
