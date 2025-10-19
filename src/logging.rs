/// Comprehensive logging utilities with structured logging, async processing, and performance monitoring.
///
/// This module provides advanced logging capabilities using the tracing ecosystem,
/// including structured logging, async log processing, performance spans, and
/// thread-safe buffering. It integrates with serde for log serialization and
/// provides utilities for monitoring application performance.
/// Initialize tracing with comprehensive configuration options.
// Standard library imports
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

// External crate imports
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json;
use smol;
use tracing::{Level, debug, error, info, instrument, span, warn};
use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

///
/// This function sets up tracing with JSON formatting, environment filtering,
/// and optional async log processing.
///
/// # Parameters
/// - `enable_json`: Whether to use JSON formatting for logs.
/// - `async_buffer_size`: Size of async log buffer (0 disables async processing).
/// - `max_level`: Maximum log level to enable.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::init_advanced_tracing;
///
/// // Initialize with JSON logging and async processing
/// init_advanced_tracing(true, 1000, Some(tracing::Level::DEBUG));
/// ```
pub fn init_advanced_tracing(
    enable_json: bool,
    async_buffer_size: usize,
    max_level: Option<tracing::Level>,
) {
    let filter = if let Some(level) = max_level {
        LevelFilter::from_level(level)
    } else {
        LevelFilter::INFO
    };

    let registry = tracing_subscriber::registry().with(filter);

    if async_buffer_size > 0 {
        // Set up async logging
        let (tx, rx) = smol::channel::bounded(async_buffer_size);

        // Start async log processor
        smol::spawn(async move {
            let rx = rx;
            while let Ok(log_entry) = rx.recv().await {
                // Process log entry asynchronously
                println!("{log_entry}");
            }
        })
        .detach();

        let async_layer = fmt::layer().with_writer(move || AsyncLogWriter { tx: tx.clone() });

        if enable_json {
            registry.with(async_layer.json()).init();
        } else {
            registry.with(async_layer).init();
        }
    } else if enable_json {
        registry.with(fmt::layer().json()).init();
    } else {
        registry.with(fmt::layer()).init();
    }
}

/// Async log writer for buffered logging.
struct AsyncLogWriter {
    tx: smol::channel::Sender<String>,
}

impl std::io::Write for AsyncLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            let tx = self.tx.clone();
            let s = s.to_string();
            smol::spawn(async move {
                let _ = tx.send(s).await;
            })
            .detach();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Initialize tracing with default settings
pub fn init_tracing() {
    init_advanced_tracing(false, 0, None);
}

/// Structured log entry for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: serde_json::Value,
}

/// Thread-safe log buffer for collecting and processing logs.
#[derive(Debug)]
pub struct LogBuffer {
    buffer: Mutex<VecDeque<LogEntry>>,
    max_size: usize,
}

impl LogBuffer {
    /// Create a new log buffer with maximum size.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: Mutex::new(VecDeque::with_capacity(max_size)),
            max_size,
        }
    }

    /// Add a log entry to the buffer.
    pub fn push(&self, entry: LogEntry) {
        let mut buffer = self.buffer.lock();
        if buffer.len() >= self.max_size {
            buffer.pop_front();
        }
        buffer.push_back(entry);
    }

    /// Get all log entries and clear the buffer.
    pub fn drain(&self) -> Vec<LogEntry> {
        let mut buffer = self.buffer.lock();
        buffer.drain(..).collect()
    }

    /// Get the current number of entries.
    pub fn len(&self) -> usize {
        self.buffer.lock().len()
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().is_empty()
    }
}

/// Global log buffer instance.
static LOG_BUFFER: std::sync::LazyLock<Arc<LogBuffer>> =
    std::sync::LazyLock::new(|| Arc::new(LogBuffer::new(1000)));

/// Log with structured data and buffering.
///
/// This function creates a structured log entry and adds it to the global buffer
/// in addition to emitting it through tracing.
///
/// # Parameters
/// - `level`: The log level.
/// - `message`: The log message.
/// - `data`: Additional structured data.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::log_structured_buffered;
/// use serde_json::json;
///
/// log_structured_buffered("info", "User login", json!({
///     "user_id": 123,
///     "ip": "192.168.1.1"
/// }));
/// ```
pub fn log_structured_buffered(level: &str, message: &str, data: serde_json::Value) {
    let entry = LogEntry {
        timestamp: Utc::now(),
        level: level.to_string(),
        message: message.to_string(),
        module: std::module_path!().to_string().into(),
        file: std::file!().to_string().into(),
        line: std::line!().into(),
        fields: data,
    };

    // Add to buffer
    LOG_BUFFER.push(entry.clone());

    // Emit through tracing
    match level {
        "info" => info!("{}: {}", message, entry.fields),
        "warn" => warn!("{}: {}", message, entry.fields),
        "error" => error!("{}: {}", message, entry.fields),
        "debug" => debug!("{}: {}", message, entry.fields),
        _ => info!("{}: {}", message, entry.fields),
    }
}

/// Get buffered log entries and clear the buffer.
pub fn drain_log_buffer() -> Vec<LogEntry> {
    LOG_BUFFER.drain()
}

/// Performance monitoring span.
///
/// This function creates a tracing span for timing operations.
/// Use with the `instrument` attribute for automatic span creation.
///
/// # Parameters
/// - `name`: The span name.
/// - `f`: The function to execute within the span.
///
/// # Returns
/// The result of the function.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::timed_span;
///
/// let result = timed_span("database_query", || {
///     // Some operation
///     42
/// });
/// ```
#[instrument(skip(f))]
pub fn timed_span<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = span!(Level::INFO, "timed_operation", name);
    let _enter = span.enter();
    f()
}

/// Log performance metrics.
///
/// This function logs timing and performance information.
///
/// # Parameters
/// - `operation`: The operation name.
/// - `duration`: How long the operation took.
/// - `metadata`: Additional performance metadata.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::log_performance;
/// use std::time::Duration;
/// use serde_json::json;
///
/// log_performance("file_read", Duration::from_millis(150), json!({
///     "file_size": 1024,
///     "throughput": "6.8 MB/s"
/// }));
/// ```
pub fn log_performance(operation: &str, duration: Duration, metadata: serde_json::Value) {
    info!(
        operation = operation,
        duration_ms = duration.as_millis(),
        metadata = %metadata,
        "Performance metric"
    );
}

/// Create a performance timer.
///
/// This function returns a timer that logs when dropped.
///
/// # Parameters
/// - `operation`: The operation name.
///
/// # Returns
/// A timer that logs on drop.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::performance_timer;
///
/// let _timer = performance_timer("complex_calculation");
/// // ... do work ...
/// // Timer logs automatically when dropped
/// ```
#[must_use]
pub fn performance_timer(operation: &str) -> PerformanceTimer {
    PerformanceTimer {
        operation: operation.to_string(),
        start: Instant::now(),
    }
}

/// Performance timer that logs on drop.
pub struct PerformanceTimer {
    operation: String,
    start: Instant,
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        log_performance(&self.operation, duration, serde_json::Value::Null);
    }
}

/// Log an info message
pub fn log_info(message: &str) {
    info!("{}", message);
}

/// Log a warning message
pub fn log_warn(message: &str) {
    warn!("{}", message);
}

/// Log an error message
pub fn log_error(message: &str) {
    error!("{}", message);
}

/// Log a debug message
pub fn log_debug(message: &str) {
    debug!("{}", message);
}

/// Log with structured data (legacy function for compatibility)
pub fn log_structured(level: &str, message: &str, data: serde_json::Value) {
    log_structured_buffered(level, message, data);
}

/// Serialize log entries to JSON.
///
/// This function serializes a vector of log entries to JSON format.
///
/// # Parameters
/// - `entries`: The log entries to serialize.
///
/// # Returns
/// JSON string representation of the log entries.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::{drain_log_buffer, serialize_logs};
///
/// let entries = drain_log_buffer();
/// let json = serialize_logs(&entries).unwrap();
/// println!("Logs: {}", json);
/// ```
pub fn serialize_logs(entries: &[LogEntry]) -> Result<String, serde_json::Error> {
    serde_json::to_string(entries)
}

/// Deserialize log entries from JSON.
///
/// This function deserializes log entries from JSON format.
///
/// # Parameters
/// - `json`: The JSON string containing log entries.
///
/// # Returns
/// Vector of deserialized log entries.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::deserialize_logs;
///
/// let json = r#"[{"timestamp":"2025-10-19T...","level":"info","message":"test"}]"#;
/// let entries = deserialize_logs(json).unwrap();
/// println!("Loaded {} log entries", entries.len());
/// ```
pub fn deserialize_logs(json: &str) -> Result<Vec<LogEntry>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Filter log entries by level.
///
/// This function filters a vector of log entries by log level.
///
/// # Parameters
/// - `entries`: The log entries to filter.
/// - `min_level`: The minimum log level to include.
///
/// # Returns
/// Filtered vector of log entries.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::{drain_log_buffer, filter_logs_by_level};
///
/// let entries = drain_log_buffer();
/// let errors_only = filter_logs_by_level(&entries, "error");
/// println!("Found {} errors", errors_only.len());
/// ```
#[must_use]
pub fn filter_logs_by_level(entries: &[LogEntry], min_level: &str) -> Vec<LogEntry> {
    let level_priority = |level: &str| match level {
        "error" => 4,
        "warn" => 3,
        "info" => 2,
        "debug" => 1,
        _ => 0,
    };

    let min_priority = level_priority(min_level);
    entries
        .iter()
        .filter(|entry| level_priority(&entry.level) >= min_priority)
        .cloned()
        .collect()
}

/// Get log statistics.
///
/// This function analyzes log entries and returns statistics.
///
/// # Parameters
/// - `entries`: The log entries to analyze.
///
/// # Returns
/// Statistics about the log entries.
///
/// # Examples
/// ```rust
/// use trash_analyzer::base::logging::{drain_log_buffer, get_log_stats};
///
/// let entries = drain_log_buffer();
/// let stats = get_log_stats(&entries);
/// println!("Log stats: {:?}", stats);
/// ```
#[must_use]
pub fn get_log_stats(entries: &[LogEntry]) -> LogStats {
    let mut stats = LogStats::default();
    for entry in entries {
        stats.total_entries += 1;
        match entry.level.as_str() {
            "error" => stats.error_count += 1,
            "warn" => stats.warn_count += 1,
            "info" => stats.info_count += 1,
            "debug" => stats.debug_count += 1,
            _ => stats.other_count += 1,
        }
    }
    stats
}

/// Log statistics structure.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_entries: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub debug_count: usize,
    pub other_count: usize,
}

/// Async log processor for background log handling.
///
/// This function sets up an async log processing pipeline.
///
/// # Parameters
/// - `buffer_size`: Size of the processing buffer.
/// - `processor`: Function to process each log entry.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::base::logging::{LogEntry, setup_async_log_processor};
///
/// setup_async_log_processor(100, |entry: LogEntry| async move {
///     // Send to external logging service
///     println!("Processing log: {}", entry.message);
/// });
/// ```
pub fn setup_async_log_processor<F, Fut>(buffer_size: usize, processor: F)
where
    F: Fn(LogEntry) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (_tx, rx) = smol::channel::bounded(buffer_size);

    // Store sender globally for log_structured_buffered to use
    // This is a simplified version - in practice you'd want better global state management

    // Start processor
    smol::spawn(async move {
        let rx = rx;
        while let Ok(entry) = rx.recv().await {
            processor(entry).await;
        }
    })
    .detach();
}
