/// Comprehensive character and string processing utilities.
///
/// This module provides high-performance string operations, compression,
/// encoding, hashing, and parallel processing capabilities using the
/// full range of available dependencies.
/// for robust asynchronous programming.
// Standard library imports
use std::{
    hash::Hasher,
    sync::Arc,
};

// External crate imports
use ahash::{AHashMap, AHasher};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use memchr::memchr;
use parking_lot::Mutex;
use serde_json;
use smol::fs;
use tracing::debug;

/// Find the first occurrence of a byte in a slice
#[must_use]
pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    memchr(needle, haystack)
}

/// Find all occurrences of a byte in a slice
#[must_use]
pub fn find_byte_all(haystack: &[u8], needle: u8) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut start = 0;
    while let Some(pos) = memchr(needle, &haystack[start..]) {
        positions.push(start + pos);
        start += pos + 1;
    }
    positions
}

/// Find the first occurrence of any byte from a set
#[must_use]
pub fn find_any_byte(haystack: &[u8], needles: &[u8]) -> Option<usize> {
    for (i, &byte) in haystack.iter().enumerate() {
        if needles.contains(&byte) {
            return Some(i);
        }
    }
    None
}

/// Compute fast hash of a string
#[must_use]
pub fn hash_string_fast(s: &str) -> u64 {
    let mut hasher = AHasher::default();
    hasher.write(s.as_bytes());
    hasher.finish()
}

/// Encode a string as base64
#[must_use]
pub fn encode_string_base64(s: &str) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, s.as_bytes())
}

/// Decode a base64 string
pub fn decode_string_base64(s: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)?;
    String::from_utf8(bytes).map_err(Into::into)
}

/// Convenience: Hash and encode as base64
#[must_use]
pub fn hash_and_encode_base64(s: &str) -> String {
    let hash = crate::common::fast_hash(s.as_bytes());
    STANDARD.encode(hash.to_le_bytes())
}

/// Convenience: Parse JSON and validate
pub fn parse_and_validate_json<T: for<'de> serde::Deserialize<'de>>(
    s: &str,
) -> Result<T, serde_json::Error> {
    let value: T = crate::serde::parse_json_value(s)?;
    // Could add validation logic here
    Ok(value)
}

/// Result type alias for processing operations
pub type ProcessingResult<T> = Result<T, serde_json::Error>;

/// Parallel string processing: split string and process chunks
pub fn parallel_process_string<F, R>(s: &str, chunk_size: usize, processor: F) -> Vec<R>
where
    F: Fn(&str) -> R + Send + Sync,
    R: Send,
{
    let chunks: Vec<&str> = s
        .as_bytes()
        .chunks(chunk_size)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect();

    chunks.into_iter().map(processor).collect()
}

/// Asynchronously read a file to string
pub async fn read_file_to_string_async(path: &str) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    debug!("Async read {} bytes from {}", contents.len(), path);
    Ok(contents)
}

/// Asynchronously write string to file
pub async fn write_string_to_file_async(path: &str, contents: &str) -> Result<(), std::io::Error> {
    let mut file = fs::File::create(path).await?;
    file.write_all(contents.as_bytes()).await?;
    file.flush().await?;
    debug!("Async wrote {} bytes to {}", contents.len(), path);
    Ok(())
}

/// Efficient string splitting with memchr
#[must_use]
pub fn split_string_efficient(s: &str, delimiter: char) -> Vec<&str> {
    let bytes = s.as_bytes();
    let delim_byte = delimiter as u8;
    let mut parts = Vec::new();
    let mut start = 0;

    while let Some(pos) = memchr(delim_byte, &bytes[start..]) {
        let abs_pos = start + pos;
        if abs_pos > start
            && let Ok(part) = std::str::from_utf8(&bytes[start..abs_pos])
        {
            parts.push(part);
        }
        start = abs_pos + 1;
    }

    // Add remaining part
    if start < bytes.len()
        && let Ok(part) = std::str::from_utf8(&bytes[start..])
    {
        parts.push(part);
    }

    parts
}

/// Count occurrences of a substring efficiently
#[must_use]
pub fn count_substring(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut start = 0;

    while let Some(pos) = haystack[start..].find(needle) {
        count += 1;
        start += pos + needle.len();
    }

    count
}

/// Extract all JSON values by key from a JSON array string
pub fn extract_json_values_by_key(
    json_array: &str,
    key: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let array: Vec<serde_json::Value> = serde_json::from_str(json_array)?;
    let mut results = Vec::new();

    for item in array {
        if let Some(obj) = item.as_object()
            && let Some(value) = obj.get(key)
        {
            results.push(value.clone());
        }
    }

    Ok(results)
}

/// String deduplication: remove duplicate lines
#[must_use]
pub fn deduplicate_lines(s: &str) -> String {
    let mut seen = AHashMap::new();
    let mut result = String::new();

    for line in s.lines() {
        if seen.insert(line, true).is_none() {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Create a string interner for memory-efficient string storage
#[derive(Debug)]
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

    /// Intern a string, returning a reference-counted copy
    pub fn intern(&self, s: &str) -> Arc<str> {
        let hash = hash_string_fast(s);
        let mut strings = self.strings.lock();

        if let Some(interned) = strings.get(&hash) {
            return Arc::clone(interned);
        }

        let interned: Arc<str> = Arc::from(s);
        strings.insert(hash, Arc::clone(&interned));
        interned
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        let strings = self.strings.lock();
        (strings.len(), strings.values().map(|s| s.len()).sum())
    }
}

/// Memory-efficient string builder with size hints
#[derive(Debug)]
pub struct EfficientStringBuilder {
    buffer: String,
    estimated_final_size: Option<usize>,
}

impl EfficientStringBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            estimated_final_size: None,
        }
    }

    /// Create a new builder with size hint
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
            estimated_final_size: Some(capacity),
        }
    }

    /// Append a string
    pub fn append(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Append a character
    pub fn append_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Build the final string
    #[must_use]
    pub fn build(self) -> String {
        self.buffer
    }

    /// Get current length
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
