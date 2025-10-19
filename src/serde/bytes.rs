use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Work with Bytes buffers for efficient memory management.
///
/// This function serializes to JSON and wraps the result in a Bytes buffer,
/// which is more efficient for sharing and zero-copy operations.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
///
/// # Parameters
/// - `value`: The value to serialize.
///
/// # Returns
/// - `Ok(Bytes)` containing the JSON data in a Bytes buffer.
/// - `Err(serde_json::Error)` if serialization fails.
///
/// # Errors
/// Returns `serde_json::Error` if the value cannot be serialized to JSON.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::serialize_to_bytes;
/// use serde::Serialize;
/// use bytes::Bytes;
///
/// #[derive(Serialize)]
/// struct Message { text: String, id: u64 }
///
/// let msg = Message { text: "Hello".to_string(), id: 123 };
/// let bytes: Bytes = serialize_to_bytes(&msg).unwrap();
/// // Can be cloned cheaply and shared across threads
/// let cloned = bytes.clone();
/// ```
pub fn serialize_to_bytes<T: Serialize>(value: &T) -> Result<Bytes, serde_json::Error> {
    let vec = serde_json::to_vec(value)?;
    Ok(Bytes::from(vec))
}

/// Deserialize from a Bytes buffer.
///
/// This function deserializes JSON data from a Bytes buffer, which is
/// efficient for zero-copy operations.
///
/// # Type Parameters
/// - `T`: The type to deserialize into, must implement `DeserializeOwned`.
///
/// # Parameters
/// - `bytes`: The Bytes buffer containing JSON data.
///
/// # Returns
/// - `Ok(T)` containing the deserialized value.
/// - `Err(serde_json::Error)` if deserialization fails.
///
/// # Errors
/// Returns `serde_json::Error` if the JSON is malformed or doesn't match the expected type.
///
/// # Examples
/// ```rust
/// use trash_analyzer::serde::{serialize_to_bytes, deserialize_from_bytes};
/// use serde::{Serialize, Deserialize};
/// use bytes::Bytes;
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Message { text: String, id: u64 }
///
/// let msg = Message { text: "Hello".to_string(), id: 123 };
/// let bytes = serialize_to_bytes(&msg).unwrap();
/// let deserialized: Message = deserialize_from_bytes(&bytes).unwrap();
/// assert_eq!(deserialized, msg);
/// ```
pub fn deserialize_from_bytes<T: for<'de> Deserialize<'de>>(
    bytes: &Bytes,
) -> Result<T, serde_json::Error> {
    serde_json::from_slice(bytes.as_ref())
}
