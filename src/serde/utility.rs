use ahash::AHasher;
use chrono::{DateTime, Utc};
use memchr::memchr;
use serde::{Deserialize, Serialize};
use std::hash::Hasher;

/// Utility functions for advanced JSON processing and metadata handling.
///
/// This module provides specialized utilities for JSON operations including
/// fast hashing, timestamping, logging, and efficient key/value extraction.
/// Designed for high-performance applications requiring additional metadata
/// and debugging capabilities.
///
/// ## Features
///
/// - **Fast Hashing**: Non-cryptographic hashing using ahash for performance
/// - **Timestamp Metadata**: Automatic timestamping for audit trails
/// - **Operation Logging**: Debug logging for serialization operations
/// - **Key Extraction**: Efficient JSON key searching and value extraction
/// - **Memory Efficient**: Optimized string operations using memchr
/// - **Type Safe**: Full compile-time type checking with serde
///
/// ## Examples
///
/// ### Fast Content Hashing
/// ```rust
/// use trash_utilities::serde::hash_json_ahash;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Document {
///     id: u64,
///     title: String,
///     content: String,
/// }
///
/// let doc = Document {
///     id: 123,
///     title: "Important Document".to_string(),
///     content: "This is the document content...".to_string(),
/// };
///
/// let hash = hash_json_ahash(&doc).unwrap();
/// println!("Document hash: {}", hash);
/// // Use hash for caching, deduplication, or change detection
/// ```
///
/// ### Timestamped Operations
/// ```rust
/// use trash_utilities::serde::{serialize_with_timestamp, deserialize_with_timestamp};
/// use serde::Serialize;
/// use chrono::{DateTime, Utc};
///
/// #[derive(Serialize, serde::Deserialize, Debug)]
/// struct UserAction {
///     user_id: u64,
///     action: String,
/// }
///
/// let action = UserAction {
///     user_id: 456,
///     action: "login".to_string(),
/// };
///
/// // Serialize with automatic timestamp
/// let json = serialize_with_timestamp(&action, "user_action").unwrap();
/// println!("{}", json);
/// // {"timestamp":"2025-10-19T...","operation":"user_action","data":{...}}
///
/// // Deserialize with metadata
/// let (data, timestamp, operation): (UserAction, DateTime<Utc>, String) =
///     deserialize_with_timestamp(&json).unwrap();
/// ```
///
/// ### JSON Key Operations
/// ```rust
/// use trash_utilities::serde::{json_contains_key, extract_json_value};
///
/// let user_json = r#"{
///     "id": 123,
///     "name": "Alice",
///     "email": "alice@example.com",
///     "active": true
/// }"#;
///
/// // Fast key existence check
/// assert!(json_contains_key(user_json, "name"));
/// assert!(!json_contains_key(user_json, "phone"));
///
/// // Extract specific values
/// assert_eq!(extract_json_value(user_json, "name"), Some(r#""Alice""#.to_string()));
/// assert_eq!(extract_json_value(user_json, "id"), Some("123".to_string()));
/// assert_eq!(extract_json_value(user_json, "phone"), None);
/// ```
///
/// ### Debug Logging
/// ```rust
/// use trash_utilities::serde::serialize_with_logging;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct ApiRequest {
///     endpoint: String,
///     params: std::collections::HashMap<String, String>,
/// }
///
/// let request = ApiRequest {
///     endpoint: "/api/users".to_string(),
///     params: [("limit".to_string(), "10".to_string())].into(),
/// };
///
/// // Serialize with debug logging
/// let json = serialize_with_logging(&request, "api_request").unwrap();
/// // Output: Serializing with context: api_request
/// ```
/// Compute a fast non-cryptographic hash of serialized JSON.
///
/// This function uses ahash for fast hashing, suitable for hash tables
/// and checksums where cryptographic security is not required.
///
/// # Type Parameters
/// - `T`: The type to serialize and hash, must implement `Serialize`.
///
/// # Parameters
/// - `value`: The value to hash.
///
/// # Returns
/// - `Ok(u64)` containing the hash value.
/// - `Err(serde_json::Error)` if serialization fails.
///
/// # Errors
/// Returns `serde_json::Error` if the value cannot be serialized to JSON.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::hash_json_ahash;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Data { id: u32, name: String }
///
/// let data = Data { id: 123, name: "test".to_string() };
/// let hash = hash_json_ahash(&data).unwrap();
/// println!("Fast hash: {}", hash);
/// ```
pub fn hash_json_ahash<T: Serialize>(value: &T) -> Result<u64, serde_json::Error> {
    let json = serde_json::to_string(value)?;
    let mut hasher = AHasher::default();
    hasher.write(json.as_bytes());
    Ok(hasher.finish())
}

/// Serialize a value to JSON with logging.
///
/// This function serializes a value to JSON and logs the operation,
/// useful for debugging serialization performance.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
///
/// # Parameters
/// - `value`: The value to serialize.
/// - `context`: A context string for logging.
///
/// # Returns
/// - `Ok(String)` containing the JSON representation.
/// - `Err(serde_json::Error)` if serialization fails.
///
/// # Errors
/// Returns `serde_json::Error` if the value cannot be serialized to JSON.
pub fn serialize_with_logging<T: Serialize>(
    value: &T,
    context: &str,
) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(value)?;
    println!("Serializing with context: {context}");
    Ok(json)
}

/// Serialize with timestamp metadata.
///
/// This function wraps the data with timestamp information, useful for
/// versioning or audit trails.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
///
/// # Parameters
/// - `value`: The value to serialize.
/// - `operation`: A string describing the operation (e.g., "create", "update").
///
/// # Returns
/// - `Ok(String)` containing JSON with timestamp metadata.
/// - `Err(serde_json::Error)` if serialization fails.
///
/// # Errors
/// Returns `serde_json::Error` if the value cannot be serialized to JSON.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::serialize_with_timestamp;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config { settings: Vec<String> }
///
/// let config = Config { settings: vec!["debug".to_string()] };
/// let json = serialize_with_timestamp(&config, "save_config").unwrap();
/// println!("{}", json);
/// // Output: {"timestamp":"2025-10-19T...","operation":"save_config","data":{...}}
/// ```
pub fn serialize_with_timestamp<T: Serialize>(
    value: &T,
    operation: &str,
) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct TimestampedData<'a, T> {
        timestamp: DateTime<Utc>,
        operation: &'a str,
        data: &'a T,
    }

    let timestamped = TimestampedData {
        timestamp: Utc::now(),
        operation,
        data: value,
    };

    serde_json::to_string(&timestamped)
}

/// Deserialize timestamped data.
///
/// This function deserializes data that was serialized with `serialize_with_timestamp`.
///
/// # Type Parameters
/// - `T`: The type to deserialize into, must implement `DeserializeOwned`.
///
/// # Parameters
/// - `json`: The JSON string containing timestamped data.
///
/// # Returns
/// - `Ok((T, DateTime<Utc>, String))` containing the data, timestamp, and operation.
/// - `Err(serde_json::Error)` if deserialization fails.
///
/// # Errors
/// Returns `serde_json::Error` if the JSON is malformed or doesn't match the expected timestamped data structure.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::{serialize_with_timestamp, deserialize_with_timestamp};
/// use serde::Serialize;
/// use chrono::{DateTime, Utc};
///
/// #[derive(Serialize, serde::Deserialize, Debug)]
/// struct Config { settings: Vec<String> }
///
/// let config = Config { settings: vec!["debug".to_string()] };
/// let json = serialize_with_timestamp(&config, "save").unwrap();
/// let (data, timestamp, operation): (Config, DateTime<Utc>, String) =
///     deserialize_with_timestamp(&json).unwrap();
/// println!("Operation '{}' at {}", operation, timestamp);
/// ```
pub fn deserialize_with_timestamp<T: for<'de> Deserialize<'de>>(
    json: &str,
) -> Result<(T, DateTime<Utc>, String), serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct TimestampedData<T> {
        timestamp: DateTime<Utc>,
        operation: String,
        data: T,
    }

    let timestamped: TimestampedData<T> = serde_json::from_str(json)?;
    Ok((
        timestamped.data,
        timestamped.timestamp,
        timestamped.operation,
    ))
}

/// Efficiently search for a key in JSON string using memchr.
///
/// This function uses optimized string searching to quickly find JSON keys.
/// Much faster than regex for simple key lookups.
///
/// # Parameters
/// - `json`: The JSON string to search.
/// - `key`: The key to find (without quotes).
///
/// # Returns
/// `true` if the key exists in the JSON, `false` otherwise.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::json_contains_key;
///
/// let json = r#"{"name":"Alice","age":30,"active":true}"#;
/// assert!(json_contains_key(json, "name"));
/// assert!(json_contains_key(json, "age"));
/// assert!(!json_contains_key(json, "email"));
/// ```
#[must_use]
pub fn json_contains_key(json: &str, key: &str) -> bool {
    // Look for "key": pattern
    let key_pattern = format!("\"{key}\":");
    memchr(b'"', json.as_bytes()).is_some() && json.contains(&key_pattern)
}

/// Extract a JSON value by key path using efficient searching.
///
/// This function finds a top-level key in JSON and extracts its value.
/// Uses memchr for efficient string operations.
///
/// # Parameters
/// - `json`: The JSON string to search.
/// - `key`: The key to extract.
///
/// # Returns
/// - `Some(String)` containing the JSON value if found.
/// - `None` if the key is not found or parsing fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::extract_json_value;
///
/// let json = r#"{"name":"Alice","age":30,"active":true}"#;
/// assert_eq!(extract_json_value(json, "name"), Some(r#""Alice""#.to_string()));
/// assert_eq!(extract_json_value(json, "age"), Some("30".to_string()));
/// assert_eq!(extract_json_value(json, "email"), None);
/// ```
#[must_use]
pub fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let key_pattern = format!("\"{key}\":");
    if let Some(key_start) = json.find(&key_pattern) {
        let value_start = key_start + key_pattern.len();
        let remaining = &json[value_start..];

        // Find the value boundaries
        let chars = remaining.chars();
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut value_end = 0;

        for (i, c) in chars.enumerate() {
            if c == '}' && brace_count == 0 && bracket_count == 0 && !in_string {
                value_end = i;
                break;
            }
            match c {
                '"' => in_string = !in_string,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => brace_count -= 1,
                '[' if !in_string => bracket_count += 1,
                ']' if !in_string => bracket_count -= 1,
                ',' if !in_string && brace_count == 0 && bracket_count == 0 => {
                    value_end = i;
                    break;
                }
                _ => {}
            }
        }

        if value_end > 0 {
            Some(remaining[..value_end].trim().to_string())
        } else {
            None
        }
    } else {
        None
    }
}
