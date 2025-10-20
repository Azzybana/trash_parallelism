//! # Trash Utilities
//!
//! A high-performance Rust library providing comprehensive async, threading,
//! memory management, and utility functions for building efficient applications.
//!
//! ## Features
//!
//! - **Async Operations**: High-performance async utilities with smol, futures-lite, and crossfire
//! - **Threading**: Parallel processing with `fork_union` and work-stealing schedulers
//! - **Memory Management**: Efficient allocation with mimalloc and custom memory pools
//! - **Channels**: Advanced channel communication with monitoring and serialization
//! - **System Utilities**: Time handling, environment variables, file system operations
//! - **Data Processing**: Parsing, serialization, and data manipulation
//! - **I/O Operations**: File and network I/O with async support
//! - **Logging**: Structured logging
//! - **Utilities**: Compression, hashing, JSON handling, and more
//!
//! ## Quick Start
//!
//! ```rust
//! use trash_utilities::*;
//!
//! // Spawn an async task
//! spawn_task!("my_task", async {
//!     println!("Hello from async task!");
//!     Ok(())
//! });
//!
//! // Parallel processing
//! let data = vec![1, 2, 3, 4, 5];
//! let result = parallel_map(data, |x| x * 2);
//!
//! // Memory operations
//! let ptr = alloc_from_pool!("default", 1024);
//!
//! // System utilities
//! let now = current_utc_time();
//! let home = read_env_var("HOME").unwrap_or_default();
//! ```
//!
//! ## Modules
//!
//! - `async`: Comprehensive async utilities
//! - `channels`: Advanced channel communication
//! - `chars`: Character and string processing
//! - `common`: Shared utility functions
//! - `data`: Data parsing and serialization
//! - `io`: I/O operations
//! - `logging`: Structured logging
//! - `macros`: Ergonomic macros
//! - `memory`: Memory management and pools
//! - `serde`: Serialization utilities
//! - `sys`: System utilities
//! - `parallel`: Threading and parallelism
//! - `utils`: Additional utilities
//!
//! ## Re-exports
//!
//! Key functions and types are re-exported at the crate root for convenience:
//!
//! - `spawn_task!`, `parallel_map!`, `alloc_from_pool!` from macros
//! - `current_utc_time`, `read_env_var` from sys
//! - `parallel_map`, `parallel_for_each` from threads

/// The dynamic version of the crate, computed from commit count during build.
pub const VERSION: &str = env!("DYNAMIC_VERSION");

pub mod r#async;
pub mod channels;
pub mod chars;
pub mod common;
pub mod data;
pub mod io;
pub mod memory;
pub mod parallel;
pub mod serde;
pub mod sys;

// Re-exports
pub use parallel::{parallel_for_each, parallel_map};
pub use sys::{current_utc_time, read_env_var};
