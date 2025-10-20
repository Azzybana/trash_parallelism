use futures_lite::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use smol::fs;
use std::io::{Read, Write};

/// Streaming serialization for files and network I/O.
///
/// This module provides streaming serialization capabilities for efficient
/// handling of large data structures and network protocols. Supports both
/// synchronous file I/O and asynchronous operations with futures.
///
/// ## Features
///
/// - **Streaming I/O**: Direct serialization to/from readers and writers
/// - **Async Support**: Non-blocking file operations with async/await
/// - **Memory Efficient**: Process large data without loading everything into memory
/// - **Network Ready**: Compatible with TCP streams and other I/O sources
/// - **Error Propagation**: Comprehensive error handling for I/O operations
/// - **Pretty Printing**: Formatted JSON output for configuration files
///
/// ## Examples
///
/// ### File-Based Configuration
/// ```rust,no_run
/// use trash_utilities::serde::{serialize_pretty_to_writer, deserialize_from_reader};
/// use serde::{Serialize, Deserialize};
/// use std::fs::File;
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct AppConfig {
///     database_url: String,
///     max_connections: u32,
///     features: Vec<String>,
/// }
///
/// let config = AppConfig {
///     database_url: "postgres://localhost/app".to_string(),
///     max_connections: 50,
///     features: vec!["logging".to_string(), "metrics".to_string()],
/// };
///
/// // Write pretty-printed config to file
/// let mut file = File::create("config.json").unwrap();
/// serialize_pretty_to_writer(&mut file, &config).unwrap();
///
/// // Read config back
/// let file = File::open("config.json").unwrap();
/// let loaded: AppConfig = deserialize_from_reader(file).unwrap();
/// assert_eq!(config.database_url, loaded.database_url);
/// ```
///
/// ### Async File Operations
/// ```rust,no_run
/// use trash_utilities::serde::{serialize_to_file_async, deserialize_from_file_async};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct UserData {
///     id: u64,
///     preferences: std::collections::HashMap<String, String>,
/// }
///
/// async fn save_and_load_user() -> Result<(), Box<dyn std::error::Error>> {
///     let user = UserData {
///         id: 12345,
///         preferences: [("theme".to_string(), "dark".to_string())].into(),
///     };
///
///     // Save asynchronously
///     serialize_to_file_async(&user, "user.json").await?;
///
///     // Load asynchronously
///     let loaded: UserData = deserialize_from_file_async("user.json").await?;
///     assert_eq!(user, loaded);
///
///     Ok(())
/// }
/// ```
///
/// ### Network Protocol Streaming
/// ```rust,no_run
/// use trash_utilities::serde::serialize_to_writer;
/// use serde::Serialize;
/// use std::net::TcpStream;
///
/// #[derive(Serialize)]
/// struct ApiRequest {
///     method: String,
///     path: String,
///     headers: std::collections::HashMap<String, String>,
/// }
///
/// fn send_request(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
///     let request = ApiRequest {
///         method: "POST".to_string(),
///         path: "/api/data".to_string(),
///         headers: [("content-type".to_string(), "application/json".to_string())].into(),
///     };
///
///     // Stream JSON directly to network
///     serialize_to_writer(stream, &request)?;
///     Ok(())
/// }
/// ```
///
/// ### Large Data Processing
/// ```rust,no_run
/// use trash_utilities::serde::deserialize_from_reader;
/// use serde::Deserialize;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// #[derive(Deserialize)]
/// struct LargeDataset {
///     records: Vec<Record>,
/// }
///
/// #[derive(Deserialize)]
/// struct Record {
///     id: u64,
///     data: Vec<f64>,
/// }
///
/// fn process_large_file() -> Result<(), Box<dyn std::error::Error>> {
///     // Use buffered reader for efficiency
///     let file = File::open("large_dataset.json")?;
///     let reader = BufReader::new(file);
///
///     // Stream deserialize without loading entire file into memory
///     let dataset: LargeDataset = deserialize_from_reader(reader)?;
///     
///     println!("Processed {} records", dataset.records.len());
///     Ok(())
/// }
/// ```
/// Serialize a value to JSON and write it to a writer.
///
/// This function serializes a value to JSON and writes it directly to any type
/// that implements `Write`. Useful for streaming JSON to files or network sockets.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
/// - `W`: The writer type, must implement `Write`.
///
/// # Parameters
/// - `writer`: The writer to output the JSON to.
/// - `value`: The value to serialize.
///
/// # Returns
/// - `Ok(())` if serialization and writing succeed.
/// - `Err(serde_json::Error)` if serialization fails.
/// - `Err(std::io::Error)` if writing fails.
///
/// # Errors
/// Returns `serde_json::Error` if the value cannot be serialized to JSON, or `std::io::Error` if writing to the writer fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::serde::serialize_to_writer;
/// use serde::Serialize;
/// use std::fs::File;
///
/// #[derive(Serialize)]
/// struct Person { name: String, age: u32 }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
/// let mut file = File::create("person.json").unwrap();
/// serialize_to_writer(&mut file, &person).unwrap();
/// ```
pub fn serialize_to_writer<W: Write, T: Serialize>(
    writer: W,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer(writer, value)?;
    Ok(())
}

/// Deserialize a value from JSON read from a reader.
///
/// This function reads JSON from any type that implements `Read` and deserializes
/// it into a struct. Useful for streaming JSON from files or network sockets.
///
/// # Type Parameters
/// - `R`: The reader type, must implement `Read`.
/// - `T`: The type to deserialize into, must implement `DeserializeOwned`.
///
/// # Parameters
/// - `reader`: The reader to read JSON from.
///
/// # Returns
/// - `Ok(T)` containing the deserialized value.
/// - `Err(serde_json::Error)` if deserialization fails.
/// - `Err(std::io::Error)` if reading fails.
///
/// # Errors
/// Returns `serde_json::Error` if the JSON is malformed or doesn't match the expected type, or `std::io::Error` if reading from the reader fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::serde::deserialize_from_reader;
/// use serde::Deserialize;
/// use std::fs::File;
///
/// #[derive(Deserialize, Debug)]
/// struct Person { name: String, age: u32 }
///
/// let file = File::open("person.json").unwrap();
/// let person: Person = deserialize_from_reader(file).unwrap();
/// println!("{:?}", person);
/// ```
pub fn deserialize_from_reader<R: Read, T: for<'de> Deserialize<'de>>(
    reader: R,
) -> Result<T, Box<dyn std::error::Error>> {
    let value: T = serde_json::from_reader(reader)?;
    Ok(value)
}

/// Serialize a value to pretty-printed JSON and write it to a writer.
///
/// Similar to `serialize_to_writer` but produces formatted JSON with indentation.
/// Useful for creating human-readable configuration files or debug output.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
/// - `W`: The writer type, must implement `Write`.
///
/// # Parameters
/// - `writer`: The writer to output the JSON to.
/// - `value`: The value to serialize.
///
/// # Returns
/// - `Ok(())` if serialization and writing succeed.
/// - `Err(serde_json::Error)` if serialization fails.
/// - `Err(std::io::Error)` if writing fails.
///
/// # Errors
/// Returns `serde_json::Error` if the value cannot be serialized to JSON, or `std::io::Error` if writing to the writer fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::serde::serialize_pretty_to_writer;
/// use serde::Serialize;
/// use std::fs::File;
///
/// #[derive(Serialize)]
/// struct Config { settings: Vec<String>, debug: bool }
///
/// let config = Config {
///     settings: vec!["option1".to_string()],
///     debug: true,
/// };
/// let mut file = File::create("config.json").unwrap();
/// serialize_pretty_to_writer(&mut file, &config).unwrap();
/// ```
pub fn serialize_pretty_to_writer<W: Write, T: Serialize>(
    writer: W,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer_pretty(writer, value)?;
    Ok(())
}

/// Asynchronously serialize a value to JSON and write to a file.
///
/// This function uses async I/O for efficient file operations.
/// Useful for non-blocking serialization to disk.
///
/// # Type Parameters
/// - `T`: The type to serialize, must implement `Serialize`.
///
/// # Parameters
/// - `value`: The value to serialize.
/// - `path`: The file path to write to.
///
/// # Returns
/// - `Ok(())` if serialization and writing succeed.
/// - `Err(Box<dyn std::error::Error>)` if serialization or I/O fails.
///
/// # Errors
/// Returns an error if the value cannot be serialized to JSON or if file I/O operations fail.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::serde::serialize_to_file_async;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config { debug: bool, level: String }
///
/// let config = Config { debug: true, level: "info".to_string() };
/// serialize_to_file_async(&config, "config.json").await.unwrap();
/// ```
pub async fn serialize_to_file_async<T: Serialize>(
    value: &T,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(value)?;
    let mut file = fs::File::create(path).await?;
    file.write_all(json.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

/// Asynchronously deserialize from a file.
///
/// This function uses async I/O to read and deserialize JSON from a file.
/// Useful for non-blocking file operations.
///
/// # Type Parameters
/// - `T`: The type to deserialize into, must implement `DeserializeOwned`.
///
/// # Parameters
/// - `path`: The file path to read from.
///
/// # Returns
/// - `Ok(T)` containing the deserialized value.
/// - `Err(Box<dyn std::error::Error>)` if I/O or deserialization fails.
///
/// # Errors
/// Returns an error if the file cannot be read or if the JSON content cannot be deserialized into the target type.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::serde::deserialize_from_file_async;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct Config { debug: bool, level: String }
///
/// let config: Config = deserialize_from_file_async("config.json").await.unwrap();
/// println!("Loaded config: {:?}", config);
/// ```
pub async fn deserialize_from_file_async<T: for<'de> Deserialize<'de>>(
    path: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let value: T = serde_json::from_str(&contents)?;
    Ok(value)
}
