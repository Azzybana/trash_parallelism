/// Common utilities shared across multiple modules for code reuse and maintainability.
///
/// This module centralizes frequently used utility functions and types that are
/// duplicated across the codebase. By providing these in a single location, we
/// reduce code duplication, improve maintainability, and ensure consistency.
///
/// ## Features
///
/// - **Cryptographic Operations**: Hashing, encoding, and security utilities
/// - **JSON Utilities**: Common JSON manipulation and processing functions
/// - **General Utilities**: Time operations, thread-safe data structures, and helpers
/// - **Performance Optimized**: Efficient implementations for common operations
/// - **Thread Safe**: All utilities are safe to use across thread boundaries
///
/// ## Architecture
///
/// ### Core Components
/// - **`crypto`**: Cryptographic hashing, encoding, and security functions
/// - **`json`**: JSON manipulation, validation, and utility functions
/// - **`utils`**: General-purpose utilities including time, threading, and data structures
///
/// ## Examples
///
/// ### Time Operations
/// ```rust
/// use trash_utilities::common::utils::*;
///
/// // Current UTC time
/// let now = current_utc_time();
/// let formatted = format_datetime(&now);
/// println!("Current time: {}", formatted);
///
/// // Date parsing
/// let date = parse_date("2023-12-25").unwrap();
/// let datetime = parse_datetime("2023-12-25T00:00:00Z").unwrap();
/// ```
///
/// ### Thread-Safe Data Structures
/// ```rust
/// use trash_utilities::common::utils::*;
///
/// // Atomic counter for thread-safe counting
/// let counter = AtomicCounter::new();
/// assert_eq!(counter.increment(), 1);
/// assert_eq!(counter.get(), 1);
/// counter.reset();
/// assert_eq!(counter.get(), 0);
///
/// // String interning for memory efficiency
/// let interner = StringInterner::new();
/// let s1 = interner.intern("frequently_used");
/// let s2 = interner.intern("frequently_used");
/// assert_eq!(s1.as_ptr(), s2.as_ptr()); // Same memory location
/// assert_eq!(interner.len(), 1);
///
/// // LRU cache with size limits
/// let cache = LruCache::new(100);
/// cache.insert("key1", "value1");
/// cache.insert("key2", "value2");
/// assert_eq!(cache.get(&"key1"), Some("value1"));
/// assert_eq!(cache.len(), 2);
/// ```
///
/// ### Cryptographic Operations
/// ```rust
/// use trash_utilities::common::crypto::*;
///
/// // Fast non-cryptographic hashing
/// let hash = hash_data_fast(b"some data");
/// println!("Hash: {}", hash);
///
/// // Base64 encoding/decoding
/// let data = b"Hello, World!";
/// let encoded = encode_base64(data);
/// let decoded = decode_base64(&encoded).unwrap();
/// assert_eq!(data.to_vec(), decoded);
/// ```
///
/// ### JSON Utilities
/// ```rust
/// use trash_utilities::common::json::*;
///
/// // JSON validation
/// let json_str = r#"{"name": "Alice", "age": 30}"#;
/// assert!(validate_json_structure(json_str));
///
/// // JSON key extraction
/// let value = extract_json_value(json_str, "name");
/// assert_eq!(value, Some(r#""Alice""#.to_string()));
///
/// // JSON key existence checking
/// assert!(json_has_key(json_str, "name"));
/// assert!(!json_has_key(json_str, "email"));
/// ```
///
/// ### Performance Monitoring
/// ```rust
/// use trash_utilities::common::utils::*;
///
/// // Performance timer with automatic reporting
/// {
///     let _timer = Timer::new("expensive_operation");
///     // ... expensive operation ...
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// } // Timer automatically reports duration when dropped
/// // Output: Timer 'expensive_operation' completed in 10ms
/// ```
///
/// ### Memory Management Helpers
/// ```rust
/// use trash_utilities::common::utils::*;
///
/// // Efficient string building
/// let mut builder = EfficientStringBuilder::new();
/// builder.append("Hello");
/// builder.append(", ");
/// builder.append("world!");
/// let result = builder.build();
/// assert_eq!(result, "Hello, world!");
///
/// // Memory pool allocation (if available)
/// // let ptr = alloc_from_pool("default", 1024);
/// ```
// Submodule declarations
pub mod crypto;
pub mod json;
pub mod utils;

// Re-exports for backward compatibility
pub use crypto::*;
pub use json::*;
pub use utils::*;
