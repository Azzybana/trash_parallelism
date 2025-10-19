/// Core character and string processing utilities.
///
/// This module provides high-performance string operations including
/// searching, hashing, encoding, splitting, deduplication, interning,
/// and efficient string building using optimized algorithms and data structures.
// Standard library imports
use std::{hash::Hasher, sync::Arc};

// External crate imports
use ahash::{AHashMap, AHasher};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use memchr::memchr;
use parking_lot::Mutex;

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