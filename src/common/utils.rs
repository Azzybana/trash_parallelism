/// Time utilities and thread-safe data structures.
///
/// This module provides date/time operations and concurrent data structures
/// for efficient multi-threaded programming.
// Standard library imports
use std::sync::Arc;

// External crate imports
use ahash::AHashMap;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;

/// Gets the current UTC time.
///
/// This function returns the current time in UTC using `chrono::Utc::now()`.
///
/// # Returns
/// A `DateTime<Utc>` representing the current UTC time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::common::utils::current_utc_time;
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
/// use trash_analyzer::common::utils::{current_utc_time, format_datetime};
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
///
/// Returns a `chrono::ParseError` if the string is not a valid RFC 3339 date/time.
///
/// # Examples
/// ```rust
/// use trash_analyzer::common::utils::parse_datetime;
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
///
/// Returns a `chrono::ParseError` if the string is not a valid date in YYYY-MM-DD format.
///
/// # Examples
/// ```rust
/// use trash_analyzer::common::utils::parse_date;
/// use chrono::NaiveDate;
///
/// let date = parse_date("2023-01-01").unwrap();
/// println!("Parsed date: {}", date);
/// ```
pub fn parse_date(s: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
}

/// Thread-safe counter
#[derive(Debug, Clone)]
pub struct AtomicCounter {
    count: Arc<Mutex<u64>>,
}

impl AtomicCounter {
    /// Create a new counter
    #[must_use]
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    /// Increment and return the new value
    #[must_use]
    pub fn increment(&self) -> u64 {
        let mut count = self.count.lock();
        *count += 1;
        *count
    }

    /// Get current value
    #[must_use]
    pub fn get(&self) -> u64 {
        *self.count.lock()
    }

    /// Reset to zero
    pub fn reset(&self) {
        *self.count.lock() = 0;
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Efficient string interning
#[derive(Debug)]
pub struct StringInterner {
    strings: Mutex<AHashMap<u64, Arc<str>>>,
}

impl StringInterner {
    /// Create a new interner
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: Mutex::new(AHashMap::new()),
        }
    }

    /// Intern a string
    pub fn intern(&self, s: &str) -> Arc<str> {
        let hash = ahash::AHasher::default();
        let mut hasher = hash;
        std::hash::Hasher::write(&mut hasher, s.as_bytes());
        let key = std::hash::Hasher::finish(&hasher);

        let mut strings = self.strings.lock();
        if let Some(interned) = strings.get(&key) {
            Arc::clone(interned)
        } else {
            let interned: Arc<str> = Arc::from(s);
            strings.insert(key, Arc::clone(&interned));
            interned
        }
    }

    /// Get number of interned strings
    #[must_use]
    pub fn len(&self) -> usize {
        self.strings.lock().len()
    }

    /// Check if interner is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strings.lock().is_empty()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a thread-safe LRU cache
#[derive(Debug)]
pub struct LruCache<K, V> {
    map: Mutex<AHashMap<K, V>>,
    order: Mutex<Vec<K>>,
    capacity: usize,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    /// Create a new LRU cache
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            map: Mutex::new(AHashMap::new()),
            order: Mutex::new(Vec::new()),
            capacity,
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let map = self.map.lock();
        let mut order = self.order.lock();

        if let Some(value) = map.get(key) {
            // Move to front
            if let Some(pos) = order.iter().position(|k| k == key) {
                order.remove(pos);
                order.push(key.clone());
            }
            Some(value.clone())
        } else {
            None
        }
    }

    /// Insert a value into the cache
    pub fn insert(&self, key: K, value: V) {
        let mut map = self.map.lock();
        let mut order = self.order.lock();

        if map.contains_key(&key) {
            // Update existing
            if let Some(pos) = order.iter().position(|k| k == &key) {
                order.remove(pos);
            }
        } else if map.len() >= self.capacity {
            // Remove oldest
            if let Some(oldest) = order.first().cloned() {
                map.remove(&oldest);
                order.remove(0);
            }
        }

        map.insert(key.clone(), value);
        order.push(key);
    }

    /// Get cache size
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.lock().len()
    }

    /// Check if cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.lock().is_empty()
    }
}
