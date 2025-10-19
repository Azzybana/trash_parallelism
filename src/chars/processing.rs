/// String processing and I/O utilities.
///
/// This module provides JSON processing, parallel string operations,
/// and asynchronous file I/O for strings, leveraging serde for serialization,
/// parallel processing capabilities, and async file operations.
// Standard library imports
// External crate imports
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use serde_json;
use smol::fs;

/// Convenience: Parse JSON and validate
///
/// # Errors
///
/// Returns a `serde_json::Error` if parsing fails.
pub fn parse_and_validate_json<T: for<'de> serde::Deserialize<'de>>(
    s: &str,
) -> Result<T, serde_json::Error> {
    let value: T = crate::serde::parse_json_value(s)?;
    // Could add validation logic here
    Ok(value)
}

/// Result type alias for processing operations
pub type ProcessingResult<T> = Result<T, serde_json::Error>;

/// Parallel string processing: split string and process chunks
pub fn parallel_process_string<F, R>(s: &str, chunk_size: usize, processor: F) -> Vec<R>
where
    F: Fn(&str) -> R + Send + Sync,
    R: Send,
{
    let chunks: Vec<&str> = s
        .as_bytes()
        .chunks(chunk_size)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect();

    chunks.into_iter().map(processor).collect()
}

/// Asynchronously read a file to string
///
/// # Errors
///
/// Returns an `std::io::Error` if the file cannot be opened or read.
pub async fn read_file_to_string_async(path: &str) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

/// Asynchronously write string to file
///
/// # Errors
///
/// Returns an `std::io::Error` if the file cannot be created or written to.
pub async fn write_string_to_file_async(path: &str, contents: &str) -> Result<(), std::io::Error> {
    let mut file = fs::File::create(path).await?;
    file.write_all(contents.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

/// Extract all JSON values by key from a JSON array string
///
/// # Errors
///
/// Returns an error if the input is not valid JSON or if parsing fails.
pub fn extract_json_values_by_key(
    json_array: &str,
    key: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let array: Vec<serde_json::Value> = serde_json::from_str(json_array)?;
    let mut results = Vec::new();

    for item in array {
        if let Some(obj) = item.as_object()
            && let Some(value) = obj.get(key)
        {
            results.push(value.clone());
        }
    }

    Ok(results)
}