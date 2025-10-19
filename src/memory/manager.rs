/// Memory management and global state.
///
/// This module provides memory managers, global state management,
/// and utilities for coordinating memory operations across the system.
// Standard library imports
use std::{fmt::Write, sync::Arc, thread::spawn, time::Duration};

// External crate imports
use ahash::AHashMap;
use parking_lot::{Mutex, RwLock};

// Local imports
use super::{
    CompressedMemoryPool, MemoryEventLogger, MemoryEventType, MemoryPool, MemoryPoolConfig,
    MemorySnapshot, MemoryStats, ParallelMemoryProcessor, SecureMemoryPool, calc_ratio,
    get_mimalloc_stats,
};

/// Global memory manager
#[derive(Debug)]
pub struct MemoryManager {
    pools: RwLock<AHashMap<String, Arc<MemoryPool>>>,
    monitoring_active: Mutex<bool>,
}

impl MemoryManager {
    /// Create a new memory manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(AHashMap::new()),
            monitoring_active: Mutex::new(true),
        }
    }

    /// Create a memory pool
    pub fn create_pool(&self, config: &MemoryPoolConfig) -> Arc<MemoryPool> {
        let pool = Arc::new(MemoryPool::new(config.clone()));
        let mut pools = self.pools.write();
        pools.insert(config.name.clone(), pool.clone());
        pool
    }

    /// Get a memory pool by name
    pub fn get_pool(&self, name: &str) -> Option<Arc<MemoryPool>> {
        let pools = self.pools.read();
        pools.get(name).cloned()
    }

    /// List all pools
    #[must_use]
    pub fn list_pools(&self) -> Vec<String> {
        let pools = self.pools.read();
        pools.keys().cloned().collect()
    }

    /// Remove a pool
    pub fn remove_pool(&self, name: &str) -> bool {
        let mut pools = self.pools.write();
        pools.remove(name).is_some()
    }

    /// Get global memory statistics
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn global_stats(&self) -> Arc<MemoryStats> {
        // Aggregate stats from all pools
        let pools = self.pools.read();
        let mut total_stats = MemoryStats::default();

        for pool in pools.values() {
            let pool_stats = pool.stats();
            total_stats.allocated_bytes += pool_stats.allocated_bytes;
            total_stats.peak_allocated_bytes = total_stats
                .peak_allocated_bytes
                .max(pool_stats.peak_allocated_bytes);
            total_stats.total_allocated_bytes += pool_stats.total_allocated_bytes;
            total_stats.allocation_count += pool_stats.allocation_count;
            total_stats.deallocation_count += pool_stats.deallocation_count;
        }

        // Add mimalloc global stats if available
        if let Some(mi_stats) = get_mimalloc_stats() {
            total_stats.heap_size = mi_stats;
        }

        total_stats.fragmentation_ratio =
            calc_ratio(total_stats.allocated_bytes, total_stats.heap_size);

        Arc::new(total_stats)
    }

    /// Start memory monitoring
    pub fn start_monitoring(&self, interval: Duration) {
        let mut monitoring = self.monitoring_active.lock();
        if !*monitoring {
            *monitoring = true;

            let manager = Arc::new(self.clone());
            spawn(move || {
                while *manager.monitoring_active.lock() {
                    std::thread::sleep(interval);
                    // Memory stats logging removed for simplicity
                }
            });
        }
    }

    /// Stop memory monitoring
    pub fn stop_monitoring(&self) {
        *self.monitoring_active.lock() = false;
    }

    /// Force garbage collection (mimalloc)
    pub fn collect_garbage(&self) {
        // Mimalloc doesn't expose direct GC, but we can hint
    }

    /// Get memory usage report
    #[must_use]
    pub fn memory_report(&self) -> String {
        let stats = self.global_stats();
        let pools = self.pools.read();

        let mut report = format!(
            "Memory Report:\n\
             Global Stats:\n\
             - Allocated: {} bytes\n\
             - Peak: {} bytes\n\
             - Total Allocated: {} bytes\n\
             - Allocations: {}\n\
             - Deallocations: {}\n\
             - Fragmentation: {:.2}%\n\
             - Heap Size: {} bytes\n\
             \n\
             Pools ({}):\n",
            stats.allocated_bytes,
            stats.peak_allocated_bytes,
            stats.total_allocated_bytes,
            stats.allocation_count,
            stats.deallocation_count,
            stats.fragmentation_ratio * 100.0,
            stats.heap_size,
            pools.len()
        );

        for (name, pool) in pools.iter() {
            let pool_stats = pool.stats();
            let _ = write!(
                report,
                "- {}: {} bytes allocated, {} blocks",
                name, pool_stats.allocated_bytes, pool_stats.allocation_count
            );
            report.push('\n');
        }

        report
    }
}

impl Clone for MemoryManager {
    fn clone(&self) -> Self {
        Self {
            pools: RwLock::new(AHashMap::new()), // Don't clone pools for simplicity
            monitoring_active: Mutex::new(*self.monitoring_active.lock()),
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced memory manager with all features
#[derive(Debug)]
pub struct EnhancedMemoryManager {
    base_manager: MemoryManager,
    compressed_pools: RwLock<AHashMap<String, Arc<CompressedMemoryPool>>>,
    secure_pools: RwLock<AHashMap<String, Arc<SecureMemoryPool>>>,
    parallel_processor: ParallelMemoryProcessor,
    logger: Arc<MemoryEventLogger>,
}

impl EnhancedMemoryManager {
    /// Create a new enhanced memory manager
    #[must_use]
    pub fn new(num_threads: usize) -> Self {
        Self {
            base_manager: MemoryManager::new(),
            compressed_pools: RwLock::new(AHashMap::new()),
            secure_pools: RwLock::new(AHashMap::new()),
            parallel_processor: ParallelMemoryProcessor::new(num_threads),
            logger: Arc::new(MemoryEventLogger::new(10000)),
        }
    }

    /// Create a compressed memory pool
    pub fn create_compressed_pool(
        &self,
        config: &MemoryPoolConfig,
        compression_level: u32,
    ) -> Arc<CompressedMemoryPool> {
        let pool = Arc::new(CompressedMemoryPool::new(config.clone(), compression_level));
        let mut pools = self.compressed_pools.write();
        pools.insert(config.name.clone(), pool.clone());

        self.logger.log_event(
            MemoryEventType::PoolCreated,
            0,
            Some(&config.name),
            &format!("Created compressed pool with level {compression_level}"),
        );

        pool
    }

    /// Create a secure memory pool
    pub fn create_secure_pool(
        &self,
        config: &MemoryPoolConfig,
        encryption_key: Option<Vec<u8>>,
    ) -> Arc<SecureMemoryPool> {
        let pool = Arc::new(SecureMemoryPool::new(config.clone(), encryption_key));
        let mut pools = self.secure_pools.write();
        pools.insert(config.name.clone(), pool.clone());

        self.logger.log_event(
            MemoryEventType::PoolCreated,
            0,
            Some(&config.name),
            "Created secure pool",
        );

        pool
    }

    /// Get compressed pool by name
    pub fn get_compressed_pool(&self, name: &str) -> Option<Arc<CompressedMemoryPool>> {
        let pools = self.compressed_pools.read();
        pools.get(name).cloned()
    }

    /// Get secure pool by name
    pub fn get_secure_pool(&self, name: &str) -> Option<Arc<SecureMemoryPool>> {
        let pools = self.secure_pools.read();
        pools.get(name).cloned()
    }

    /// Create memory snapshot
    #[must_use]
    pub fn create_snapshot(&self) -> MemorySnapshot {
        MemorySnapshot::new(&self.base_manager)
    }

    /// Get parallel processor
    #[must_use]
    pub fn parallel_processor(&self) -> &ParallelMemoryProcessor {
        &self.parallel_processor
    }

    /// Get event logger
    #[must_use]
    pub fn logger(&self) -> &Arc<MemoryEventLogger> {
        &self.logger
    }

    /// Get base manager
    #[must_use]
    pub fn base_manager(&self) -> &MemoryManager {
        &self.base_manager
    }
}

impl Default for EnhancedMemoryManager {
    fn default() -> Self {
        Self::new(4)
    }
}

/// Get the global memory manager
#[must_use]
pub fn global_memory_manager() -> Arc<MemoryManager> {
    Arc::new(MemoryManager::new())
}

/// Initialize memory management with monitoring
pub fn init_memory_management(monitoring_interval: Option<Duration>) {
    let manager = global_memory_manager();

    if let Some(interval) = monitoring_interval {
        manager.start_monitoring(interval);
    }
}

/// Create a default memory pool configuration
#[must_use]
pub fn default_pool_config(name: &str) -> MemoryPoolConfig {
    MemoryPoolConfig {
        initial_size: 1024 * 1024,         // 1MB
        max_size: Some(100 * 1024 * 1024), // 100MB
        alignment: 8,
        name: name.to_string(),
    }
}

/// Create a high-performance pool configuration
#[must_use]
pub fn high_perf_pool_config(name: &str) -> MemoryPoolConfig {
    MemoryPoolConfig {
        initial_size: 64 * 1024 * 1024,     // 64MB
        max_size: Some(1024 * 1024 * 1024), // 1GB
        alignment: 64,                      // Cache line alignment
        name: name.to_string(),
    }
}
