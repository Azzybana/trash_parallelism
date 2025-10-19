/// System utilities for time handling, environment variables, file system operations, and system interactions.
///
/// This module provides convenient wrappers for system-level operations including
/// time manipulation, environment variable access, path operations, and file metadata.
///
/// ## Submodules
///
/// - [`datetime`]: Date and time utilities with serialization and parallel processing
/// - [`env`]: Environment variable handling with JSON parsing and validation
/// - [`path`]: File system operations with parallel and async capabilities
///
/// ## Backward Compatibility
///
/// All functions are re-exported at the module level for backward compatibility.
/// You can access them directly as `trash_analyzer::sys::function_name()` or
/// use the submodules for more organized access.
// Standard library imports
use std::time::{Duration, Instant};

// Submodule declarations
pub mod datetime;
pub mod env;
pub mod path;

// Re-exports for backward compatibility
pub use datetime::*;
pub use env::*;
pub use path::*;

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
        let duration = self.start.elapsed();
        println!("Timer '{}' completed in {:?}", self.label, duration);
    }
}
