/// JSON processing utilities.
///
/// This module provides comprehensive JSON parsing, serialization,
/// validation, and manipulation utilities with error handling.
///
/// # Examples
///
/// Basic JSON operations:
/// ```rust
/// use trash_utilities::common::json::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Person { name: String, age: u32 }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
///
/// // Serialize to JSON
/// let json = to_json_value(&person).unwrap();
/// println!("JSON: {}", json);
///
/// // Parse from JSON
/// let parsed: Person = parse_json_value(&json).unwrap();
/// assert_eq!(parsed.name, "Alice");
///
/// // Pretty print
/// let pretty = pretty_json_value(&person).unwrap();
/// println!("Pretty JSON:\n{}", pretty);
///
/// // Validate JSON
/// assert!(validate_json(&json));
///
/// // Extract by path
/// let nested_json = r#"{"user": {"name": "Bob", "age": 25}}"#;
/// let name = extract_json_path(nested_json, "user.name").unwrap();
/// assert_eq!(name.unwrap().as_str().unwrap(), "Bob");
/// ```
// External crate imports
use serde::{Deserialize, Serialize};
use serde_json;

/// JSON utilities with error handling
///
/// Parses a JSON string into the specified type using serde.
///
/// # Type Parameters
///
/// * `T` - The type to deserialize into (must implement `serde::Deserialize`).
///
/// # Parameters
///
/// * `json` - The JSON string to parse.
///
/// # Returns
///
/// The deserialized value on success.
///
/// # Errors
///
/// Returns a `serde_json::Error` if parsing fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::json::parse_json_value;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config { debug: bool, port: u16 }
///
/// let json = r#"{"debug":true,"port":8080}"#;
/// let config: Config = parse_json_value(json).unwrap();
/// assert!(config.debug);
/// assert_eq!(config.port, 8080);
/// ```
pub fn parse_json_value<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize a value to JSON string
///
/// Converts a value that implements `serde::Serialize` into a compact JSON string.
///
/// # Type Parameters
///
/// * `T` - The type to serialize (must implement `serde::Serialize`).
///
/// # Parameters
///
/// * `value` - The value to serialize.
///
/// # Returns
///
/// A compact JSON string representation on success.
///
/// # Errors
///
/// Returns a `serde_json::Error` if serialization fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::json::to_json_value;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Point { x: i32, y: i32 }
///
/// let point = Point { x: 10, y: 20 };
/// let json = to_json_value(&point).unwrap();
/// assert_eq!(json, r#"{"x":10,"y":20}"#);
/// ```
pub fn to_json_value<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Serialize a value to pretty-printed JSON string
///
/// Converts a value into a human-readable JSON string with proper indentation.
///
/// # Type Parameters
///
/// * `T` - The type to serialize (must implement `serde::Serialize`).
///
/// # Parameters
///
/// * `value` - The value to serialize.
///
/// # Returns
///
/// A pretty-printed JSON string on success.
///
/// # Errors
///
/// Returns a `serde_json::Error` if serialization fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::json::pretty_json_value;
///
/// let data = vec![1, 2, 3];
/// let pretty = pretty_json_value(&data).unwrap();
/// println!("Pretty JSON:\n{}", pretty);
/// // Output will be formatted with indentation
/// ```
pub fn pretty_json_value<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

/// Validate JSON structure
///
/// Checks if a string contains valid JSON without deserializing it to a specific type.
/// Useful for quick validation before further processing.
///
/// # Parameters
///
/// * `json` - The JSON string to validate.
///
/// # Returns
///
/// `true` if the string is valid JSON, `false` otherwise.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::json::validate_json;
///
/// assert!(validate_json(r#"{"name":"Alice","age":30}"#));
/// assert!(validate_json(r#"[1,2,3]"#));
/// assert!(!validate_json(r#"{"invalid": json}"#));
/// assert!(!validate_json("not json at all"));
/// ```
#[must_use]
pub fn validate_json(json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(json).is_ok()
}

/// Merge two JSON objects
///
/// Merges two JSON objects by combining their properties.
/// If both inputs are objects, properties from the second object are added to the first.
/// Other JSON value types are not supported for merging.
///
/// # Parameters
///
/// * `a` - The base JSON string (must be a JSON object).
/// * `b` - The JSON string to merge into the base (must be a JSON object).
///
/// # Returns
///
/// A new JSON string with the merged objects on success.
///
/// # Errors
///
/// Returns a `serde_json::Error` if parsing or serialization fails,
/// or if either input is not a JSON object.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::json::merge_json;
///
/// let base = r#"{"name":"Alice","age":30}"#;
/// let extra = r#"{"city":"New York","job":"Engineer"}"#;
/// let merged = merge_json(base, extra).unwrap();
/// // Result: {"name":"Alice","age":30,"city":"New York","job":"Engineer"}
/// println!("Merged: {}", merged);
/// ```
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
/// Traverses a JSON object using dot-notation paths to extract nested values.
/// For example, "user.name" would extract the "name" field from within a "user" object.
///
/// # Parameters
///
/// * `json` - The JSON string to traverse.
/// * `path` - The dot-separated path to the desired value (e.g., "user.profile.name").
///
/// # Returns
///
/// `Some(value)` if the path exists and can be traversed, `None` if the path doesn't exist.
///
/// # Errors
///
/// Returns an error if the JSON is invalid, or if traversal encounters a non-object value
/// when trying to access a property.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::json::extract_json_path;
///
/// let json = r#"{
///     "user": {
///         "name": "Alice",
///         "profile": {
///             "age": 30,
///             "city": "Boston"
///         }
///     },
///     "active": true
/// }"#;
///
/// let name = extract_json_path(json, "user.name").unwrap();
/// assert_eq!(name.unwrap().as_str().unwrap(), "Alice");
///
/// let age = extract_json_path(json, "user.profile.age").unwrap();
/// assert_eq!(age.unwrap().as_u64().unwrap(), 30);
///
/// let missing = extract_json_path(json, "user.missing").unwrap();
/// assert!(missing.is_none());
/// ```
pub fn extract_json_path(
    json: &str,
    path: &str,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let mut current = &value;

    for segment in path.split('.') {
        match current {
            serde_json::Value::Object(obj) => {
                if let Some(next) = obj.get(segment) {
                    current = next;
                } else {
                    return Ok(None);
                }
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
