//! # Environment Variable Utilities
//!
//! This module provides comprehensive environment variable handling with type-safe parsing,
//! JSON configuration support via our serde module, and parallel processing capabilities.
//!
//! ## Overview
//!
//! The `env` module offers:
//! - **Basic Environment Access**: Reading variables with defaults and type parsing
//! - **JSON Configuration**: Parse complex JSON configs from environment variables
//! - **Batch Processing**: Parallel reading of multiple environment variables
//! - **Validation**: Check for required environment variables
//!
//! ## Usage Patterns
//!
//! ### Basic Environment Variable Access
//! ```rust
//! use trash_analyzer::sys::env::*;
//!
//! let home = read_env_var("HOME").unwrap();
//! let port: u16 = read_env_var_parse("PORT").unwrap_or(8080);
//! let debug = read_env_var_or("DEBUG", "false");
//! ```
//!
//! ### JSON Configuration from Environment
//! ```rust
//! use trash_analyzer::sys::env::read_env_var_json;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct Config { host: String, port: u16 }
//!
//! let config: Config = read_env_var_json("APP_CONFIG").unwrap();
//! ```
//!
//! ### Batch Processing
//! ```rust
//! use trash_analyzer::sys::env::read_env_vars_parallel;
//!
//! let keys = vec!["HOME", "USER", "PATH"];
//! let values = read_env_vars_parallel(&keys).unwrap();
//! ```

use std::collections::HashMap;
use crate::parallel;
use crate::serde;
use ::serde::de::DeserializeOwned;

/// Reads an environment variable as a string.
///
/// # Parameters
/// - `key`: The name of the environment variable.
///
/// # Returns
/// - `Ok(String)` containing the value if the variable exists.
/// - `Err(std::env::VarError)` if the variable is not set or invalid.
///
/// # Errors
/// Returns a `std::env::VarError` if the environment variable is not set or contains invalid Unicode.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::env::read_env_var;
///
/// let home = read_env_var("HOME").unwrap();
/// println!("Home directory: {}", home);
/// ```
pub fn read_env_var(key: &str) -> Result<String, std::env::VarError> {
    std::env::var(key)
}

/// Reads an environment variable with a default value.
///
/// # Parameters
/// - `key`: The name of the environment variable.
/// - `default`: The default value to return if the variable is not set.
///
/// # Returns
/// The value of the environment variable or the default.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::env::read_env_var_or;
///
/// let port = read_env_var_or("PORT", "8080");
/// println!("Port: {}", port);
/// ```
#[must_use]
pub fn read_env_var_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Reads an environment variable and parses it as a type that implements `FromStr`.
///
/// # Type Parameters
/// - `T`: The type to parse the value into, must implement `FromStr`.
///
/// # Parameters
/// - `key`: The name of the environment variable.
///
/// # Returns
/// - `Ok(T)` if parsing succeeds.
/// - `Err` if the variable is not set or parsing fails.
///
/// # Errors
/// Returns an error if the environment variable is not set, or if parsing the value fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::env::read_env_var_parse;
///
/// let port: u16 = read_env_var_parse("PORT").unwrap_or(8080);
/// println!("Port: {}", port);
/// ```
pub fn read_env_var_parse<T>(key: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + 'static,
{
    let value = std::env::var(key)?;
    Ok(value.parse()?)
}

/// Reads an environment variable containing JSON and deserializes it to a type using our serde module.
///
/// # Type Parameters
/// - `T`: The type to deserialize into, must implement `Deserialize`.
///
/// # Parameters
/// - `key`: The name of the environment variable containing JSON.
///
/// # Returns
/// - `Ok(T)` if the variable exists and JSON parsing succeeds.
/// - `Err` if the variable is not set or parsing fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::env::read_env_var_json;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config { host: String, port: u16 }
///
/// let config: Config = read_env_var_json("APP_CONFIG").unwrap();
/// ```
pub fn read_env_var_json<T>(key: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    let json = std::env::var(key)?;
    serde::deserialize_from_json(&json).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Reads multiple environment variables in parallel using our parallel module.
///
/// This function processes a collection of environment variable keys concurrently,
/// which can improve performance when reading many variables.
///
/// # Parameters
/// - `keys`: A slice of environment variable names to read.
///
/// # Returns
/// A `HashMap` where keys are the variable names and values are the variable values.
/// Variables that are not set or contain errors are omitted from the result.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::env::read_env_vars_parallel;
///
/// let keys = vec!["HOME", "USER", "PATH"];
/// let vars = read_env_vars_parallel(&keys);
/// for (key, value) in vars {
///     println!("{}: {}", key, value);
/// }
/// ```
#[must_use]
pub fn read_env_vars_parallel(keys: &[&str]) -> HashMap<String, String> {
    parallel::parallel_map(keys.to_vec(), |key| {
        std::env::var(key).ok().map(|value| (key.to_string(), value))
    })
    .into_iter()
    .flatten()
    .collect()
}

/// Validates that all required environment variables are set.
///
/// # Parameters
/// - `required_keys`: A slice of environment variable names that must be set.
///
/// # Returns
/// - `Ok(())` if all required variables are set.
/// - `Err(Vec<String>)` containing the names of missing variables.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::env::validate_required_env_vars;
///
/// let required = vec!["HOME", "USER"];
/// match validate_required_env_vars(&required) {
///     Ok(()) => println!("All required env vars are set"),
///     Err(missing) => println!("Missing: {:?}", missing),
/// }
/// ```
pub fn validate_required_env_vars(required_keys: &[&str]) -> Result<(), Vec<String>> {
    let missing: Vec<String> = required_keys
        .iter()
        .filter_map(|key| std::env::var(key).err().map(|_| key.to_string()))
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}