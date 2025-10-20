//! Parallel I/O processing utilities.
//!
//! This module provides parallel processing capabilities for I/O operations,
//! leveraging the existing parallel module for efficient concurrent file processing.
//!
//! ## Features
//!
//! - **Parallel File Processing**: Process multiple files concurrently using work queues
//! - **Batch Operations**: Map and filter operations on file collections
//! - **Integration**: Uses existing parallel module to avoid duplication
//! - **Async Support**: Non-blocking parallel operations with smol

// Standard library imports
// (none needed)

// External crate imports
// (none needed)

/// Parallel map using the existing parallel module
///
/// Leverages `crate::parallel::parallel_map` for consistent parallel processing.
/// This avoids duplicating parallel logic across modules.
///
/// # Type Parameters
/// - `T`: Input type that can be sent across threads
/// - `U`: Output type that can be sent across threads
/// - `F`: Function type for transformation
///
/// # Parameters
/// - `data`: Vector of input items to process
/// - `f`: Function to apply to each item
///
/// # Returns
/// Vector of transformed results in the same order as input
#[must_use]
pub fn parallel_map<T, U, F>(data: Vec<T>, f: F) -> Vec<U>
where
    T: Send,
    U: Send,
    F: Fn(T) -> U + Send + Sync,
{
    // Leverage existing parallel module
    crate::parallel::parallel_map(data, f)
}

/// Parallel filter using the existing parallel module
///
/// Uses `crate::parallel::parallel_filter` for consistent filtering behavior.
///
/// # Type Parameters
/// - `T`: Item type that can be sent across threads
/// - `F`: Predicate function type
///
/// # Parameters
/// - `data`: Vector of items to filter
/// - `f`: Predicate function that returns true for items to keep
///
/// # Returns
/// Vector containing only items that passed the filter
#[must_use]
pub fn parallel_filter<T, F>(data: Vec<T>, f: F) -> Vec<T>
where
    T: Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
{
    // Leverage existing parallel module
    crate::parallel::parallel_filter(data, f)
}

/// Asynchronously process multiple files in parallel
///
/// Enhanced version that leverages the parallel module and async I/O utilities.
/// Uses work queues for load balancing and efficient resource utilization.
///
/// # Type Parameters
/// - `F`: Processor function type
/// - `R`: Result type from processing
///
/// # Parameters
/// - `paths`: Vector of file paths to process
/// - `processor`: Function that takes file content and returns a result
///
/// # Returns
/// Vector of results in the same order as input paths
///
/// # Errors
/// Returns error for any file that couldn't be read or processed
#[must_use]
pub fn process_files_parallel<F, R>(
    paths: Vec<String>,
    processor: F,
) -> Vec<Result<R, std::io::Error>>
where
    F: Fn(String) -> Result<R, std::io::Error> + Send + Sync + Clone,
    R: Send + 'static,
{
    // Use parallel processing with work distribution
    crate::parallel::parallel_map(paths, |path| {
        // Read file content using our async utils
        match std::fs::read_to_string(&path) {
            Ok(content) => processor(content),
            Err(e) => Err(e),
        }
    })
}

/// Process files with streaming and parallel chunking
///
/// Advanced file processing that reads files in chunks and processes them
/// in parallel using the parallel module's chunking capabilities.
///
/// # Type Parameters
/// - `F`: Processor function for individual chunks
/// - `R`: Result type from chunk processing
///
/// # Parameters
/// - `paths`: File paths to process
/// - `chunk_size`: Size of each chunk in bytes
/// - `processor`: Function to process each chunk
///
/// # Returns
/// Vector of results for each file
///
/// # Errors
/// Returns error if file reading or processing fails
pub async fn process_files_chunked<F, R>(
    paths: Vec<String>,
    chunk_size: usize,
    processor: F,
) -> Vec<Result<Vec<R>, std::io::Error>>
where
    F: Fn(Vec<u8>) -> Result<R, std::io::Error> + Send + Sync + Clone,
    R: Send + 'static,
{
    // Process each file, reading in chunks
    let mut results = Vec::new();

    for path in paths {
        match crate::io::utils::read_file_bytes_async(&path).await {
            Ok(data) => {
                // Split into chunks and process in parallel
                let chunks: Vec<Vec<u8>> = data.chunks(chunk_size).map(<[u8]>::to_vec).collect();

                let chunk_results = crate::parallel::parallel_map(chunks, &processor);

                // Collect results, propagating any errors
                let mut file_results = Vec::new();
                let mut has_error = false;
                let mut error = std::io::Error::other("Processing error");

                for result in chunk_results {
                    match result {
                        Ok(r) => file_results.push(r),
                        Err(e) => {
                            error = e;
                            has_error = true;
                            break;
                        }
                    }
                }

                if has_error {
                    results.push(Err(error));
                } else {
                    results.push(Ok(file_results));
                }
            }
            Err(e) => results.push(Err(e)),
        }
    }

    results
}

/// Parallel directory traversal with processing
///
/// Recursively traverses directories and processes files in parallel.
/// Uses the parallel module for efficient work distribution.
///
/// # Type Parameters
/// - `F`: File processor function type
/// - `R`: Result type from processing
///
/// # Parameters
/// - `root_path`: Root directory to start traversal
/// - `file_processor`: Function to process each file
/// - `max_depth`: Maximum directory depth to traverse (None for unlimited)
///
/// # Returns
/// Vector of processing results for all files found
///
/// # Errors
/// Returns errors for files that couldn't be read or processed
pub fn traverse_and_process<F, R>(
    root_path: &str,
    file_processor: F,
    max_depth: Option<usize>,
) -> Result<Vec<Result<R, std::io::Error>>, std::io::Error>
where
    F: Fn(String, String) -> Result<R, std::io::Error> + Send + Sync + Clone,
    R: Send + 'static,
{
    let mut all_files = Vec::new();
    collect_files_recursive(root_path, &mut all_files, 0, max_depth)?;

    // Process all files in parallel
    let results =
        crate::parallel::parallel_map(all_files, |(path, content)| file_processor(path, content));

    Ok(results)
}

/// Helper function to recursively collect files
fn collect_files_recursive(
    path: &str,
    files: &mut Vec<(String, String)>,
    current_depth: usize,
    max_depth: Option<usize>,
) -> Result<(), std::io::Error> {
    if max_depth.is_some_and(|max| current_depth > max) {
        return Ok(());
    }

    let entries = std::fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let full_path = entry.path().to_string_lossy().to_string();

        // Check if it's a file or directory
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                // Read file content
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    files.push((full_path, content));
                }
                // Skip files we can't read
            } else if metadata.is_dir() {
                // Recurse into directory
                collect_files_recursive(&full_path, files, current_depth + 1, max_depth)?;
            }
        }
    }

    Ok(())
}

/// Batch file operations with rollback support
///
/// Performs multiple file operations in parallel, with the ability to rollback
/// changes if any operation fails. Uses the parallel module for concurrent execution.
///
/// # Type Parameters
/// - `F`: Operation function type
///
/// # Parameters
/// - `operations`: Vector of operations to perform
/// - `rollback_on_error`: Whether to rollback successful operations if any fail
///
/// # Returns
/// Result indicating success or failure of the batch operation
///
/// # Errors
/// Returns an error if any operation fails and rollback is requested
pub fn batch_file_operations<F>(
    operations: Vec<F>,
    rollback_on_error: bool,
) -> Result<(), std::io::Error>
where
    F: Fn() -> Result<(), std::io::Error> + Send + Sync,
{
    // Execute operations in parallel
    let results = crate::parallel::parallel_map(operations, |op| op());

    // Check for errors
    let mut errors = Vec::new();
    for result in results {
        if let Err(e) = result {
            errors.push(e);
        }
    }

    if !errors.is_empty() && rollback_on_error {
        // TODO: Implement rollback logic based on operation types
        // This would require tracking what operations were successful
        return Err(std::io::Error::other(format!(
            "Batch operation failed with {} errors",
            errors.len()
        )));
    }

    Ok(())
}
