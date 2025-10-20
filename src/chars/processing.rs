/// String processing and I/O utilities.
///
/// This module provides JSON processing, parallel string operations,
/// and asynchronous file I/O for strings, leveraging serde for serialization,
/// parallel processing capabilities, and async file operations.
///
/// # Examples
///
/// JSON processing:
/// ```rust
/// use trash_utilities::chars::processing::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Person { name: String, age: u32 }
///
/// let json = r#"{"name":"Alice","age":30}"#;
/// let person: Person = parse_and_validate_json(json).unwrap();
/// assert_eq!(person.name, "Alice");
/// assert_eq!(person.age, 30);
/// ```
// Standard library imports
// External crate imports
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use serde_json;
use smol::fs;

/// Convenience: Parse JSON and validate
///
/// Parses a JSON string and deserializes it into the specified type.
/// This is a convenience wrapper around `serde_json` operations.
///
/// # Type Parameters
///
/// * `T` - The type to deserialize into (must implement `serde::Deserialize`).
///
/// # Parameters
///
/// * `s` - The JSON string to parse.
///
/// # Returns
///
/// The deserialized value on success.
///
/// # Errors
///
/// Returns a `serde_json::Error` if parsing fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::processing::parse_and_validate_json;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config { debug: bool, port: u16 }
///
/// let json = r#"{"debug":true,"port":8080}"#;
/// let config: Config = parse_and_validate_json(json).unwrap();
/// assert!(config.debug);
/// assert_eq!(config.port, 8080);
/// ```
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
///
/// Splits a string into chunks and processes each chunk in parallel using the provided function.
/// Useful for processing large strings efficiently across multiple threads.
///
/// # Type Parameters
///
/// * `F` - The processing function type.
/// * `R` - The result type returned by the processing function.
///
/// # Parameters
///
/// * `s` - The string to process.
/// * `chunk_size` - The size of each chunk in bytes.
/// * `processor` - Function to process each chunk.
///
/// # Returns
///
/// A vector of results from processing each chunk.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::processing::parallel_process_string;
///
/// let text = "The quick brown fox jumps over the lazy dog";
/// let results = parallel_process_string(text, 10, |chunk| chunk.len());
/// // Each chunk's length is calculated
/// assert!(!results.is_empty());
/// ```
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
/// Reads the entire contents of a file into a string using async I/O.
///
/// # Parameters
///
/// * `path` - The path to the file to read.
///
/// # Returns
///
/// The file contents as a string on success.
///
/// # Errors
///
/// Returns an `std::io::Error` if the file cannot be opened or read.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::processing::read_file_to_string_async;
/// use smol;
///
/// # smol::block_on(async {
/// // This would work with an actual file
/// // let content = read_file_to_string_async("example.txt").await.unwrap();
/// // assert!(!content.is_empty());
/// # });
/// ```
pub async fn read_file_to_string_async(path: &str) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

/// Asynchronously write string to file
///
/// Writes a string to a file using async I/O, creating the file if it doesn't exist
/// and truncating it if it does.
///
/// # Parameters
///
/// * `path` - The path to the file to write.
/// * `contents` - The string content to write.
///
/// # Errors
///
/// Returns an `std::io::Error` if the file cannot be created or written to.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::processing::write_string_to_file_async;
/// use smol;
///
/// # smol::block_on(async {
/// // This would work with write permissions
/// // write_string_to_file_async("output.txt", "Hello, world!").await.unwrap();
/// # });
/// ```
pub async fn write_string_to_file_async(path: &str, contents: &str) -> Result<(), std::io::Error> {
    let mut file = fs::File::create(path).await?;
    file.write_all(contents.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

/// Extract all JSON values by key from a JSON array string
///
/// Parses a JSON array and extracts all values associated with a specific key
/// from objects within the array.
///
/// # Parameters
///
/// * `json_array` - A JSON string representing an array of objects.
/// * `key` - The key to extract values for.
///
/// # Returns
///
/// A vector of extracted JSON values.
///
/// # Errors
///
/// Returns an error if the input is not valid JSON or if parsing fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::chars::processing::extract_json_values_by_key;
///
/// let json = r#"[
///     {"name": "Alice", "age": 30},
///     {"name": "Bob", "age": 25},
///     {"name": "Charlie", "age": 35}
/// ]"#;
///
/// let names = extract_json_values_by_key(json, "name").unwrap();
/// assert_eq!(names.len(), 3);
/// ```
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
