/// Cryptographic and encoding utilities.
///
/// This module provides high-performance hashing, encoding/decoding,
/// compression/decompression, and byte manipulation utilities using
/// optimized algorithms and data structures.
// Standard library imports
use std::io::{Read, Write};

// External crate imports
use ahash::AHashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use brotli::{CompressorWriter, Decompressor};
use bytes::Bytes;

/// Creates a new empty `AHashMap` with the `AHash` hashing algorithm.
///
/// `AHash` is a high-performance hashing algorithm, often faster than the standard
/// library's hasher for hash maps.
///
/// # Type Parameters
/// - `K`: The key type, must implement `Hash` and `Eq`.
/// - `V`: The value type.
///
/// # Returns
/// An empty `AHashMap<K, V>`.
///
/// # Examples
/// ```rust
/// use trash_analyzer::common::crypto::create_ahash_map;
/// use ahash::AHashMap;
///
/// let mut map: AHashMap<String, i32> = create_ahash_map();
/// map.insert("key".to_string(), 42);
/// ```
#[must_use]
pub fn create_ahash_map<K, V>() -> AHashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    AHashMap::new()
}

/// Wraps a byte slice into a `Bytes` object for efficient sharing.
///
/// This function creates a copy of the data in `Bytes`, which is useful for
/// zero-copy operations in async contexts.
///
/// # Parameters
/// - `data`: The byte slice to wrap.
///
/// # Returns
/// A `Bytes` object containing a copy of the input data.
///
/// # Examples
/// ```rust
/// use trash_analyzer::common::crypto::wrap_bytes;
/// use bytes::Bytes;
///
/// let data = b"Hello, world!";
/// let bytes = wrap_bytes(data);
/// assert_eq!(bytes.as_ref(), data);
/// ```
#[must_use]
pub fn wrap_bytes(data: &[u8]) -> Bytes {
    Bytes::copy_from_slice(data)
}

/// Encodes binary data to a Base64 string.
///
/// This function uses the standard Base64 encoding without padding.
///
/// # Parameters
/// - `data`: The byte slice to encode.
///
/// # Returns
/// A `String` containing the Base64-encoded data.
///
/// # Examples
/// ```rust
/// use trash_analyzer::common::crypto::encode_base64;
///
/// let data = b"Hello, world!";
/// let encoded = encode_base64(data);
/// println!("Encoded: {}", encoded);
/// ```
#[must_use]
pub fn encode_base64(data: &[u8]) -> String {
    STANDARD.encode(data)
}

/// Decodes a Base64 string to binary data.
///
/// This function decodes standard Base64 without padding.
///
/// # Parameters
/// - `data`: The Base64-encoded string to decode.
///
/// # Returns
/// - `Ok(Vec<u8>)` containing the decoded bytes if successful.
/// - `Err(base64::DecodeError)` if decoding fails (e.g., invalid Base64).
///
/// # Errors
///
/// Returns a `base64::DecodeError` if the input is not valid base64.
///
/// # Examples
/// ```rust
/// use trash_analyzer::common::crypto::{encode_base64, decode_base64};
///
/// let data = b"Hello, world!";
/// let encoded = encode_base64(data);
/// let decoded = decode_base64(&encoded).unwrap();
/// assert_eq!(decoded, data);
/// ```
pub fn decode_base64(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    STANDARD.decode(data)
}

/// Compress data using Brotli
///
/// # Errors
///
/// Returns an `std::io::Error` if compression fails.
pub fn compress_brotli(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    let mut output = Vec::new();
    {
        let mut compressor = CompressorWriter::new(&mut output, 4096, level, level);
        compressor.write_all(data)?;
        compressor.flush()?;
    }
    Ok(output)
}

/// Decompress Brotli data
///
/// # Errors
///
/// Returns an `std::io::Error` if decompression fails.
pub fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decompressor = Decompressor::new(data, 4096);
    let mut output = Vec::new();
    decompressor.read_to_end(&mut output)?;
    Ok(output)
}

/// Compute fast hash of data using ahash
#[must_use]
pub fn fast_hash(data: &[u8]) -> u64 {
    use ahash::AHasher;
    use std::hash::Hasher;
    let mut hasher = AHasher::default();
    hasher.write(data);
    hasher.finish()
}

/// Compute fast hash of data with key using ahash
#[must_use]
pub fn keyed_hash(key: &[u8], data: &[u8]) -> u64 {
    use ahash::AHasher;
    use std::hash::Hasher;
    let mut hasher = AHasher::default();
    hasher.write(key);
    hasher.write(data);
    hasher.finish()
}

/// Verify keyed hash
#[must_use]
pub fn verify_keyed_hash(key: &[u8], data: &[u8], expected: u64) -> bool {
    keyed_hash(key, data) == expected
}
