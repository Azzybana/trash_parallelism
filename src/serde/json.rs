use serde::{Deserialize, Serialize};

/// JSON serialization and deserialization utilities.
///
/// This module provides comprehensive JSON handling with both compact and
/// pretty-printed output, validation, and efficient parsing. Built on `serde_json`
/// for maximum performance and compatibility.
///
/// ## Features
///
/// - **Compact Serialization**: Minified JSON for network efficiency
/// - **Pretty Printing**: Human-readable formatted JSON for debugging
/// - **Type Validation**: Runtime JSON structure validation
/// - **Flexible Parsing**: Support for owned and borrowed data
/// - **Error Handling**: Detailed error reporting with context
/// - **Performance Optimized**: Zero-copy operations where possible
///
/// ## Examples
///
/// ### Basic Serialization
/// ```rust
/// use trash_utilities::serde::{serialize_to_json, deserialize_from_json};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct User {
///     id: u32,
///     name: String,
///     email: String,
/// }
///
/// let user = User {
///     id: 123,
///     name: "Alice Johnson".to_string(),
///     email: "alice@example.com".to_string(),
/// };
///
/// // Serialize to compact JSON
/// let json = serialize_to_json(&user).unwrap();
/// assert_eq!(json, r#"{"id":123,"name":"Alice Johnson","email":"alice@example.com"}"#);
///
/// // Deserialize back
/// let recovered: User = deserialize_from_json(&json).unwrap();
/// assert_eq!(user, recovered);
/// ```
///
/// ### Pretty Printing for Configuration
/// ```rust
/// use trash_utilities::serde::pretty_json;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     database_url: String,
///     max_connections: u32,
///     features: Vec<String>,
/// }
///
/// let config = Config {
///     database_url: "postgres://localhost/mydb".to_string(),
///     max_connections: 100,
///     features: vec!["ssl".to_string(), "compression".to_string()],
/// };
///
/// let pretty = pretty_json(&config).unwrap();
/// println!("{}", pretty);
/// // Output:
/// // {
/// //   "database_url": "postgres://localhost/mydb",
/// //   "max_connections": 100,
/// //   "features": [
/// //     "ssl",
/// //     "compression"
/// //   ]
/// // }
/// ```
///
/// ### JSON Validation
/// ```rust
/// use trash_utilities::serde::validate_json;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct ApiResponse {
///     status: String,
///     data: Vec<String>,
/// }
///
/// let json = r#"{
///     "status": "success",
///     "data": ["item1", "item2"]
/// }"#;
///
/// // Validate and parse
/// let response: ApiResponse = validate_json(json).unwrap();
/// assert_eq!(response.status, "success");
/// assert_eq!(response.data.len(), 2);
/// ```
///
/// ### Error Handling
/// ```rust
/// use trash_utilities::serde::deserialize_from_json;
///
/// #[derive(serde::Deserialize, Debug)]
/// struct Person { name: String, age: u32 }
///
/// // Invalid JSON
/// let result: Result<Person, _> = deserialize_from_json(r#"{"name": "Alice", "age": "thirty"}"#);
/// assert!(result.is_err()); // Age should be a number
///
/// // Missing required field
/// let result2: Result<Person, _> = deserialize_from_json(r#"{"name": "Bob"}"#);
/// assert!(result2.is_err()); // Missing age field
/// ```
/// Serialize a struct to a compact JSON string.
///
/// This function converts any serializable type into a minified JSON string.
/// It's suitable for network transmission or storage where size matters.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
///
/// # Parameters
/// - `value`: The value to serialize.
///
/// # Returns
/// - `Ok(String)` containing the JSON representation.
/// - `Err(serde_json::Error)` if serialization fails.
///
/// # Errors
/// Returns `serde_json::Error` if the value cannot be serialized to JSON.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::serialize_to_json;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person { name: String, age: u32 }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
/// let json = serialize_to_json(&person).unwrap();
/// assert_eq!(json, r#"{"name":"Alice","age":30}"#);
/// ```
pub fn serialize_to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Deserialize a JSON string into a struct.
///
/// This function parses a JSON string into any deserializable type.
/// The lifetime parameter ensures the returned value doesn't outlive the input string.
///
/// # Type Parameters
/// - `T`: The type to deserialize into, must implement `Deserialize<'a>`.
///
/// # Parameters
/// - `json`: The JSON string to parse.
///
/// # Returns
/// - `Ok(T)` containing the deserialized value.
/// - `Err(serde_json::Error)` if deserialization fails.
///
/// # Errors
/// Returns a `serde_json::Error` if the JSON is malformed or doesn't match the expected type.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::deserialize_from_json;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct Person { name: String, age: u32 }
///
/// let json = r#"{"name":"Alice","age":30}"#;
/// let person: Person = deserialize_from_json(json).unwrap();
/// assert_eq!(person, Person { name: "Alice".to_string(), age: 30 });
/// ```
pub fn deserialize_from_json<'a, T: Deserialize<'a>>(
    json: &'a str,
) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse a JSON value from a string.
///
/// This is a convenience function for deserializing JSON.
///
/// # Type Parameters
/// - `T`: The type to deserialize into.
///
/// # Parameters
/// - `s`: The JSON string.
///
/// # Returns
/// - `Ok(T)` on success.
/// - `Err(serde_json::Error)` on failure.
///
/// # Errors
/// Returns a `serde_json::Error` if the JSON is malformed or doesn't match the expected type.
pub fn parse_json_value<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(s)
}

/// Pretty-print a struct as formatted JSON.
///
/// This function produces human-readable JSON with proper indentation and spacing.
/// Useful for configuration files, debugging, or user-facing output.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
///
/// # Parameters
/// - `value`: The value to serialize.
///
/// # Returns
/// - `Ok(String)` containing the pretty-printed JSON.
/// - `Err(serde_json::Error)` if serialization fails.
///
/// # Errors
/// Returns a `serde_json::Error` if the value cannot be serialized to JSON.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::pretty_json;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person { name: String, age: u32 }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
/// let json = pretty_json(&person).unwrap();
/// println!("{}", json);
/// // Output:
/// // {
/// //   "name": "Alice",
/// //   "age": 30
/// // }
/// ```
pub fn pretty_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

/// Validate JSON structure against a schema at runtime.
///
/// This function attempts to deserialize and re-serialize to validate
/// the JSON structure. More comprehensive validation would require
/// a JSON Schema library.
///
/// # Type Parameters
/// - `T`: The type to validate against, must implement `Serialize + DeserializeOwned`.
///
/// # Parameters
/// - `json`: The JSON string to validate.
///
/// # Returns
/// - `Ok(T)` if the JSON is valid for the type.
/// - `Err(serde_json::Error)` if validation fails.
///
/// # Errors
/// Returns `serde_json::Error` if the JSON is malformed or doesn't match the expected type.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::validate_json;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Person { name: String, age: u32 }
///
/// let json = r#"{"name":"Alice","age":30}"#;
/// let person: Person = validate_json(json).unwrap();
/// println!("Valid person: {:?}", person);
/// ```
pub fn validate_json<T: for<'de> Deserialize<'de> + Serialize>(
    json: &str,
) -> Result<T, serde_json::Error> {
    let value: T = serde_json::from_str(json)?;
    // Re-serialize to ensure it's fully valid
    let _ = serde_json::to_string(&value)?;
    Ok(value)
}
