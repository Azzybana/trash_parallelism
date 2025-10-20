/// Cryptographic and encoding utilities.
///
/// This module provides high-performance hashing, encoding/decoding,
/// compression/decompression, and byte manipulation utilities using
/// optimized algorithms and data structures.
///
/// # Examples
///
/// Basic hashing and encoding:
/// ```rust
/// use trash_utilities::common::crypto::*;
///
/// // Fast hashing
/// let hash = fast_hash(b"hello world");
/// assert_ne!(hash, 0);
///
/// // Base64 encoding/decoding
/// let data = b"secret data";
/// let encoded = encode_base64(data);
/// let decoded = decode_base64(&encoded).unwrap();
/// assert_eq!(decoded, data);
///
/// // Compression
/// let compressed = compress_brotli(b"some data to compress", 6).unwrap();
/// let decompressed = decompress_brotli(&compressed).unwrap();
/// assert_eq!(decompressed, b"some data to compress");
/// ```
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
/// Uses the Brotli compression algorithm for efficient data compression.
/// Higher compression levels provide better compression but are slower.
///
/// # Parameters
///
/// * `data` - The byte data to compress.
/// * `level` - Compression level (1-11, where 1 is fastest, 11 is best compression).
///
/// # Returns
///
/// The compressed data as a byte vector on success.
///
/// # Errors
///
/// Returns an `std::io::Error` if compression fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::crypto::compress_brotli;
///
/// let data = b"This is some data to compress";
/// let compressed = compress_brotli(data, 6).unwrap(); // Level 6 is a good balance
/// assert!(compressed.len() < data.len()); // Compressed data should be smaller
/// ```
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
/// Decompresses data that was compressed with Brotli.
///
/// # Parameters
///
/// * `data` - The compressed Brotli data to decompress.
///
/// # Returns
///
/// The decompressed data as a byte vector on success.
///
/// # Errors
///
/// Returns an `std::io::Error` if decompression fails (e.g., corrupted data).
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::crypto::{compress_brotli, decompress_brotli};
///
/// let original = b"Hello, compressed world!";
/// let compressed = compress_brotli(original, 6).unwrap();
/// let decompressed = decompress_brotli(&compressed).unwrap();
/// assert_eq!(decompressed, original);
/// ```
pub fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decompressor = Decompressor::new(data, 4096);
    let mut output = Vec::new();
    decompressor.read_to_end(&mut output)?;
    Ok(output)
}

/// Compute fast hash of data using ahash
///
/// Uses the `AHash` algorithm for high-performance hashing.
/// `AHash` is designed to be fast while maintaining good distribution properties.
///
/// # Parameters
///
/// * `data` - The byte data to hash.
///
/// # Returns
///
/// A 64-bit hash value.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::crypto::fast_hash;
///
/// let hash1 = fast_hash(b"hello");
/// let hash2 = fast_hash(b"world");
/// assert_ne!(hash1, hash2);
/// assert_eq!(fast_hash(b"hello"), hash1); // Deterministic
/// ```
#[must_use]
pub fn fast_hash(data: &[u8]) -> u64 {
    use ahash::AHasher;
    use std::hash::Hasher;
    let mut hasher = AHasher::default();
    hasher.write(data);
    hasher.finish()
}

/// Compute fast hash of data with key using ahash
///
/// Computes a hash that incorporates both a key and data.
/// Useful for keyed hashing operations where you want different keys to produce different hashes.
///
/// # Parameters
///
/// * `key` - The key bytes to include in the hash.
/// * `data` - The data bytes to hash.
///
/// # Returns
///
/// A 64-bit hash value combining both key and data.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::crypto::keyed_hash;
///
/// let key = b"secret_key";
/// let data = b"message";
/// let hash = keyed_hash(key, data);
/// // Same key+data always produces same hash
/// assert_eq!(keyed_hash(key, data), hash);
/// // Different key produces different hash
/// assert_ne!(keyed_hash(b"other_key", data), hash);
/// ```
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
///
/// Verifies that the provided data with the given key produces the expected hash.
/// Useful for integrity checking and authentication.
///
/// # Parameters
///
/// * `key` - The key bytes used in the original hash.
/// * `data` - The data bytes to verify.
/// * `expected` - The expected hash value.
///
/// # Returns
///
/// `true` if the computed hash matches the expected value, `false` otherwise.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::crypto::{keyed_hash, verify_keyed_hash};
///
/// let key = b"my_key";
/// let data = b"important data";
/// let hash = keyed_hash(key, data);
///
/// // Verification should succeed
/// assert!(verify_keyed_hash(key, data, hash));
///
/// // Verification should fail with wrong hash
/// assert!(!verify_keyed_hash(key, data, hash + 1));
///
/// // Verification should fail with wrong key
/// assert!(!verify_keyed_hash(b"wrong_key", data, hash));
/// ```
#[must_use]
pub fn verify_keyed_hash(key: &[u8], data: &[u8], expected: u64) -> bool {
    keyed_hash(key, data) == expected
}
