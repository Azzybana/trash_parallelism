//! Data processing utilities for parsing, encoding, and serialization.
//!
//! This module provides functions for parsing various data formats,
//! encoding/decoding operations, and JSON manipulation utilities.
//!
//! ## Features
//!
//! - **Configuration Parsing**: Key-value pair parsing from config files
//! - **Data Validation**: Structure and format validation
//! - **Efficient Parsing**: High-performance parsing using optimized algorithms
//! - **Memory Efficient**: Low-overhead data processing
//! - **Thread Safe**: All operations are safe to use across threads
//!
//! ## Examples
//!
//! ### Configuration File Parsing
//! ```rust
//! use trash_utilities::data::parse_key_value;
//! use ahash::AHashMap;
//!
//! // Parse a simple config file format
//! let config_content = r#"
//! app.name=MyApp
//! app.version=1.0.0
//! database.host=localhost
//! database.port=5432
//! features.logging=true
//! features.metrics=false
//! "#;
//!
//! let config = parse_key_value(config_content);
//!
//! assert_eq!(config.get("app.name"), Some(&"MyApp".to_string()));
//! assert_eq!(config.get("database.port"), Some(&"5432".to_string()));
//! assert_eq!(config.get("features.logging"), Some(&"true".to_string()));
//! ```
//!
//! ### Environment Variable Style Parsing
//! ```rust
//! use trash_utilities::data::parse_key_value;
//!
//! // Parse environment variable style content
//! let env_content = "
//! HOME=/home/user
//! PATH=/usr/bin:/bin
//! USER=testuser
//! SHELL=/bin/bash
//! ";
//!
//! let env_vars = parse_key_value(env_content);
//! assert_eq!(env_vars.get("USER"), Some(&"testuser".to_string()));
//! assert_eq!(env_vars.get("SHELL"), Some(&"/bin/bash".to_string()));
//! ```
//!
//! ### Docker Compose Style Parsing
//! ```rust
//! use trash_utilities::data::parse_key_value;
//!
//! // Parse Docker environment file format
//! let docker_env = "
//! POSTGRES_DB=myapp
//! POSTGRES_USER=admin
//! POSTGRES_PASSWORD=secret123
//! DATABASE_URL=postgresql://admin:secret123@localhost:5432/myapp
//! ";
//!
//! let db_config = parse_key_value(docker_env);
//!
//! // Use the parsed configuration
//! let db_url = db_config.get("DATABASE_URL").unwrap();
//! println!("Connecting to: {}", db_url);
//! ```
//!
//! ### Configuration Validation
//! ```rust
//! use trash_utilities::data::parse_key_value;
//!
//! fn validate_config(content: &str) -> Result<(), String> {
//!     let config = parse_key_value(content);
//!
//!     // Check required fields
//!     let required_fields = ["app.name", "database.host", "database.port"];
//!     for field in &required_fields {
//!         if !config.contains_key(*field) {
//!             return Err(format!("Missing required field: {}", field));
//!         }
//!     }
//!
//!     // Validate port is numeric
//!     if let Some(port_str) = config.get("database.port") {
//!         if port_str.parse::<u16>().is_err() {
//!             return Err("database.port must be a valid port number".to_string());
//!         }
//!     }
//!
//!     Ok(())
//! }
//!
//! let valid_config = "
//! app.name=MyApp
//! database.host=localhost
//! database.port=5432
//! ";
//!
//! assert!(validate_config(valid_config).is_ok());
//!
//! let invalid_config = "
//! app.name=MyApp
//! database.host=localhost
//! ";
//!
//! assert!(validate_config(invalid_config).is_err());
//! ```

// Standard library imports
// (none)

// External crate imports
use ahash::AHashMap;
use memchr::memchr;

/// Parses key-value pairs from a string content, typically from config files.
///
/// This function assumes each line is in the format "key=value". It uses `memchr`
/// for efficient byte-level searching of the '=' character. Lines without '=' are ignored.
/// Leading and trailing whitespace is trimmed from both keys and values.
///
/// # Parameters
/// - `content`: The string content to parse. Can contain multiple lines.
///
/// # Returns
/// An `AHashMap<String, String>` containing the parsed key-value pairs.
/// Keys and values are trimmed of whitespace. Lines without '=' are silently ignored.
///
/// # Performance
/// Uses `memchr` for O(n) byte-level searching, making it very fast for large config files.
///
/// # Examples
/// ```rust
/// use trash_analyzer::data::parse_key_value;
/// use ahash::AHashMap;
///
/// let content = "name=John\nage=30\ncity=New York";
/// let map = parse_key_value(content);
/// assert_eq!(map.get("name"), Some(&"John".to_string()));
/// assert_eq!(map.get("age"), Some(&"30".to_string()));
/// assert_eq!(map.get("city"), Some(&"New York".to_string()));
/// ```
///
/// # Format
/// The expected format is simple: `key=value` per line. Empty lines and lines without
/// '=' are ignored. Keys and values are trimmed of whitespace.
///
/// ```text
/// # This is a comment (ignored)
/// name=John Doe
/// age=30
/// city=New York
/// ```
#[must_use]
pub fn parse_key_value(content: &str) -> AHashMap<String, String> {
    let mut map = AHashMap::new();
    for line in content.lines() {
        if let Some(pos) = memchr(b'=', line.as_bytes()) {
            let key = &line[..pos];
            let value = &line[pos + 1..];
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}
