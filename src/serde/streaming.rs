use std::io::{Read, Write};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use smol::fs;

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
