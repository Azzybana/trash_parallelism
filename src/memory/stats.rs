/// Statistics and monitoring for memory operations.
///
/// This module provides comprehensive memory statistics tracking,
/// profiling capabilities, event logging, and snapshot functionality
/// for debugging and performance analysis.
// Standard library imports
use std::{hash::Hasher, time::Instant};

// External crate imports
use ahash::{AHashMap, AHasher};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

// Local imports
use super::MemoryManager;

/// Memory allocation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    pub allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub total_allocated_bytes: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub fragmentation_ratio: f64,
    pub heap_size: usize,
}

/// Allocation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStats {
    pub total_size: usize,
    pub count: usize,
    pub avg_size: usize,
    pub rate: f64, // allocations per second
}

/// Memory profiler for tracking allocations
#[derive(Debug)]
pub struct MemoryProfiler {
    allocations: parking_lot::Mutex<AHashMap<String, Vec<(usize, Instant)>>>,
    active: std::sync::atomic::AtomicBool,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    #[must_use]
    pub fn new() -> Self {
        Self {
            allocations: parking_lot::Mutex::new(AHashMap::new()),
            active: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Start profiling
    pub fn start(&self) {
        self.active
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Stop profiling
    pub fn stop(&self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record an allocation
    pub fn record_allocation(&self, tag: &str, size: usize) {
        if !self.active.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let mut allocations = self.allocations.lock();
        allocations
            .entry(tag.to_string())
            .or_default()
            .push((size, Instant::now()));
    }

    /// Get profiling report
    #[must_use]
    pub fn report(&self) -> AHashMap<String, AllocationStats> {
        let allocations = self.allocations.lock();
        let mut report = AHashMap::new();

        for (tag, allocs) in allocations.iter() {
            let total_size: usize = allocs.iter().map(|(size, _)| size).sum();
            let count = allocs.len();
            let avg_size = if count > 0 { total_size / count } else { 0 };

            // Calculate allocation rate (allocations per second)
            let now = Instant::now();
            let time_span = allocs
                .iter()
                .map(|(_, time)| now.duration_since(*time))
                .max();
            // Using a built-in function to handle sanity checking.
            #[allow(clippy::cast_precision_loss)]
            let rate = if let Some(span) = time_span {
                if span.as_secs_f64() > 0.0 {
                    count as f64 / span.as_secs_f64()
                } else {
                    0.0
                }
            } else {
                0.0
            };

            report.insert(
                tag.clone(),
                AllocationStats {
                    total_size,
                    count,
                    avg_size,
                    rate,
                },
            );
        }

        report
    }

    /// Clear profiling data
    pub fn clear(&self) {
        self.allocations.lock().clear();
    }

    /// Check if profiling is active
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory event logger with timestamps
#[derive(Debug)]
pub struct MemoryEventLogger {
    events: Mutex<Vec<MemoryEvent>>,
    max_events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: MemoryEventType,
    pub size: usize,
    pub pool_name: Option<String>,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEventType {
    Allocation,
    Deallocation,
    PoolCreated,
    PoolDestroyed,
    Compression,
    Decompression,
    Encryption,
    Decryption,
}

impl MemoryEventLogger {
    /// Create a new event logger
    #[must_use]
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            max_events,
        }
    }

    /// Log a memory event
    pub fn log_event(
        &self,
        event_type: MemoryEventType,
        size: usize,
        pool_name: Option<&str>,
        details: &str,
    ) {
        let event = MemoryEvent {
            timestamp: Utc::now(),
            event_type,
            size,
            pool_name: pool_name.map(std::string::ToString::to_string),
            details: details.to_string(),
        };

        let mut events = self.events.lock();
        events.push(event);

        // Maintain max size
        if events.len() > self.max_events {
            events.remove(0);
        }
    }

    /// Get recent events
    #[must_use]
    pub fn recent_events(&self, count: usize) -> Vec<MemoryEvent> {
        let events = self.events.lock();
        let start = if events.len() > count {
            events.len() - count
        } else {
            0
        };
        events[start..].to_vec()
    }

    /// Export events as JSON
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let events = self.events.lock();
        serde_json::to_string(&*events)
    }

    /// Clear all events
    pub fn clear(&self) {
        self.events.lock().clear();
    }

    /// Get number of events
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.lock().len()
    }

    /// Check if logger is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.lock().is_empty()
    }
}

impl Default for MemoryEventLogger {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Memory snapshot for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp: DateTime<Utc>,
    pub stats: MemoryStats,
    pub pools: AHashMap<String, MemoryStats>,
    pub checksum: u64,
}

impl MemorySnapshot {
    /// Create a new memory snapshot
    #[must_use]
    pub fn new(manager: &MemoryManager) -> Self {
        let stats = manager.global_stats();
        let pools = manager
            .list_pools()
            .into_iter()
            .filter_map(|name| {
                manager
                    .get_pool(&name)
                    .map(|pool| (name, (*pool.stats()).clone()))
            })
            .collect();

        let checksum_data = format!("{:?}{:?}{:?}", stats, pools, Utc::now());
        let mut hasher = AHasher::default();
        std::hash::Hasher::write(&mut hasher, checksum_data.as_bytes());
        let checksum = hasher.finish();

        Self {
            timestamp: Utc::now(),
            stats: (*stats).clone(),
            pools,
            checksum,
        }
    }

    /// Export snapshot as base64-encoded JSON
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails.
    pub fn export_base64(&self) -> Result<String, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(STANDARD.encode(json.as_bytes()))
    }

    /// Import snapshot from base64-encoded JSON
    ///
    /// # Errors
    ///
    /// Returns an error if base64 decoding or JSON parsing fails.
    pub fn import_base64(data: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = STANDARD.decode(data)?;
        let json_str = String::from_utf8(json)?;
        let snapshot: Self = serde_json::from_str(&json_str)?;
        Ok(snapshot)
    }

    /// Verify snapshot integrity
    #[must_use]
    pub fn verify(&self) -> bool {
        let checksum_data = format!("{:?}{:?}{:?}", self.stats, self.pools, self.timestamp);
        let mut hasher = AHasher::default();
        std::hash::Hasher::write(&mut hasher, checksum_data.as_bytes());
        let computed = hasher.finish();
        computed == self.checksum
    }

    /// Get snapshot timestamp
    #[must_use]
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get memory statistics
    #[must_use]
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Get pool statistics
    #[must_use]
    pub fn pools(&self) -> &AHashMap<String, MemoryStats> {
        &self.pools
    }
}
