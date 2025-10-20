/// Base64 encoding and decoding utilities for binary data serialization.
///
/// This module provides efficient base64 encoding/decoding operations,
/// including direct binary data conversion and JSON-embedded serialization.
/// Built on the standard base64 encoding with optimized performance.
///
/// ## Features
///
/// - **Binary Data Encoding**: Convert binary data to base64 strings for text protocols
/// - **JSON Integration**: Embed structured data in JSON using base64 encoding
/// - **Round-trip Safety**: Guaranteed lossless encoding/decoding
/// - **Standard Compliance**: RFC 4648 base64 encoding
/// - **Error Handling**: Comprehensive error reporting for invalid input
///
/// ## Examples
///
/// ### Basic Binary Encoding
/// ```rust
/// use trash_utilities::serde::{encode_base64, decode_base64};
///
/// let binary_data = b"Hello, World! This is binary data.";
/// let encoded = encode_base64(binary_data);
/// let decoded = decode_base64(&encoded).unwrap();
/// assert_eq!(binary_data.to_vec(), decoded);
/// ```
///
/// ### JSON with Embedded Binary Data
/// ```rust
/// use trash_utilities::serde::{serialize_to_base64_json, deserialize_from_base64_json};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Message {
///     text: String,
///     binary_attachment: Vec<u8>,
/// }
///
/// let message = Message {
///     text: "Check out this file!".to_string(),
///     binary_attachment: vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG header
/// };
///
/// // Serialize with binary data embedded as base64
/// let b64_json = serialize_to_base64_json(&message).unwrap();
/// let decoded: Message = deserialize_from_base64_json(&b64_json).unwrap();
/// assert_eq!(message, decoded);
/// ```
///
/// ### Protocol Buffers over HTTP
/// ```rust
/// use trash_utilities::serde::{encode_base64, decode_base64};
///
/// // Simulate sending protobuf data over HTTP
/// let protobuf_bytes = vec![0x08, 0x96, 0x01, 0x12, 0x07, 0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67];
/// let b64_payload = encode_base64(&protobuf_bytes);
///
/// // Send b64_payload in HTTP request...
/// // Later, decode back to protobuf
/// let recovered_bytes = decode_base64(&b64_payload).unwrap();
/// assert_eq!(protobuf_bytes, recovered_bytes);
/// ```
/// Encode data as base64 string.
///
/// # Parameters
/// - `data`: The binary data to encode.
///
/// # Returns
/// A base64-encoded string.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::encode_base64;
///
/// let data = b"Hello, world!";
/// let encoded = encode_base64(data);
/// println!("Base64: {}", encoded);
/// ```
#[must_use]
pub fn encode_base64(data: &[u8]) -> String {
    crate::common::encode_base64(data)
}

/// Decode a base64 string to binary data.
///
/// This function decodes a base64 string back to binary data.
/// Returns an error if the input is not valid base64.
///
/// # Parameters
/// - `data`: The base64 string to decode.
///
/// # Returns
/// - `Ok(Vec<u8>)` containing the decoded bytes.
/// - `Err(base64::DecodeError)` if decoding fails.
///
/// # Errors
/// Returns `base64::DecodeError` if the input string is not valid base64.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::{encode_base64, decode_base64};
///
/// let data = b"Hello, world!";
/// let encoded = encode_base64(data);
/// let decoded = decode_base64(&encoded).unwrap();
/// assert_eq!(decoded, data);
/// ```
pub fn decode_base64(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    crate::common::decode_base64(data)
}

/// Serialize a value to JSON and encode it as base64.
///
/// This convenience function serializes a value to JSON bytes and then
/// encodes those bytes as base64. Useful for embedding structured data
/// in text formats like JSON or XML.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
///
/// # Parameters
/// - `value`: The value to serialize and encode.
///
/// # Returns
/// - `Ok(String)` containing the base64-encoded JSON.
/// - `Err(serde_json::Error)` if serialization fails.
///
/// # Errors
/// Returns `serde_json::Error` if the value cannot be serialized to JSON.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::serialize_to_base64_json;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person { name: String, age: u32 }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
/// let b64_json = serialize_to_base64_json(&person).unwrap();
/// println!("Base64 JSON: {}", b64_json);
/// ```
pub fn serialize_to_base64_json<T: serde::Serialize>(
    value: &T,
) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    Ok(encode_base64(&bytes))
}

/// Decode base64 JSON and deserialize to a struct.
///
/// This function decodes a base64 string to JSON bytes and then deserializes
/// those bytes into a struct. The inverse of `serialize_to_base64_json`.
///
/// # Type Parameters
/// - `T`: The type to deserialize into, must implement `DeserializeOwned`.
///
/// # Parameters
/// - `b64_json`: The base64-encoded JSON string.
///
/// # Returns
/// - `Ok(T)` containing the deserialized value.
/// - `Err(Box<dyn std::error::Error>)` if decoding or deserialization fails.
///
/// # Errors
/// Returns an error if the base64 string is invalid or if the decoded JSON cannot be deserialized into the target type.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::{serialize_to_base64_json, deserialize_from_base64_json};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Person { name: String, age: u32 }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
/// let b64_json = serialize_to_base64_json(&person).unwrap();
/// let decoded: Person = deserialize_from_base64_json(&b64_json).unwrap();
/// assert_eq!(decoded, person);
/// ```
pub fn deserialize_from_base64_json<T: for<'de> serde::Deserialize<'de>>(
    b64_json: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = decode_base64(b64_json)?;
    let value: T = serde_json::from_slice(&bytes)?;
    Ok(value)
}
