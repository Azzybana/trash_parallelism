/// JSON processing utilities.
///
/// This module provides comprehensive JSON parsing, serialization,
/// validation, and manipulation utilities with error handling.
// External crate imports
use serde::{Deserialize, Serialize};
use serde_json;

/// JSON utilities with error handling
///
/// # Errors
///
/// Returns a `serde_json::Error` if parsing fails.
pub fn parse_json_value<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize a value to JSON string
///
/// # Errors
///
/// Returns a `serde_json::Error` if serialization fails.
pub fn to_json_value<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Serialize a value to pretty-printed JSON string
///
/// # Errors
///
/// Returns a `serde_json::Error` if serialization fails.
pub fn pretty_json_value<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

/// Validate JSON structure
#[must_use]
pub fn validate_json(json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(json).is_ok()
}

/// Merge two JSON objects
///
/// # Errors
///
/// Returns a `serde_json::Error` if parsing or serialization fails.
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
///
/// # Errors
///
/// Returns an error if the JSON is invalid, the path is not found, or traversal fails.
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
