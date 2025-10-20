/// Time utilities and thread-safe data structures.
///
/// This module provides date/time operations and concurrent data structures
/// for efficient multi-threaded programming.
///
/// # Examples
///
/// Time operations:
/// ```rust
/// use trash_utilities::common::utils::*;
///
/// // Current time
/// let now = current_utc_time();
/// let formatted = format_datetime(&now);
/// println!("Current time: {}", formatted);
///
/// // Parse time
/// let parsed = parse_datetime("2023-01-01T12:00:00Z").unwrap();
/// let date = parse_date("2023-01-01").unwrap();
/// ```
///
/// Thread-safe data structures:
/// ```rust
/// use trash_utilities::common::utils::*;
///
/// // Atomic counter
/// let counter = AtomicCounter::new();
/// assert_eq!(counter.increment(), 1);
/// assert_eq!(counter.get(), 1);
/// counter.reset();
/// assert_eq!(counter.get(), 0);
///
/// // String interning
/// let interner = StringInterner::new();
/// let s1 = interner.intern("hello");
/// let s2 = interner.intern("hello");
/// assert_eq!(s1.as_ptr(), s2.as_ptr()); // Same memory
/// assert_eq!(interner.len(), 1);
///
/// // LRU cache
/// let cache = LruCache::new(3);
/// cache.insert("key1", "value1");
/// cache.insert("key2", "value2");
/// assert_eq!(cache.get(&"key1"), Some("value1"));
/// assert_eq!(cache.len(), 2);
/// ```
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
    dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()
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
///
/// A counter that can be safely shared across threads using `Arc` and `Mutex`.
/// Provides atomic increment operations and thread-safe access to the current value.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::utils::AtomicCounter;
/// use std::sync::Arc;
/// use std::thread;
///
/// let counter = Arc::new(AtomicCounter::new());
///
/// let handles: Vec<_> = (0..10).map(|_| {
///     let counter = Arc::clone(&counter);
///     thread::spawn(move || {
///         counter.increment();
///     })
/// }).collect();
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
///
/// assert_eq!(counter.get(), 10);
/// ```
#[derive(Debug, Clone)]
pub struct AtomicCounter {
    count: Arc<Mutex<u64>>,
}

impl AtomicCounter {
    /// Create a new counter
    ///
    /// Initializes the counter to zero.
    ///
    /// # Returns
    ///
    /// A new `AtomicCounter` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::AtomicCounter;
    ///
    /// let counter = AtomicCounter::new();
    /// assert_eq!(counter.get(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    /// Increment and return the new value
    ///
    /// Atomically increments the counter by 1 and returns the new value.
    ///
    /// # Returns
    ///
    /// The new counter value after incrementing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::AtomicCounter;
    ///
    /// let counter = AtomicCounter::new();
    /// assert_eq!(counter.increment(), 1);
    /// assert_eq!(counter.increment(), 2);
    /// ```
    #[must_use]
    pub fn increment(&self) -> u64 {
        let mut count = self.count.lock();
        *count += 1;
        *count
    }

    /// Get current value
    ///
    /// Returns the current value of the counter without modifying it.
    ///
    /// # Returns
    ///
    /// The current counter value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::AtomicCounter;
    ///
    /// let counter = AtomicCounter::new();
    /// counter.increment();
    /// counter.increment();
    /// assert_eq!(counter.get(), 2);
    /// ```
    #[must_use]
    pub fn get(&self) -> u64 {
        *self.count.lock()
    }

    /// Reset to zero
    ///
    /// Sets the counter value back to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::AtomicCounter;
    ///
    /// let counter = AtomicCounter::new();
    /// counter.increment();
    /// assert_eq!(counter.get(), 1);
    /// counter.reset();
    /// assert_eq!(counter.get(), 0);
    /// ```
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
///
/// A string interner that stores unique strings and returns reference-counted copies.
/// This reduces memory usage when the same strings are used repeatedly, as identical
/// strings share the same memory location.
///
/// Uses `AHash` for fast hashing and `Arc<str>` for efficient reference counting.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::utils::StringInterner;
/// use std::sync::Arc;
///
/// let interner = StringInterner::new();
///
/// // First interning creates new storage
/// let s1: Arc<str> = interner.intern("hello");
/// assert_eq!(interner.len(), 1);
///
/// // Second interning of same string returns same reference
/// let s2: Arc<str> = interner.intern("hello");
/// assert_eq!(s1.as_ptr(), s2.as_ptr()); // Same memory location
/// assert_eq!(interner.len(), 1); // Still only one unique string
///
/// // Different string creates new storage
/// let s3 = interner.intern("world");
/// assert_ne!(s1.as_ptr(), s3.as_ptr());
/// assert_eq!(interner.len(), 2);
/// ```
#[derive(Debug)]
pub struct StringInterner {
    strings: Mutex<AHashMap<u64, Arc<str>>>,
}

impl StringInterner {
    /// Create a new interner
    ///
    /// Initializes an empty string interner.
    ///
    /// # Returns
    ///
    /// A new `StringInterner` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::StringInterner;
    ///
    /// let interner = StringInterner::new();
    /// assert!(interner.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: Mutex::new(AHashMap::new()),
        }
    }

    /// Intern a string
    ///
    /// Stores the string if it hasn't been seen before, or returns a reference
    /// to the existing interned copy if it has.
    ///
    /// # Parameters
    ///
    /// * `s` - The string to intern.
    ///
    /// # Returns
    ///
    /// An `Arc<str>` pointing to the interned string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::StringInterner;
    ///
    /// let interner = StringInterner::new();
    /// let s1 = interner.intern("test");
    /// let s2 = interner.intern("test");
    /// assert_eq!(s1, s2);
    /// // s1 and s2 point to the same memory location
    /// ```
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
    ///
    /// Returns the count of unique strings that have been interned.
    ///
    /// # Returns
    ///
    /// The number of unique interned strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::StringInterner;
    ///
    /// let interner = StringInterner::new();
    /// interner.intern("hello");
    /// interner.intern("world");
    /// interner.intern("hello"); // Duplicate
    /// assert_eq!(interner.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.strings.lock().len()
    }

    /// Check if interner is empty
    ///
    /// Returns true if no strings have been interned yet.
    ///
    /// # Returns
    ///
    /// `true` if no strings are interned, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::StringInterner;
    ///
    /// let interner = StringInterner::new();
    /// assert!(interner.is_empty());
    /// interner.intern("test");
    /// assert!(!interner.is_empty());
    /// ```
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
///
/// A least-recently-used (LRU) cache with a fixed capacity that automatically
/// evicts the oldest entries when the capacity is exceeded. Thread-safe using
/// `Mutex` for concurrent access.
///
/// Uses `AHash` for fast key lookups and maintains access order for LRU eviction.
///
/// # Type Parameters
///
/// * `K` - The key type (must implement `Hash`, `Eq`, and `Clone`).
/// * `V` - The value type.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::common::utils::LruCache;
///
/// let cache = LruCache::new(3);
///
/// // Insert some items
/// cache.insert("key1", "value1");
/// cache.insert("key2", "value2");
/// cache.insert("key3", "value3");
/// assert_eq!(cache.len(), 3);
///
/// // Access an item to make it most recent
/// assert_eq!(cache.get(&"key1"), Some("value1"));
///
/// // Insert another item, should evict key2 (least recently used)
/// cache.insert("key4", "value4");
/// assert_eq!(cache.len(), 3);
/// assert_eq!(cache.get(&"key2"), None); // key2 was evicted
/// assert_eq!(cache.get(&"key1"), Some("value1")); // key1 still there
/// assert_eq!(cache.get(&"key3"), Some("value3"));
/// assert_eq!(cache.get(&"key4"), Some("value4"));
/// ```
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
    ///
    /// # Parameters
    ///
    /// * `capacity` - The maximum number of items the cache can hold.
    ///
    /// # Returns
    ///
    /// A new `LruCache` with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::LruCache;
    ///
    /// let cache: LruCache<String, i32> = LruCache::new(100);
    /// assert!(cache.is_empty());
    /// ```
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            map: Mutex::new(AHashMap::new()),
            order: Mutex::new(Vec::new()),
            capacity,
        }
    }

    /// Get a value from the cache
    ///
    /// Retrieves a value from the cache and marks it as most recently used.
    /// If the key exists, it gets moved to the front of the LRU order.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to look up.
    ///
    /// # Returns
    ///
    /// `Some(value)` if the key exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::LruCache;
    ///
    /// let cache = LruCache::new(2);
    /// cache.insert("key1", "value1");
    ///
    /// assert_eq!(cache.get(&"key1"), Some("value1"));
    /// assert_eq!(cache.get(&"nonexistent"), None);
    /// ```
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
    ///
    /// Inserts a key-value pair into the cache. If the key already exists,
    /// its value is updated and it becomes the most recently used.
    /// If the cache is at capacity, the least recently used item is evicted.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to insert.
    /// * `value` - The value to associate with the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::LruCache;
    ///
    /// let cache = LruCache::new(2);
    ///
    /// cache.insert("key1", "value1");
    /// cache.insert("key2", "value2");
    /// assert_eq!(cache.len(), 2);
    ///
    /// // Inserting when at capacity evicts oldest
    /// cache.insert("key3", "value3");
    /// assert_eq!(cache.len(), 2);
    /// assert_eq!(cache.get(&"key1"), None); // key1 was evicted
    /// ```
    pub fn insert(&self, key: K, value: V) {
        let mut map = self.map.lock();
        let mut order = self.order.lock();

        let was_present = map.contains_key(&key);
        if was_present {
            // Update existing - move to front
            if let Some(pos) = order.iter().position(|k| k == &key) {
                order.remove(pos);
            }
        }

        map.insert(key.clone(), value);
        order.push(key);

        // Evict if over capacity and this was a new insertion
        if !was_present
            && map.len() > self.capacity
            && let Some(oldest) = order.first().cloned()
        {
            map.remove(&oldest);
            order.remove(0);
        }
    }

    /// Get cache size
    ///
    /// Returns the current number of items in the cache.
    ///
    /// # Returns
    ///
    /// The number of items currently stored in the cache.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::LruCache;
    ///
    /// let cache = LruCache::new(10);
    /// assert_eq!(cache.len(), 0);
    /// cache.insert("key", "value");
    /// assert_eq!(cache.len(), 1);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.lock().len()
    }

    /// Check if cache is empty
    ///
    /// Returns true if the cache contains no items.
    ///
    /// # Returns
    ///
    /// `true` if the cache is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::common::utils::LruCache;
    ///
    /// let cache = LruCache::new(10);
    /// assert!(cache.is_empty());
    /// cache.insert("key", "value");
    /// assert!(!cache.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.lock().is_empty()
    }
}
