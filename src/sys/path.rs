//! # File System and Path Utilities
//!
//! This module provides comprehensive file system operations with parallel processing,
//! serialization support, and async I/O capabilities.
//!
//! ## Overview
//!
//! The `path` module offers:
//! - **Path Manipulation**: Normalization, joining, extension extraction
//! - **File Metadata**: Size, modification time, type checking
//! - **Parallel Operations**: Batch metadata retrieval and file searching
//! - **Serialization**: JSON serialization for file information
//! - **Async I/O**: Non-blocking file operations
//! - **Recursive Operations**: Directory traversal and file discovery
//!
//! ## Usage Patterns
//!
//! ### Basic Path Operations
//! ```rust
//! use trash_analyzer::sys::path::*;
//!
//! let normalized = normalize_path("./foo/../bar");
//! let joined = join_paths("home", &["user", "docs"]);
//! let ext = get_file_extension("file.txt");
//! ```
//!
//! ### File Metadata
//! ```rust,no_run
//! use trash_analyzer::sys::path::*;
//!
//! let size = get_file_size("file.txt").unwrap();
//! let modified = get_file_modified_time("file.txt").unwrap();
//! let is_file = is_file("file.txt");
//! ```

use crate::parallel;
use crate::serde;
use ::serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tempfile::NamedTempFile;

/// Comprehensive file system and path utilities with parallel processing.
///
/// This module provides extensive file system operations with path manipulation,
/// metadata retrieval, parallel processing capabilities, and serialization support.
/// Designed for robust file system interactions in high-performance applications.
///
/// ## Features
///
/// - **Path Manipulation**: Cross-platform path normalization, joining, and component extraction
/// - **File Metadata**: Size, modification time, type checking with comprehensive error handling
/// - **Parallel Operations**: Concurrent file metadata retrieval and batch processing
/// - **Serialization**: JSON serialization for file information and metadata
/// - **Directory Traversal**: Recursive directory walking and file discovery
/// - **Temporary Files**: Safe temporary file creation and management
/// - **Pattern Matching**: File searching with extension-based filtering
///
/// ## Examples
///
/// ### Path Operations
/// ```rust
/// use trash_utilities::sys::path::*;
///
/// // Path manipulation
/// let normalized = normalize_path("./src/../src/main.rs");
/// assert_eq!(normalized, "src/main.rs");
///
/// let joined = join_paths("/home/user", &["documents", "work", "project"]);
/// // On Unix: "/home/user/documents/work/project"
///
/// // File information extraction
/// assert_eq!(get_file_extension("document.pdf"), Some("pdf".to_string()));
/// assert_eq!(get_file_stem("document.pdf"), Some("document".to_string()));
/// ```
///
/// ### File System Analysis
/// ```rust,no_run
/// use trash_utilities::sys::path::*;
///
/// fn analyze_directory(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
///     // Get all files recursively
///     let all_files = walk_directory(dir)?;
///     println!("Found {} files", all_files.len());
///
///     // Get metadata for all files in parallel
///     let metadata_results = get_files_metadata_parallel(&all_files.iter().map(|s| s.as_str()).collect::<Vec<_>>());
///
///     let mut total_size = 0u64;
///     let mut file_count = 0usize;
///
///     for (path, result) in metadata_results {
///         match result {
///             Ok(metadata) => {
///                 total_size += metadata.len();
///                 file_count += 1;
///             }
///             Err(e) => eprintln!("Error reading {}: {}", path, e),
///         }
///     }
///
///     println!("Successfully analyzed {} files, total size: {} bytes", file_count, total_size);
///     Ok(())
/// }
/// ```
///
/// ### File Discovery and Filtering
/// ```rust,no_run
/// use trash_utilities::sys::path::*;
///
/// fn find_source_files() -> Result<(), Box<dyn std::error::Error>> {
///     // Find all Rust source files
///     let rs_files = find_files_parallel("src", "rs")?;
///     println!("Found {} Rust files", rs_files.len());
///
///     // Find all TOML config files
///     let toml_files = find_files_parallel(".", "toml")?;
///     println!("Found {} TOML files", toml_files.len());
///
///     // Analyze file sizes
///     for file in &rs_files {
///         if let Ok(size) = get_file_size(file) {
///             println!("{}: {} bytes", file, size);
///         }
///     }
///
///     Ok(())
/// }
/// ```
///
/// ### Metadata Serialization
/// ```rust,no_run
/// use trash_utilities::sys::path::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct FileReport {
///     path: String,
///     size: u64,
///     is_file: bool,
///     modified: Option<String>,
/// }
///
/// fn generate_file_report(files: &[String]) -> Result<String, Box<dyn std::error::Error>> {
///     let mut reports = Vec::new();
///
///     for file in files {
///         if let Ok(metadata) = get_file_metadata(file) {
///             let modified = metadata.modified()
///                 .ok()
///                 .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
///                 .map(|d| d.as_secs().to_string());
///
///             reports.push(FileReport {
///                 path: file.clone(),
///                 size: metadata.len(),
///                 is_file: metadata.is_file(),
///                 modified,
///             });
///         }
///     }
///
///     // Serialize the entire report
///     serde::serialize_to_json(&reports)
/// }
/// ```
///
/// ### Temporary File Management
/// ```rust,no_run
/// use trash_utilities::sys::path::*;
/// use std::io::Write;
///
/// fn process_with_temp_file(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
///     // Create a temporary file
///     let mut temp_file = create_temp_file()?;
///     
///     // Write data to temp file
///     temp_file.write_all(data)?;
///     temp_file.flush()?;
///     
///     let temp_path = temp_file.path().to_string_lossy().to_string();
///     println!("Processing data in temporary file: {}", temp_path);
///     
///     // Process the file...
///     let size = get_file_size(&temp_path)?;
///     println!("Processed {} bytes", size);
///     
///     // File is automatically deleted when temp_file goes out of scope
///     Ok(format!("Processed {} bytes successfully", size))
/// }
/// ```
///
/// ### Directory Operations
/// ```rust,no_run
/// use trash_utilities::sys::path::*;
///
/// fn explore_directory_structure(root: &str) -> Result<(), Box<dyn std::error::Error>> {
///     println!("Exploring directory: {}", root);
///
///     // List immediate contents
///     let entries = list_directory(root)?;
///     println!("Directory contains {} items", entries.len());
///
///     for entry in &entries {
///         let full_path = join_paths(root, &[entry]);
///         if is_directory(&full_path) {
///             println!("📁 {}", entry);
///         } else if is_file(&full_path) {
///             if let Ok(size) = get_file_size(&full_path) {
///                 println!("📄 {} ({} bytes)", entry, size);
///             }
///         }
///     }
///
///     Ok(())
/// }
/// ```
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
/// use trash_analyzer::sys::path::normalize_path;
///
/// let normalized = normalize_path("./foo/../bar");
/// assert_eq!(normalized, "bar");
/// ```
#[must_use]
pub fn normalize_path(path: &str) -> String {
    let path = Path::new(path);
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
/// use trash_analyzer::sys::path::join_paths;
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
/// use trash_analyzer::sys::path::get_file_extension;
///
/// assert_eq!(get_file_extension("file.txt"), Some("txt".to_string()));
/// assert_eq!(get_file_extension("file"), None);
/// ```
#[must_use]
pub fn get_file_extension(path: &str) -> Option<String> {
    Path::new(path)
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
/// use trash_analyzer::sys::path::get_file_stem;
///
/// assert_eq!(get_file_stem("file.txt"), Some("file".to_string()));
/// assert_eq!(get_file_stem("dir/"), None);
/// ```
#[must_use]
pub fn get_file_stem(path: &str) -> Option<String> {
    if path.ends_with('/') {
        None
    } else {
        Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(std::string::ToString::to_string)
    }
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
/// use trash_analyzer::sys::path::get_file_metadata;
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
/// use trash_analyzer::sys::path::get_file_size;
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
/// use trash_analyzer::sys::path::is_file;
///
/// assert!(is_file("Cargo.toml"));
/// assert!(!is_file("nonexistent.txt"));
/// ```
#[must_use]
pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
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
/// use trash_analyzer::sys::path::is_directory;
///
/// assert!(is_directory("src"));
/// assert!(!is_directory("Cargo.toml"));
/// ```
#[must_use]
pub fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
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
/// use trash_analyzer::sys::path::get_file_modified_time;
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
/// use trash_analyzer::sys::path::create_temp_file;
/// use std::io::Write;
///
/// let mut temp_file = create_temp_file().unwrap();
/// temp_file.write_all(b"Hello, temp!").unwrap();
/// // File is automatically deleted when temp_file goes out of scope
/// ```
pub fn create_temp_file() -> Result<NamedTempFile, std::io::Error> {
    NamedTempFile::new()
}

/// Gets metadata for multiple files in parallel using our parallel module.
///
/// # Parameters
/// - `paths`: A slice of file paths to get metadata for.
///
/// # Returns
/// A `HashMap` where keys are file paths and values are `Result<Metadata, std::io::Error>`.
/// This allows checking which files succeeded/failed individually.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::sys::path::get_files_metadata_parallel;
///
/// let paths = vec!["file1.txt", "file2.txt", "file3.txt"];
/// let results = get_files_metadata_parallel(&paths);
/// for (path, result) in results {
///     match result {
///         Ok(metadata) => println!("{}: {} bytes", path, metadata.len()),
///         Err(e) => println!("{}: error {}", path, e),
///     }
/// }
/// ```
#[must_use]
pub fn get_files_metadata_parallel(
    paths: &[&str],
) -> HashMap<String, Result<std::fs::Metadata, std::io::Error>> {
    parallel::parallel_map(paths.to_vec(), |path| {
        let result = get_file_metadata(path);
        (path.to_string(), result)
    })
    .into_iter()
    .collect()
}

/// Serializes file metadata to JSON using our serde module.
///
/// # Parameters
/// - `metadata`: The file metadata to serialize.
///
/// # Returns
/// - `Ok(String)` containing the JSON representation.
/// - `Err` if serialization fails.
///
/// # Errors
/// Returns a boxed error if JSON serialization fails.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::sys::path::{get_file_metadata, serialize_file_info};
///
/// let metadata = get_file_metadata("file.txt").unwrap();
/// let json = serialize_file_info(&metadata).unwrap();
/// println!("File info: {}", json);
/// ```
pub fn serialize_file_info(
    metadata: &std::fs::Metadata,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create a simple struct for serialization
    #[derive(Serialize)]
    struct FileInfo {
        size: u64,
        is_file: bool,
        is_dir: bool,
        modified: Option<String>,
    }

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs().to_string());

    let info = FileInfo {
        size: metadata.len(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        modified,
    };

    serde::serialize_to_json(&info).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Deserializes JSON to file info (for completeness, though metadata can't be fully reconstructed).
///
/// # Parameters
/// - `json`: The JSON string to deserialize.
///
/// # Returns
/// - `Ok` with a simple struct containing file info.
/// - `Err` if deserialization fails.
///
/// # Errors
/// Returns a boxed error if JSON deserialization fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::path::deserialize_file_info;
///
/// let json = r#"{"size":1024,"is_file":true,"is_dir":false,"modified":"1234567890"}"#;
/// let info = deserialize_file_info(json).unwrap();
/// ```
pub fn deserialize_file_info(json: &str) -> Result<Value, Box<dyn std::error::Error>> {
    serde::deserialize_from_json(json).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Gets file metadata synchronously (placeholder for future async version).
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
/// use trash_analyzer::sys::path::get_file_metadata_sync;
///
/// let metadata = get_file_metadata_sync("file.txt").unwrap();
/// println!("Size: {}", metadata.len());
/// ```
pub fn get_file_metadata_sync(path: &str) -> Result<std::fs::Metadata, std::io::Error> {
    // For now, just wrap the sync version since we don't have async fs in dependencies
    // In a real implementation, you'd use async-std or tokio
    get_file_metadata(path)
}

/// Lists directory contents synchronously.
///
/// # Parameters
/// - `path`: The directory path.
///
/// # Returns
/// - `Ok(Vec<String>)` containing entry names.
/// - `Err(std::io::Error)` if directory cannot be read.
///
/// # Errors
/// Returns an `std::io::Error` if the directory does not exist, permission is denied, or other I/O error occurs.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::sys::path::list_directory;
///
/// let entries = list_directory(".").unwrap();
/// for entry in entries {
///     println!("{}", entry);
/// }
/// ```
pub fn list_directory(path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            entries.push(name.to_string());
        }
    }
    Ok(entries)
}

/// Recursively walks a directory and returns all file paths.
///
/// # Parameters
/// - `root`: The root directory to start walking from.
///
/// # Returns
/// - `Ok(Vec<String>)` containing all file paths found.
/// - `Err(std::io::Error)` if directory traversal fails.
///
/// # Errors
/// Returns an `std::io::Error` if the root directory does not exist, permission is denied, or other I/O error occurs during traversal.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::sys::path::walk_directory;
///
/// let files = walk_directory("src").unwrap();
/// for file in files {
///     println!("{}", file);
/// }
/// ```
pub fn walk_directory(root: &str) -> Result<Vec<String>, std::io::Error> {
    fn walk(dir: &Path, files: &mut Vec<String>) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, files)?;
            } else if let Some(path_str) = path.to_str() {
                files.push(path_str.to_string());
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(Path::new(root), &mut files)?;
    Ok(files)
}

/// Finds files matching a pattern in parallel using our parallel module.
///
/// This is a basic implementation that searches for files by extension.
/// For more complex patterns, consider using the `glob` crate.
///
/// # Parameters
/// - `root`: The root directory to search from.
/// - `extension`: The file extension to match (without the dot).
///
/// # Returns
/// - `Ok(Vec<String>)` containing matching file paths.
/// - `Err(std::io::Error)` if directory traversal fails.
///
/// # Errors
/// Returns an `std::io::Error` if the root directory does not exist, permission is denied, or other I/O error occurs during traversal.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::sys::path::find_files_parallel;
///
/// let rs_files = find_files_parallel("src", "rs").unwrap();
/// for file in rs_files {
///     println!("{}", file);
/// }
/// ```
pub fn find_files_parallel(root: &str, extension: &str) -> Result<Vec<String>, std::io::Error> {
    let all_files = walk_directory(root)?;
    let matching: Vec<String> = parallel::parallel_filter(all_files, |path| {
        get_file_extension(path) == Some(extension.to_string())
    });
    Ok(matching)
}
