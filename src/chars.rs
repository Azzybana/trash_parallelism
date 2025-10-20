/// Comprehensive character and string processing utilities with high-performance algorithms.
///
/// This module provides a complete suite of string manipulation, encoding, compression,
/// and processing utilities optimized for performance and memory efficiency. Built using
/// optimized algorithms and data structures for maximum throughput.
///
/// ## Features
///
/// - **High-Performance String Operations**: Fast searching, splitting, and manipulation
/// - **Multiple Encoding Support**: Base64, UTF-8, and custom encodings
/// - **Compression Algorithms**: Efficient string compression and decompression
/// - **Hashing Functions**: Fast non-cryptographic hashing for strings
/// - **String Interning**: Memory-efficient string deduplication
/// - **Parallel Processing**: Concurrent string operations where applicable
/// - **Memory Efficient**: Optimized for low memory overhead
///
/// ## Architecture
///
/// ### Core Components
/// - **`core`**: Fundamental string operations, searching, encoding, and hashing
/// - **`processing`**: Advanced string processing, compression, and parallel operations
///
/// ## Examples
///
/// ### Basic String Operations
/// ```rust
/// use trash_utilities::chars::core::*;
///
/// // Fast byte-level searching
/// let data = b"hello world rust";
/// assert_eq!(find_byte(data, b'w'), Some(6));
/// assert_eq!(find_byte(data, b'z'), None);
///
/// // String encoding and decoding
/// let original = "Hello, 世界!";
/// let encoded = encode_string_base64(original);
/// let decoded = decode_string_base64(&encoded).unwrap();
/// assert_eq!(original, decoded);
///
/// // Efficient string splitting
/// let csv_line = "name,age,city";
/// let fields = split_string_efficient(csv_line, ',');
/// assert_eq!(fields, vec!["name", "age", "city"]);
/// ```
///
/// ### String Hashing and Deduplication
/// ```rust
/// use trash_utilities::chars::core::*;
///
/// // Fast string hashing
/// let hash1 = hash_string_fast("hello");
/// let hash2 = hash_string_fast("world");
/// let hash3 = hash_string_fast("hello"); // Same as hash1
/// assert_eq!(hash1, hash3);
/// assert_ne!(hash1, hash2);
///
/// // String interning for memory efficiency
/// let interner = StringInterner::new();
/// let s1 = interner.intern("frequently_used_string");
/// let s2 = interner.intern("frequently_used_string");
///
/// // Both references point to the same memory location
/// assert_eq!(s1.as_ptr(), s2.as_ptr());
/// assert_eq!(interner.len(), 1); // Only one unique string stored
/// ```
///
/// ### Advanced String Processing
/// ```rust
/// use trash_utilities::chars::processing::*;
///
/// // String compression
/// let text = "This is a long string that can be compressed efficiently.";
/// let compressed = compress_string(&text).unwrap();
/// let decompressed = decompress_string(&compressed).unwrap();
/// assert_eq!(text, decompressed);
///
/// // Calculate compression ratio
/// let ratio = compressed.len() as f64 / text.len() as f64;
/// println!("Compression ratio: {:.2}", ratio);
///
/// // Parallel string processing
/// let texts = vec![
///     "First string".to_string(),
///     "Second string".to_string(),
///     "Third string".to_string(),
/// ];
///
/// let processed = process_strings_parallel(texts, |s| format!("[{}]", s));
/// assert_eq!(processed[0], "[First string]");
/// assert_eq!(processed[1], "[Second string]");
/// ```
///
/// ### Memory-Efficient String Building
/// ```rust
/// use trash_utilities::chars::core::*;
///
/// // Efficient string building with pre-allocated capacity
/// let mut builder = StringBuilder::with_capacity(100);
/// builder.push_str("Hello");
/// builder.push_str(", ");
/// builder.push_str("world!");
///
/// let result = builder.build();
/// assert_eq!(result, "Hello, world!");
///
/// // The builder was pre-allocated, avoiding reallocations
/// assert!(builder.capacity() >= 100);
/// ```
///
/// ### Unicode-Aware Operations
/// ```rust
/// use trash_utilities::chars::core::*;
///
/// // Unicode-safe character counting
/// let text = "Hello, 世界! 👋";
/// let char_count = count_unicode_chars(text);
/// assert_eq!(char_count, 12); // 7 ASCII + 2 Chinese + 1 emoji + 2 punctuation
///
/// // Safe substring extraction
/// let substring = safe_substring(text, 7, 2); // "世界"
/// assert_eq!(substring, "世界");
///
/// // Validate UTF-8
/// let valid_utf8 = "Valid UTF-8 string";
/// let invalid_utf8 = String::from_utf8_lossy(&[0xFF, 0xFE]);
///
/// assert!(is_valid_utf8(valid_utf8));
/// assert!(!is_valid_utf8(&invalid_utf8));
/// ```
// Submodule declarations
pub mod core;
pub mod processing;

// Re-exports for backward compatibility
pub use core::*;
pub use processing::*;
