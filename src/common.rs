/// Common utilities shared across multiple modules.
///
/// This module contains frequently used utility functions and types
/// that are duplicated across the codebase. By centralizing them here,
/// we reduce code duplication and improve maintainability.
// Submodule declarations
pub mod crypto;
pub mod json;
pub mod utils;

// Re-exports for backward compatibility
pub use crypto::*;
pub use json::*;
pub use utils::*;
