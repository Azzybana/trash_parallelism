/// Async data processing utilities for compression, serialization, and cryptography.
///
/// This module provides non-blocking operations for data transformation using brotli compression,
/// serde serialization, and cryptographic functions with ahash and base64.
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
/// # Errors
///
/// Returns an `std::io::Error` if compression fails.
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
/// # Errors
///
/// Returns an `std::io::Error` if decompression fails.
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
/// # Errors
///
/// Returns a `serde_json::Error` if serialization fails.
pub fn serialize_async<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Deserializes a value from a JSON string.
///
/// # Errors
///
/// Returns a `serde_json::Error` if deserialization fails.
pub fn deserialize_async<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

/// Async cryptographic operations (non-blocking)
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

pub async fn encode_base64_async(data: &[u8]) -> String {
    let data = data.to_vec();
    smol::unblock(move || STANDARD.encode(&data)).await
}

/// Decodes a base64-encoded string to bytes.
///
/// # Errors
///
/// Returns a `base64::DecodeError` if the input is not valid base64.
pub async fn decode_base64_async(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let data = data.to_string();
    smol::unblock(move || STANDARD.decode(&data)).await
}
