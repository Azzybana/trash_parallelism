/// Common utilities shared across multiple modules.
///
/// This module contains frequently used utility functions and types
/// that are duplicated across the codebase. By centralizing them here,
/// we reduce code duplication and improve maintainability.
// Standard library imports
use std::{
    io::{Read, Write},
    sync::Arc,
};

// External crate imports
use ahash::AHashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use brotli::{CompressorWriter, Decompressor};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json;
use tracing::debug;

/// Creates a new empty `AHashMap` with the `AHash` hashing algorithm.
///
/// `AHash` is a high-performance hashing algorithm, often faster than the standard
/// library's hasher for hash maps.
///
/// # Type Parameters
/// - `K`: The key type, must implement `Hash` and `Eq`.
/// - `V`: The value type.
///
/// # Returns
/// An empty `AHashMap<K, V>`.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::common::create_ahash_map;
/// use ahash::AHashMap;
///
/// let mut map: AHashMap<String, i32> = create_ahash_map();
/// map.insert("key".to_string(), 42);
/// ```
#[must_use]
pub fn create_ahash_map<K, V>() -> AHashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    AHashMap::new()
}

/// Wraps a byte slice into a `Bytes` object for efficient sharing.
///
/// This function creates a copy of the data in `Bytes`, which is useful for
/// zero-copy operations in async contexts.
///
/// # Parameters
/// - `data`: The byte slice to wrap.
///
/// # Returns
/// A `Bytes` object containing a copy of the input data.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::common::wrap_bytes;
/// use bytes::Bytes;
///
/// let data = b"Hello, world!";
/// let bytes = wrap_bytes(data);
/// assert_eq!(bytes.as_ref(), data);
/// ```
#[must_use]
pub fn wrap_bytes(data: &[u8]) -> Bytes {
    Bytes::copy_from_slice(data)
}

/// Encodes binary data to a Base64 string.
///
/// This function uses the standard Base64 encoding without padding.
///
/// # Parameters
/// - `data`: The byte slice to encode.
///
/// # Returns
/// A `String` containing the Base64-encoded data.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::common::encode_base64;
///
/// let data = b"Hello, world!";
/// let encoded = encode_base64(data);
/// println!("Encoded: {}", encoded);
/// ```
#[must_use]
pub fn encode_base64(data: &[u8]) -> String {
    STANDARD.encode(data)
}

/// Decodes a Base64 string to binary data.
///
/// This function decodes standard Base64 without padding.
///
/// # Parameters
/// - `data`: The Base64-encoded string to decode.
///
/// # Returns
/// - `Ok(Vec<u8>)` containing the decoded bytes if successful.
/// - `Err(base64::DecodeError)` if decoding fails (e.g., invalid Base64).
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::common::{encode_base64, decode_base64};
///
/// let data = b"Hello, world!";
/// let encoded = encode_base64(data);
/// let decoded = decode_base64(&encoded).unwrap();
/// assert_eq!(decoded, data);
/// ```
pub fn decode_base64(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    STANDARD.decode(data)
}

/// Compress data using Brotli
pub fn compress_brotli(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    let mut output = Vec::new();
    {
        let mut compressor = CompressorWriter::new(&mut output, 4096, level, level);
        compressor.write_all(data)?;
        compressor.flush()?;
    }
    debug!("Compressed {} bytes to {} bytes", data.len(), output.len());
    Ok(output)
}

/// Decompress Brotli data
pub fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decompressor = Decompressor::new(data, 4096);
    let mut output = Vec::new();
    decompressor.read_to_end(&mut output)?;
    debug!("Decompressed to {} bytes", output.len());
    Ok(output)
}

/// Compute fast hash of data using ahash
#[must_use]
pub fn fast_hash(data: &[u8]) -> u64 {
    use ahash::AHasher;
    use std::hash::Hasher;
    let mut hasher = AHasher::default();
    hasher.write(data);
    hasher.finish()
}

/// Compute fast hash of data with key using ahash
#[must_use]
pub fn keyed_hash(key: &[u8], data: &[u8]) -> u64 {
    use ahash::AHasher;
    use std::hash::Hasher;
    let mut hasher = AHasher::default();
    hasher.write(key);
    hasher.write(data);
    hasher.finish()
}

/// Verify keyed hash
#[must_use]
pub fn verify_keyed_hash(key: &[u8], data: &[u8], expected: u64) -> bool {
    keyed_hash(key, data) == expected
}

/// JSON utilities with error handling
pub fn parse_json_value<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn to_json_value<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn pretty_json_value<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

/// Validate JSON structure
#[must_use]
pub fn validate_json(json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(json).is_ok()
}

/// Merge two JSON objects
pub fn merge_json(a: &str, b: &str) -> Result<String, serde_json::Error> {
    let mut a_val: serde_json::Value = serde_json::from_str(a)?;
    let b_val: serde_json::Value = serde_json::from_str(b)?;

    if let (Some(a_obj), Some(b_obj)) = (a_val.as_object_mut(), b_val.as_object()) {
        for (k, v) in b_obj {
            a_obj.insert(k.clone(), v.clone());
        }
    }

    serde_json::to_string(&a_val)
}

/// Extract values from JSON by path
pub fn extract_json_path(
    json: &str,
    path: &str,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let mut current = &value;

    for segment in path.split('.') {
        match current {
            serde_json::Value::Object(obj) => {
                current = obj.get(segment).ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Path segment '{segment}' not found"),
                    )
                })?;
            }
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Cannot traverse into non-object",
                )));
            }
        }
    }

    Ok(Some(current.clone()))
}

/// Gets the current UTC time.
///
/// This function returns the current time in UTC using `chrono::Utc::now()`.
///
/// # Returns
/// A `DateTime<Utc>` representing the current UTC time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::common::current_utc_time;
/// use chrono::{DateTime, Utc};
///
/// let now: DateTime<Utc> = current_utc_time();
/// println!("Current time: {}", now);
/// ```
#[must_use]
pub fn current_utc_time() -> DateTime<Utc> {
    Utc::now()
}

/// Formats a `DateTime<Utc>` to an ISO 8601 string.
///
/// This function uses RFC 3339 format, which is a profile of ISO 8601.
///
/// # Parameters
/// - `dt`: The `DateTime<Utc>` to format.
///
/// # Returns
/// A `String` containing the formatted date and time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::common::{current_utc_time, format_datetime};
/// use chrono::DateTime;
///
/// let now = current_utc_time();
/// let formatted = format_datetime(&now);
/// println!("Formatted time: {}", formatted);
/// ```
#[must_use]
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Parses a date/time string in RFC 3339 format.
///
/// # Parameters
/// - `s`: The string to parse.
///
/// # Returns
/// - `Ok(DateTime<Utc>)` if parsing succeeds.
/// - `Err(chrono::ParseError)` if parsing fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::common::parse_datetime;
///
/// let dt = parse_datetime("2023-01-01T12:00:00Z").unwrap();
/// println!("Parsed datetime: {}", dt);
/// ```
pub fn parse_datetime(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
}

/// Parses a date string in YYYY-MM-DD format.
///
/// # Parameters
/// - `s`: The string to parse.
///
/// # Returns
/// - `Ok(NaiveDate)` if parsing succeeds.
/// - `Err(chrono::ParseError)` if parsing fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::common::parse_date;
/// use chrono::NaiveDate;
///
/// let date = parse_date("2023-01-01").unwrap();
/// println!("Parsed date: {}", date);
/// ```
pub fn parse_date(s: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
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
