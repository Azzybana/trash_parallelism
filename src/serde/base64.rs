/// Encode data as base64 string.
///
/// This function encodes binary data to a base64 string using standard encoding.
/// Useful for transmitting binary data over text-based protocols.
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
