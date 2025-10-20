/// Core character and string processing utilities.
///
/// This module provides high-performance string operations including
/// searching, hashing, encoding, splitting, deduplication, interning,
/// and efficient string building using optimized algorithms and data structures.
///
/// # Examples
///
/// Basic string operations:
/// ```rust
/// use trash_utilities::chars::core::*;
///
/// // Fast byte searching
/// let data = b"hello world";
/// assert_eq!(find_byte(data, b'w'), Some(6));
///
/// // String hashing and encoding
/// let hash = hash_string_fast("test");
/// let encoded = encode_string_base64("hello");
/// let decoded = decode_string_base64(&encoded).unwrap();
/// assert_eq!(decoded, "hello");
///
/// // Efficient splitting
/// let parts = split_string_efficient("a,b,c", ',');
/// assert_eq!(parts, vec!["a", "b", "c"]);
/// ```
// Standard library imports
use std::{hash::Hasher, sync::Arc};

// External crate imports
use ahash::{AHashMap, AHasher};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use memchr::memchr;
use parking_lot::Mutex;

/// Find the first occurrence of a byte in a slice
///
/// Uses the memchr crate for high-performance byte searching.
///
/// # Parameters
///
/// * `haystack` - The byte slice to search in.
/// * `needle` - The byte value to search for.
///
/// # Returns
///
/// The index of the first occurrence of `needle`, or `None` if not found.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::find_byte;
///
/// let data = b"hello world";
/// assert_eq!(find_byte(data, b'w'), Some(6));
/// assert_eq!(find_byte(data, b'z'), None);
/// ```
#[must_use]
pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    memchr(needle, haystack)
}

/// Find all occurrences of a byte in a slice
///
/// Returns all positions where the byte appears in the slice.
///
/// # Parameters
///
/// * `haystack` - The byte slice to search in.
/// * `needle` - The byte value to search for.
///
/// # Returns
///
/// A vector of indices where `needle` was found.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::find_byte_all;
///
/// let data = b"hello world";
/// let positions = find_byte_all(data, b'l');
/// assert_eq!(positions, vec![2, 3, 9]);
/// ```
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
///
/// Searches for the first occurrence of any byte in the `needles` slice.
///
/// # Parameters
///
/// * `haystack` - The byte slice to search in.
/// * `needles` - The set of bytes to search for.
///
/// # Returns
///
/// The index of the first occurrence of any byte from `needles`, or `None` if none found.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::find_any_byte;
///
/// let data = b"hello world";
/// let separators = b" ,!";
/// assert_eq!(find_any_byte(data, separators), Some(5)); // space at position 5
/// ```
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
///
/// Uses the `AHash` algorithm for high-performance hashing.
///
/// # Parameters
///
/// * `s` - The string to hash.
///
/// # Returns
///
/// A 64-bit hash value.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::hash_string_fast;
///
/// let hash1 = hash_string_fast("hello");
/// let hash2 = hash_string_fast("world");
/// assert_ne!(hash1, hash2);
/// assert_eq!(hash_string_fast("hello"), hash1); // Same input, same hash
/// ```
#[must_use]
pub fn hash_string_fast(s: &str) -> u64 {
    let mut hasher = AHasher::default();
    hasher.write(s.as_bytes());
    hasher.finish()
}

/// Encode a string as base64
///
/// Uses standard base64 encoding (RFC 4648).
///
/// # Parameters
///
/// * `s` - The string to encode.
///
/// # Returns
///
/// The base64-encoded string.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::encode_string_base64;
///
/// let encoded = encode_string_base64("hello world");
/// assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
/// ```
#[must_use]
pub fn encode_string_base64(s: &str) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, s.as_bytes())
}

/// Decode a base64 string
///
/// Decodes a base64 string back to UTF-8.
///
/// # Parameters
///
/// * `s` - The base64 string to decode.
///
/// # Returns
///
/// The decoded string on success.
///
/// # Errors
///
/// Returns an error if the input is not valid base64 or if the decoded bytes are not valid UTF-8.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::decode_string_base64;
///
/// let decoded = decode_string_base64("aGVsbG8gd29ybGQ=").unwrap();
/// assert_eq!(decoded, "hello world");
/// ```
pub fn decode_string_base64(s: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)?;
    String::from_utf8(bytes).map_err(Into::into)
}

/// Convenience: Hash and encode as base64
///
/// Computes a fast hash of the string and encodes it as base64.
///
/// # Parameters
///
/// * `s` - The string to hash and encode.
///
/// # Returns
///
/// The base64-encoded hash.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::hash_and_encode_base64;
///
/// let encoded_hash = hash_and_encode_base64("test data");
/// // Returns a base64 string representing the hash
/// assert!(!encoded_hash.is_empty());
/// ```
#[must_use]
pub fn hash_and_encode_base64(s: &str) -> String {
    let hash = crate::common::fast_hash(s.as_bytes());
    STANDARD.encode(hash.to_le_bytes())
}

/// Efficient string splitting with memchr
///
/// Splits a string by a delimiter using fast byte operations.
/// Handles UTF-8 boundaries correctly.
///
/// # Parameters
///
/// * `s` - The string to split.
/// * `delimiter` - The character to split on.
///
/// # Returns
///
/// A vector of string slices representing the split parts.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::split_string_efficient;
///
/// let parts = split_string_efficient("apple,banana,cherry", ',');
/// assert_eq!(parts, vec!["apple", "banana", "cherry"]);
/// ```
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
///
/// Counts how many times a substring appears in a string.
///
/// # Parameters
///
/// * `haystack` - The string to search in.
/// * `needle` - The substring to count.
///
/// # Returns
///
/// The number of times `needle` appears in `haystack`.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::count_substring;
///
/// let count = count_substring("hello hello world", "hello");
/// assert_eq!(count, 2);
///
/// let count = count_substring("test", "xyz");
/// assert_eq!(count, 0);
/// ```
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
///
/// Removes duplicate lines from a multi-line string, preserving order.
///
/// # Parameters
///
/// * `s` - The multi-line string to deduplicate.
///
/// # Returns
///
/// A new string with duplicate lines removed.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::deduplicate_lines;
///
/// let input = "line1\nline2\nline1\nline3\nline2";
/// let result = deduplicate_lines(input);
/// assert_eq!(result, "line1\nline2\nline3\n");
/// ```
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
///
/// A string interner stores unique strings and returns reference-counted
/// copies, reducing memory usage when the same strings are used repeatedly.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::StringInterner;
/// use std::sync::Arc;
///
/// let interner = StringInterner::new();
/// let s1: Arc<str> = interner.intern("hello");
/// let s2: Arc<str> = interner.intern("hello");
/// assert_eq!(s1.as_ptr(), s2.as_ptr()); // Same memory location
/// ```
#[derive(Debug)]
pub struct StringInterner {
    strings: Mutex<AHashMap<u64, Arc<str>>>,
}

impl StringInterner {
    /// Create a new interner
    ///
    /// # Returns
    ///
    /// A new empty `StringInterner`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::StringInterner;
    ///
    /// let interner = StringInterner::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: Mutex::new(AHashMap::new()),
        }
    }

    /// Intern a string, returning a reference-counted copy
    ///
    /// If the string has been interned before, returns a reference to the existing copy.
    /// Otherwise, stores the string and returns a reference to it.
    ///
    /// # Parameters
    ///
    /// * `s` - The string to intern.
    ///
    /// # Returns
    ///
    /// An `Arc<str>` pointing to the interned string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::StringInterner;
    ///
    /// let interner = StringInterner::new();
    /// let s1 = interner.intern("test");
    /// let s2 = interner.intern("test");
    /// assert_eq!(s1, s2);
    /// ```
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
    ///
    /// Returns the number of unique strings and total memory usage.
    ///
    /// # Returns
    ///
    /// A tuple of (`unique_string_count`, `total_bytes`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::StringInterner;
    ///
    /// let interner = StringInterner::new();
    /// interner.intern("hello");
    /// interner.intern("world");
    /// interner.intern("hello"); // Duplicate
    /// let (count, bytes) = interner.stats();
    /// assert_eq!(count, 2); // Only unique strings counted
    /// ```
    pub fn stats(&self) -> (usize, usize) {
        let strings = self.strings.lock();
        (strings.len(), strings.values().map(|s| s.len()).sum())
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory-efficient string builder with size hints
///
/// A string builder that allows efficient construction of strings by appending
/// content incrementally, with optional capacity hints for better performance.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::core::EfficientStringBuilder;
///
/// let mut builder = EfficientStringBuilder::with_capacity(100);
/// builder.append("Hello, ");
/// builder.append("world!");
/// let result = builder.build();
/// assert_eq!(result, "Hello, world!");
/// ```
#[derive(Debug)]
pub struct EfficientStringBuilder {
    buffer: String,
}

impl EfficientStringBuilder {
    /// Create a new builder
    ///
    /// # Returns
    ///
    /// A new `EfficientStringBuilder` with default capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::EfficientStringBuilder;
    ///
    /// let builder = EfficientStringBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Create a new builder with size hint
    ///
    /// # Parameters
    ///
    /// * `capacity` - The initial capacity to allocate for the string buffer.
    ///
    /// # Returns
    ///
    /// A new `EfficientStringBuilder` with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::EfficientStringBuilder;
    ///
    /// let builder = EfficientStringBuilder::with_capacity(1024);
    /// ```
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    /// Append a string
    ///
    /// # Parameters
    ///
    /// * `s` - The string to append.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::EfficientStringBuilder;
    ///
    /// let mut builder = EfficientStringBuilder::new();
    /// builder.append("Hello");
    /// builder.append(" ");
    /// builder.append("world");
    /// ```
    pub fn append(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Append a character
    ///
    /// # Parameters
    ///
    /// * `c` - The character to append.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::EfficientStringBuilder;
    ///
    /// let mut builder = EfficientStringBuilder::new();
    /// builder.append_char('H');
    /// builder.append_char('i');
    /// builder.append_char('!');
    /// ```
    pub fn append_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Build the final string
    ///
    /// Consumes the builder and returns the constructed string.
    ///
    /// # Returns
    ///
    /// The final constructed string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::EfficientStringBuilder;
    ///
    /// let mut builder = EfficientStringBuilder::new();
    /// builder.append("Final result");
    /// let result = builder.build();
    /// assert_eq!(result, "Final result");
    /// ```
    #[must_use]
    pub fn build(self) -> String {
        self.buffer
    }

    /// Get current length
    ///
    /// # Returns
    ///
    /// The current length of the string being built.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::EfficientStringBuilder;
    ///
    /// let mut builder = EfficientStringBuilder::new();
    /// builder.append("test");
    /// assert_eq!(builder.len(), 4);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the builder is empty
    ///
    /// # Returns
    ///
    /// `true` if no content has been added, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::chars::core::EfficientStringBuilder;
    ///
    /// let mut builder = EfficientStringBuilder::new();
    /// assert!(builder.is_empty());
    /// builder.append("content");
    /// assert!(!builder.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for EfficientStringBuilder {
    fn default() -> Self {
        Self::new()
    }
}
