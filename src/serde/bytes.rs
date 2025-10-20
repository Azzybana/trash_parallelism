use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Efficient serialization using Bytes buffers for zero-copy operations.
///
/// This module provides serialization utilities that leverage the `bytes` crate
/// for efficient memory management and zero-copy operations. Particularly useful
/// for network protocols, streaming data, and high-performance applications.
///
/// ## Features
///
/// - **Zero-Copy Operations**: Efficient sharing and cloning of serialized data
/// - **Memory Efficiency**: Reduced allocations through buffer reuse
/// - **Thread Safety**: Safe concurrent access to serialized data
/// - **Streaming Compatible**: Works well with async streaming patterns
/// - **Reference Counting**: Automatic memory management with Arc-like semantics
///
/// ## Examples
///
/// ### Efficient Data Sharing
/// ```rust
/// use trash_utilities::serde::{serialize_to_bytes, deserialize_from_bytes};
/// use serde::{Serialize, Deserialize};
/// use bytes::Bytes;
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Message {
///     id: u64,
///     payload: Vec<u8>,
/// }
///
/// let message = Message {
///     id: 12345,
///     payload: vec![1, 2, 3, 4, 5],
/// };
///
/// // Serialize to Bytes buffer
/// let bytes: Bytes = serialize_to_bytes(&message).unwrap();
///
/// // Clone cheaply (no data copying)
/// let bytes_clone = bytes.clone();
///
/// // Deserialize from either buffer
/// let msg1: Message = deserialize_from_bytes(&bytes).unwrap();
/// let msg2: Message = deserialize_from_bytes(&bytes_clone).unwrap();
///
/// assert_eq!(msg1, msg2);
/// assert_eq!(msg1, message);
/// ```
///
/// ### Network Protocol Efficiency
/// ```rust,no_run
/// use trash_utilities::serde::serialize_to_bytes;
/// use serde::Serialize;
/// use bytes::Bytes;
///
/// #[derive(Serialize)]
/// struct Packet {
///     sequence: u32,
///     data: Vec<u8>,
/// }
///
/// async fn send_packet(packet: Packet) -> Result<(), Box<dyn std::error::Error>> {
///     let bytes: Bytes = serialize_to_bytes(&packet)?;
///     
///     // Send bytes over network without copying
///     // network_send(bytes).await?;
///     
///     Ok(())
/// }
/// ```
///
/// ### Memory Pool Integration
/// ```rust
/// use trash_utilities::serde::{serialize_to_bytes, deserialize_from_bytes};
/// use bytes::Bytes;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct CacheEntry {
///     key: String,
///     value: Vec<u8>,
/// }
///
/// // Simulate a memory cache
/// fn cache_operation() -> Result<(), Box<dyn std::error::Error>> {
///     let entry = CacheEntry {
///         key: "user_123".to_string(),
///         value: vec![0; 1024], // 1KB of data
///     };
///     
///     // Serialize once, reuse buffer
///     let serialized: Bytes = serialize_to_bytes(&entry)?;
///     
///     // Store buffer in cache (efficient)
///     // cache.insert("user_123", serialized.clone());
///     
///     // Deserialize when needed
///     let recovered: CacheEntry = deserialize_from_bytes(&serialized)?;
///     
///     Ok(())
/// }
/// ```
/// Work with Bytes buffers for efficient memory management.
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
