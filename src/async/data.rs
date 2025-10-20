/// Async data processing utilities for compression, serialization, and cryptography.
///
/// This module provides non-blocking operations for data transformation using brotli compression,
/// serde serialization, and cryptographic functions with ahash and base64.
///
/// # Examples
///
/// Basic compression and decompression:
/// ```rust
/// use trash_utilities::async::data::{compress_data_async, decompress_data_async};
/// use smol;
///
/// # smol::block_on(async {
/// let data = b"Hello, world!";
/// let compressed = compress_data_async(data, 6).await.unwrap();
/// let decompressed = decompress_data_async(&compressed).await.unwrap();
/// assert_eq!(decompressed, data);
/// # });
/// ```
// Standard library imports
use std::io::{Read, Write};

// External crate imports
use base64::{Engine, engine::general_purpose::STANDARD};
use brotli::{CompressorWriter, Decompressor};
use serde::{Deserialize, Serialize};
use serde_json;
use smol;

/// Compresses the given data using Brotli compression at the specified level.
///
/// This function performs compression asynchronously using the smol runtime,
/// allowing other tasks to run while compression is in progress.
///
/// # Parameters
///
/// * `data` - The data to compress as a byte slice.
/// * `level` - The compression level (0-11, where 0 is fastest and 11 is most compressed).
///
/// # Returns
///
/// Returns the compressed data as a `Vec<u8>` on success.
///
/// # Errors
///
/// Returns an `std::io::Error` if compression fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::data::compress_data_async;
/// use smol;
///
/// # smol::block_on(async {
/// let data = b"Hello, world!";
/// let compressed = compress_data_async(data, 6).await.unwrap();
/// assert!(!compressed.is_empty());
/// # });
/// ```
pub async fn compress_data_async(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    let data = data.to_vec();
    smol::unblock(move || {
        let mut output = Vec::new();
        {
            let mut compressor = CompressorWriter::new(&mut output, 4096, level, level);
            compressor.write_all(&data)?;
            compressor.flush()?;
        }
        Ok(output)
    })
    .await
}

/// Decompresses the given Brotli-compressed data.
///
/// This function performs decompression asynchronously using the smol runtime.
///
/// # Parameters
///
/// * `data` - The compressed data as a byte slice.
///
/// # Returns
///
/// Returns the decompressed data as a `Vec<u8>` on success.
///
/// # Errors
///
/// Returns an `std::io::Error` if decompression fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::data::{compress_data_async, decompress_data_async};
/// use smol;
///
/// # smol::block_on(async {
/// let data = b"Hello, world!";
/// let compressed = compress_data_async(data, 6).await.unwrap();
/// let decompressed = decompress_data_async(&compressed).await.unwrap();
/// assert_eq!(decompressed, data);
/// # });
/// ```
pub async fn decompress_data_async(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let data = data.to_vec();
    smol::unblock(move || {
        let mut decompressor = Decompressor::new(&data[..], 4096);
        let mut output = Vec::new();
        decompressor.read_to_end(&mut output)?;
        Ok(output)
    })
    .await
}

/// Serializes the given value to a JSON string.
///
/// This function uses `serde_json` for serialization.
///
/// # Parameters
///
/// * `value` - The value to serialize, must implement `Serialize`.
///
/// # Type Parameters
///
/// * `T` - The type of the value to serialize.
///
/// # Returns
///
/// Returns the JSON string representation on success.
///
/// # Errors
///
/// Returns a `serde_json::Error` if serialization fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::data::serialize_async;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
/// let json = serialize_async(&person).unwrap();
/// assert!(json.contains("Alice"));
/// ```
pub fn serialize_async<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Deserializes a value from a JSON string.
///
/// This function uses `serde_json` for deserialization.
///
/// # Parameters
///
/// * `json` - The JSON string to deserialize.
///
/// # Type Parameters
///
/// * `T` - The type to deserialize into, must implement `Deserialize`.
///
/// # Returns
///
/// Returns the deserialized value on success.
///
/// # Errors
///
/// Returns a `serde_json::Error` if deserialization fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::data::deserialize_async;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let json = r#"{"name":"Alice","age":30}"#;
/// let person: Person = deserialize_async(json).unwrap();
/// assert_eq!(person.name, "Alice");
/// ```
pub fn deserialize_async<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

/// Async cryptographic operations (non-blocking)
///
/// Computes a 64-bit hash of the given data using the ahash algorithm.
///
/// # Parameters
///
/// * `data` - The data to hash as a byte slice.
///
/// # Returns
///
/// Returns the 64-bit hash value.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::data::hash_data_async;
/// use smol;
///
/// # smol::block_on(async {
/// let data = b"Hello, world!";
/// let hash = hash_data_async(data).await;
/// assert_ne!(hash, 0);
/// # });
/// ```
pub async fn hash_data_async(data: &[u8]) -> u64 {
    let data = data.to_vec();
    smol::unblock(move || {
        use ahash::AHasher;
        use std::hash::Hasher;
        let mut hasher = AHasher::default();
        hasher.write(&data);
        hasher.finish()
    })
    .await
}

/// Encodes the given data to a base64 string.
///
/// Uses the standard base64 encoding.
///
/// # Parameters
///
/// * `data` - The data to encode as a byte slice.
///
/// # Returns
///
/// Returns the base64-encoded string.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::data::encode_base64_async;
/// use smol;
///
/// # smol::block_on(async {
/// let data = b"Hello, world!";
/// let encoded = encode_base64_async(data).await;
/// assert!(!encoded.is_empty());
/// # });
/// ```
pub async fn encode_base64_async(data: &[u8]) -> String {
    let data = data.to_vec();
    smol::unblock(move || STANDARD.encode(&data)).await
}

/// Decodes a base64-encoded string to bytes.
///
/// Uses the standard base64 decoding.
///
/// # Parameters
///
/// * `data` - The base64-encoded string to decode.
///
/// # Returns
///
/// Returns the decoded bytes as a `Vec<u8>` on success.
///
/// # Errors
///
/// Returns a `base64::DecodeError` if the input is not valid base64.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::data::{encode_base64_async, decode_base64_async};
/// use smol;
///
/// # smol::block_on(async {
/// let data = b"Hello, world!";
/// let encoded = encode_base64_async(data).await;
/// let decoded = decode_base64_async(&encoded).await.unwrap();
/// assert_eq!(decoded, data);
/// # });
/// ```
pub async fn decode_base64_async(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let data = data.to_string();
    smol::unblock(move || STANDARD.decode(&data)).await
}
