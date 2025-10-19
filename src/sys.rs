/// System utilities for time handling, environment variables, file system operations, and system interactions.
///
/// This module provides convenient wrappers for system-level operations including
/// time manipulation, environment variable access, path operations, and file metadata.
// Standard library imports
use std::time::{Duration, Instant};

// External crate imports
use chrono::{DateTime, Utc};
use tempfile::NamedTempFile;
use tracing::info;

/// Gets the current UTC time.
///
/// This function returns the current time in UTC using `chrono::Utc::now()`.
///
/// # Returns
/// A `DateTime<Utc>` representing the current UTC time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::current_utc_time;
/// use chrono::{DateTime, Utc};
///
/// let now: DateTime<Utc> = current_utc_time();
/// println!("Current time: {}", now);
/// ```
#[must_use]
pub fn current_utc_time() -> DateTime<Utc> {
    Utc::now()
}

/// Formats a `DateTime<Utc>` to an ISO 8601 string.
///
/// This function uses RFC 3339 format, which is a profile of ISO 8601.
///
/// # Parameters
/// - `dt`: The `DateTime<Utc>` to format.
///
/// # Returns
/// A `String` containing the formatted date and time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::{current_utc_time, format_datetime};
/// use chrono::DateTime;
///
/// let now = current_utc_time();
/// let formatted = format_datetime(&now);
/// println!("Formatted time: {}", formatted);
/// ```
#[must_use]
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Parses a date/time string in RFC 3339 format.
///
/// # Parameters
/// - `s`: The string to parse.
///
/// # Returns
/// - `Ok(DateTime<Utc>)` if parsing succeeds.
/// - `Err(chrono::ParseError)` if parsing fails.
///
/// # Errors
/// Returns a `chrono::ParseError` if the string is not a valid RFC 3339 date/time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::parse_datetime;
///
/// let dt = parse_datetime("2023-01-01T12:00:00Z").unwrap();
/// println!("Parsed datetime: {}", dt);
/// ```
pub fn parse_datetime(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
}

/// Parses a date string in YYYY-MM-DD format.
///
/// # Parameters
/// - `s`: The string to parse.
///
/// # Returns
/// - `Ok(NaiveDate)` if parsing succeeds.
/// - `Err(chrono::ParseError)` if parsing fails.
///
/// # Errors
/// Returns a `chrono::ParseError` if the string is not in YYYY-MM-DD format.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::parse_date;
/// use chrono::NaiveDate;
///
/// let date = parse_date("2023-01-01").unwrap();
/// println!("Parsed date: {}", date);
/// ```
pub fn parse_date(s: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
}

/// Reads an environment variable as a string.
///
/// # Parameters
/// - `key`: The name of the environment variable.
///
/// # Returns
/// - `Ok(String)` containing the value if the variable exists.
/// - `Err(std::env::VarError)` if the variable is not set or invalid.
///
/// # Errors
/// Returns a `std::env::VarError` if the environment variable is not set or contains invalid Unicode.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::read_env_var;
///
/// let home = read_env_var("HOME").unwrap();
/// println!("Home directory: {}", home);
/// ```
pub fn read_env_var(key: &str) -> Result<String, std::env::VarError> {
    std::env::var(key)
}

/// Reads an environment variable with a default value.
///
/// # Parameters
/// - `key`: The name of the environment variable.
/// - `default`: The default value to return if the variable is not set.
///
/// # Returns
/// The value of the environment variable or the default.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::read_env_var_or;
///
/// let port = read_env_var_or("PORT", "8080");
/// println!("Port: {}", port);
/// ```
#[must_use]
pub fn read_env_var_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Reads an environment variable and parses it as a type that implements `FromStr`.
///
/// # Type Parameters
/// - `T`: The type to parse the value into, must implement `FromStr`.
///
/// # Parameters
/// - `key`: The name of the environment variable.
///
/// # Returns
/// - `Ok(T)` if parsing succeeds.
/// - `Err` if the variable is not set or parsing fails.
///
/// # Errors
/// Returns an error if the environment variable is not set, or if parsing the value fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::read_env_var_parse;
///
/// let port: u16 = read_env_var_parse("PORT").unwrap_or(8080);
/// println!("Port: {}", port);
/// ```
pub fn read_env_var_parse<T>(key: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + 'static,
{
    let value = std::env::var(key)?;
    Ok(value.parse()?)
}

/// Normalizes a file path, resolving `.` and `..` components.
///
/// # Parameters
/// - `path`: The path to normalize.
///
/// # Returns
/// A normalized path as a `String`.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::normalize_path;
///
/// let normalized = normalize_path("./foo/../bar");
/// assert_eq!(normalized, "bar");
/// ```
#[must_use]
pub fn normalize_path(path: &str) -> String {
    let path = std::path::Path::new(path);
    path.components()
        .fold(std::path::PathBuf::new(), |mut acc, comp| {
            match comp {
                std::path::Component::Normal(s) => acc.push(s),
                std::path::Component::ParentDir => {
                    acc.pop();
                }
                std::path::Component::CurDir => {}
                _ => acc.push(comp.as_os_str()),
            }
            acc
        })
        .to_string_lossy()
        .to_string()
}

/// Joins multiple path components into a single path.
///
/// # Parameters
/// - `base`: The base path.
/// - `components`: Additional path components to join.
///
/// # Returns
/// The joined path as a `String`.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::join_paths;
///
/// let path = join_paths("home", &["user", "docs", "file.txt"]);
/// // On Unix: "home/user/docs/file.txt"
/// ```
#[must_use]
pub fn join_paths(base: &str, components: &[&str]) -> String {
    let mut path = std::path::PathBuf::from(base);
    for comp in components {
        path.push(comp);
    }
    path.to_string_lossy().to_string()
}

/// Gets the file extension from a path.
///
/// # Parameters
/// - `path`: The file path.
///
/// # Returns
/// - `Some(String)` containing the extension if present.
/// - `None` if no extension.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::get_file_extension;
///
/// assert_eq!(get_file_extension("file.txt"), Some("txt".to_string()));
/// assert_eq!(get_file_extension("file"), None);
/// ```
#[must_use]
pub fn get_file_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .map(std::string::ToString::to_string)
}

/// Gets the file name without extension.
///
/// # Parameters
/// - `path`: The file path.
///
/// # Returns
/// - `Some(String)` containing the file name without extension.
/// - `None` if the path has no file name.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::get_file_stem;
///
/// assert_eq!(get_file_stem("file.txt"), Some("file".to_string()));
/// assert_eq!(get_file_stem("dir/"), None);
/// ```
#[must_use]
pub fn get_file_stem(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(std::string::ToString::to_string)
}

/// Gets file metadata synchronously.
///
/// # Parameters
/// - `path`: The file path.
///
/// # Returns
/// - `Ok(std::fs::Metadata)` containing file metadata.
/// - `Err(std::io::Error)` if metadata cannot be read.
///
/// # Errors
/// Returns an `std::io::Error` if the file does not exist, permission is denied, or other I/O error occurs.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::sys::get_file_metadata;
///
/// let metadata = get_file_metadata("file.txt").unwrap();
/// println!("File size: {}", metadata.len());
/// ```
pub fn get_file_metadata(path: &str) -> Result<std::fs::Metadata, std::io::Error> {
    std::fs::metadata(path)
}

/// Gets file size.
///
/// # Parameters
/// - `path`: The file path.
///
/// # Returns
/// - `Ok(u64)` containing the file size in bytes.
/// - `Err(std::io::Error)` if the file cannot be accessed.
///
/// # Errors
/// Returns an `std::io::Error` if the file cannot be accessed or metadata cannot be read.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::sys::get_file_size;
///
/// let size = get_file_size("file.txt").unwrap();
/// println!("File size: {} bytes", size);
/// ```
pub fn get_file_size(path: &str) -> Result<u64, std::io::Error> {
    Ok(get_file_metadata(path)?.len())
}

/// Checks if a path exists and is a file.
///
/// # Parameters
/// - `path`: The path to check.
///
/// # Returns
/// `true` if the path exists and is a file, `false` otherwise.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::is_file;
///
/// assert!(is_file("Cargo.toml"));
/// assert!(!is_file("nonexistent.txt"));
/// ```
#[must_use]
pub fn is_file(path: &str) -> bool {
    std::path::Path::new(path).is_file()
}

/// Checks if a path exists and is a directory.
///
/// # Parameters
/// - `path`: The path to check.
///
/// # Returns
/// `true` if the path exists and is a directory, `false` otherwise.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::sys::is_directory;
///
/// assert!(is_directory("src"));
/// assert!(!is_directory("Cargo.toml"));
/// ```
#[must_use]
pub fn is_directory(path: &str) -> bool {
    std::path::Path::new(path).is_dir()
}

/// Gets the last modified time of a file.
///
/// # Parameters
/// - `path`: The file path.
///
/// # Returns
/// - `Ok(SystemTime)` representing the last modified time.
/// - `Err(std::io::Error)` if the time cannot be retrieved.
///
/// # Errors
/// Returns an `std::io::Error` if the file does not exist or metadata cannot be read.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::sys::get_file_modified_time;
/// use std::time::SystemTime;
///
/// let modified = get_file_modified_time("file.txt").unwrap();
/// println!("Last modified: {:?}", modified);
/// ```
pub fn get_file_modified_time(path: &str) -> Result<std::time::SystemTime, std::io::Error> {
    get_file_metadata(path)?.modified()
}

/// Creates a new temporary file.
///
/// The file is created in the system's temporary directory and will be
/// automatically deleted when the `NamedTempFile` is dropped, unless
/// `keep()` is called on it.
///
/// # Returns
/// - `Ok(NamedTempFile)` if the temporary file was created successfully.
/// - `Err(std::io::Error)` if creation failed.
///
/// # Errors
/// Returns an `std::io::Error` if the temporary file cannot be created.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::sys::create_temp_file;
/// use std::io::Write;
///
/// let mut temp_file = create_temp_file().unwrap();
/// temp_file.write_all(b"Hello, temp!").unwrap();
/// // File is automatically deleted when temp_file goes out of scope
/// ```
pub fn create_temp_file() -> Result<NamedTempFile, std::io::Error> {
    NamedTempFile::new()
}

/// Performance timer
pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    /// Start a new timer
    #[must_use]
    pub fn new(label: &str) -> Self {
        Self {
            start: Instant::now(),
            label: label.to_string(),
        }
    }

    /// Get elapsed time
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        info!("Timer '{}' completed in {:?}", self.label, elapsed);
    }
}
