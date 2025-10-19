//! # `DateTime` Utilities
//!
//! This module provides comprehensive date and time utilities with chrono integration,
//! including serialization support via our serde module and parallel processing capabilities.
//!
//! ## Overview
//!
//! The `datetime` module offers:
//! - **Basic Time Operations**: Current time, formatting, parsing
//! - **Serialization Integration**: JSON serialization for timestamps
//! - **Batch Processing**: Parallel parsing of multiple dates
//! - **Timezone Utilities**: Basic timezone conversion helpers
//!
//! ## Usage Patterns
//!
//! ### Basic `DateTime` Operations
//! ```rust
//! use trash_analyzer::sys::datetime::*;
//!
//! let now = current_utc_time();
//! let formatted = format_datetime(&now);
//! let parsed = parse_datetime(&formatted).unwrap();
//! ```
//!
//! ### Serialization with Serde
//! ```rust
//! use trash_analyzer::sys::datetime::{serialize_timestamp, deserialize_timestamp};
//!
//! let timestamp = current_utc_time();
//! let json = serialize_timestamp(&timestamp).unwrap();
//! let restored = deserialize_timestamp(&json).unwrap();
//! ```
//!
//! ### Batch Processing
//! ```rust
//! use trash_analyzer::sys::datetime::batch_parse_dates;
//!
//! let date_strings = vec!["2023-01-01", "2023-01-02", "2023-01-03"];
//! let dates = batch_parse_dates(&date_strings).unwrap();
//! ```

use crate::parallel;
use crate::serde;
use chrono::{DateTime, NaiveDate, Utc};

/// Gets the current UTC time.
///
/// This function returns the current time in UTC using `chrono::Utc::now()`.
///
/// # Returns
/// A `DateTime<Utc>` representing the current UTC time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::datetime::current_utc_time;
/// use chrono::{DateTime, Utc};
///
/// let now: DateTime<Utc> = current_utc_time();
/// println!("Current time: {}", now);
/// ```
#[must_use]
pub fn current_utc_time() -> DateTime<Utc> {
    Utc::now()
}

/// Formats a `DateTime<Utc>` to an ISO 8601 string.
///
/// This function uses RFC 3339 format, which is a profile of ISO 8601.
///
/// # Parameters
/// - `dt`: The `DateTime<Utc>` to format.
///
/// # Returns
/// A `String` containing the formatted date and time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::datetime::{current_utc_time, format_datetime};
/// use chrono::DateTime;
///
/// let now = current_utc_time();
/// let formatted = format_datetime(&now);
/// println!("Formatted time: {}", formatted);
/// ```
#[must_use]
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Parses a date/time string in RFC 3339 format.
///
/// # Parameters
/// - `s`: The string to parse.
///
/// # Returns
/// - `Ok(DateTime<Utc>)` if parsing succeeds.
/// - `Err(chrono::ParseError)` if parsing fails.
///
/// # Errors
/// Returns a `chrono::ParseError` if the string is not a valid RFC 3339 date/time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::datetime::parse_datetime;
///
/// let dt = parse_datetime("2023-01-01T12:00:00Z").unwrap();
/// println!("Parsed datetime: {}", dt);
/// ```
pub fn parse_datetime(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
}

/// Parses a date string in YYYY-MM-DD format.
///
/// # Parameters
/// - `s`: The string to parse.
///
/// # Returns
/// - `Ok(NaiveDate)` if parsing succeeds.
/// - `Err(chrono::ParseError)` if parsing fails.
///
/// # Errors
/// Returns a `chrono::ParseError` if the string is not in YYYY-MM-DD format.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::datetime::parse_date;
/// use chrono::NaiveDate;
///
/// let date = parse_date("2023-01-01").unwrap();
/// println!("Parsed date: {}", date);
/// ```
pub fn parse_date(s: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
}

/// Serializes a `DateTime<Utc>` to a JSON string using our serde module.
///
/// # Parameters
/// - `dt`: The `DateTime<Utc>` to serialize.
///
/// # Returns
/// - `Ok(String)` containing the JSON representation.
/// - `Err` if serialization fails.
///
/// # Errors
/// Returns a boxed error if JSON serialization fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::datetime::{current_utc_time, serialize_timestamp};
///
/// let now = current_utc_time();
/// let json = serialize_timestamp(&now).unwrap();
/// println!("Timestamp JSON: {}", json);
/// ```
pub fn serialize_timestamp(dt: &DateTime<Utc>) -> Result<String, Box<dyn std::error::Error>> {
    serde::serialize_to_json(dt).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Deserializes a JSON string to a `DateTime<Utc>` using our serde module.
///
/// # Parameters
/// - `json`: The JSON string to deserialize.
///
/// # Returns
/// - `Ok(DateTime<Utc>)` if deserialization succeeds.
/// - `Err` if deserialization fails.
///
/// # Errors
/// Returns a boxed error if JSON deserialization fails.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::datetime::deserialize_timestamp;
///
/// let json = "\"2023-01-01T12:00:00Z\"";
/// let dt = deserialize_timestamp(json).unwrap();
/// println!("Deserialized datetime: {}", dt);
/// ```
pub fn deserialize_timestamp(json: &str) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    serde::deserialize_from_json(json).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Parses multiple date strings in parallel using our parallel module.
///
/// This function processes a collection of date strings concurrently for improved performance
/// on large datasets.
///
/// # Parameters
/// - `date_strings`: A slice of date strings in YYYY-MM-DD format.
///
/// # Returns
/// - `Ok(Vec<NaiveDate>)` containing parsed dates in the same order.
/// - `Err` if any date parsing fails (returns the first error encountered).
///
/// # Errors
/// Returns a boxed error if any date string cannot be parsed in YYYY-MM-DD format.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::datetime::batch_parse_dates;
///
/// let date_strings = vec!["2023-01-01", "2023-01-02", "2023-01-03"];
/// let dates = batch_parse_dates(&date_strings).unwrap();
/// assert_eq!(dates.len(), 3);
/// ```
pub fn batch_parse_dates(
    date_strings: &[&str],
) -> Result<Vec<NaiveDate>, Box<dyn std::error::Error>> {
    parallel::parallel_map(date_strings.to_vec(), parse_date)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Converts a UTC datetime to a different timezone offset.
///
/// This is a basic utility for timezone conversion. For more complex timezone handling,
/// consider using the `chrono-tz` crate.
///
/// # Parameters
/// - `dt`: The UTC datetime to convert.
/// - `offset_hours`: The timezone offset in hours (e.g., -5 for EST, +9 for JST).
///
/// # Returns
/// A new `DateTime<Utc>` adjusted by the offset.
///
/// # Examples
/// ```rust
/// use trash_analyzer::sys::datetime::{current_utc_time, convert_timezone_offset};
///
/// let utc = current_utc_time();
/// let est = convert_timezone_offset(&utc, -5); // EST is UTC-5
/// println!("UTC: {}, EST: {}", utc, est);
/// ```
#[must_use]
pub fn convert_timezone_offset(dt: &DateTime<Utc>, offset_hours: i32) -> DateTime<Utc> {
    *dt + chrono::Duration::hours(i64::from(offset_hours))
}
