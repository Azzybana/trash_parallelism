/// Comprehensive async utilities with advanced concurrency and processing capabilities.
///
/// This module provides high-performance async operations using smol runtime
/// and crossfire MPMC channels for optimal non-blocking throughput and zero-copy message passing.
// Submodule declarations
pub mod core;
pub mod data;
pub mod patterns;
pub mod tasks;

// Re-exports for backward compatibility
pub use core::*;
pub use data::*;
pub use patterns::*;
pub use tasks::*;
