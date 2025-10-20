//! I/O utilities for asynchronous file operations, channels, compression, and caching.
//!
//! This module provides high-performance async I/O operations using smol,
//! compression algorithms (Brotli, Gzip), cryptographic hashing, and thread-safe
//! data structures for caching and concurrency.
//!
//! ## Submodules
//!
//! - [`utils`]: Basic file operations, compression, counters, and caching
//! - [`parallelism`]: Parallel file processing using existing parallel module
//! - [`writers`]: Async file writers with buffering and compression
//! - [`streams`]: Streaming I/O operations and channel utilities
//!
//! ## Architecture
//!
//! The I/O module is organized into focused submodules to avoid code duplication
//! and maintain clear separation of concerns. Each submodule leverages existing
//! crate modules (parallel, channels, async, serde, memory) rather than
//! reimplementing functionality.

// Submodule declarations
pub mod parallelism;
pub mod streams;
pub mod utils;
pub mod writers;

// Re-exports for backward compatibility and convenience
pub use parallelism::*;
pub use streams::*;
pub use utils::*;
pub use writers::*;
