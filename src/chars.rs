/// Comprehensive character and string processing utilities.
///
/// This module provides high-performance string operations, compression,
/// encoding, hashing, and parallel processing capabilities using the
/// full range of available dependencies for robust asynchronous programming.
// Submodule declarations
pub mod core;
pub mod processing;

// Re-exports for backward compatibility
pub use core::*;
pub use processing::*;
