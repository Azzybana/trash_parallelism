//! Data processing utilities for parsing, encoding, and serialization.
//!
//! This module provides functions for parsing various data formats,
//! encoding/decoding operations, and JSON manipulation utilities.

// Standard library imports
// (none)

// External crate imports
use ahash::AHashMap;
use memchr::memchr;

/// Parses key-value pairs from a string content, typically from config files.
///
/// This function assumes each line is in the format "key=value". It uses `memchr`
/// for efficient byte-level searching of the '=' character. Lines without '=' are ignored.
///
/// # Parameters
/// - `content`: The string content to parse.
///
/// # Returns
/// An `AHashMap<String, String>` containing the parsed key-value pairs.
/// Keys and values are trimmed of whitespace.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::data::parse_key_value;
/// use ahash::AHashMap;
///
/// let content = "name=John\nage=30\ncity=New York";
/// let map = parse_key_value(content);
/// assert_eq!(map.get("name"), Some(&"John".to_string()));
/// assert_eq!(map.get("age"), Some(&"30".to_string()));
/// ```
#[must_use]
pub fn parse_key_value(content: &str) -> AHashMap<String, String> {
    let mut map = AHashMap::new();
    for line in content.lines() {
        if let Some(pos) = memchr(b'=', line.as_bytes()) {
            let key = &line[..pos];
            let value = &line[pos + 1..];
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}
