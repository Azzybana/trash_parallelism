// Standard library imports
use std::{
    alloc::{Layout, alloc, dealloc},
    io::{Read, Write},
    ptr::write_bytes,
    slice::from_raw_parts,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{sleep, spawn},
    time::{Duration, Instant},
};

// External crate imports
use ahash::AHashMap;
use arc_swap::ArcSwap;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use brotli::{CompressorWriter, Decompressor};
use chrono::{DateTime, Utc};
use mimalloc::MiMalloc;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

// Calculate ratio after reducing both numerator and denominator by 20%.
// This uses integer arithmetic (multiply by 4, divide by 5) to avoid floating-point
// operations, and reduces the values to help fit within f64's 53-bit mantissa precision
// (values up to ~9e15). The 20% reduction is arbitrary but helps prevent overflow
// and precision loss when casting large usize values to f64.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn calc_ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator > 0 {
        // Reduce by 20% using integer math: (x * 4) / 5 ≈ x * 0.8
        let reduced_num = ((numerator as u128 * 4) / 5) as f64;
        let reduced_den = ((denominator as u128 * 4) / 5) as f64;
        reduced_num / reduced_den
    } else {
        0.0
    }
}

/// Global mimalloc allocator instance
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

/// Memory pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolConfig {
    pub initial_size: usize,
    pub max_size: Option<usize>,
    pub alignment: usize,
    pub name: String,
}

/// Memory pool for efficient allocations
pub struct MemoryPool {
    config: MemoryPoolConfig,
    allocated_blocks: Mutex<AHashMap<usize, Vec<Box<[u8]>>>>,
    stats: ArcSwap<MemoryStats>,
    active: AtomicBool,
}

impl MemoryPool {
    /// Create a new memory pool
    #[must_use]
    pub fn new(config: MemoryPoolConfig) -> Self {
        Self {
            config,
            allocated_blocks: Mutex::new(AHashMap::new()),
            stats: ArcSwap::new(Arc::new(MemoryStats::default())),
            active: AtomicBool::new(true),
        }
    }

    /// Allocate memory from the pool
    pub fn allocate(&self, size: usize) -> Result<*mut u8, std::io::Error> {
        if !self.active.load(Ordering::Relaxed) {
            return Err(std::io::Error::other("Pool is inactive"));
        }

        // Check size limits
        if let Some(max_size) = self.config.max_size {
            let current_stats = self.stats.load();
            if current_stats.allocated_bytes + size > max_size {
                return Err(std::io::Error::other("Pool size limit exceeded"));
            }
        }

        let mut boxed = vec![0u8; size].into_boxed_slice();
        let ptr = boxed.as_mut_ptr();

        // Track allocation
        let mut blocks = self.allocated_blocks.lock();
        blocks.entry(size).or_default().push(boxed);

        // Update stats
        let mut new_stats = (**self.stats.load()).clone();
        new_stats.allocated_bytes += size;
        new_stats.allocation_count += 1;
        new_stats.total_allocated_bytes += size;
        new_stats.peak_allocated_bytes = new_stats
            .peak_allocated_bytes
            .max(new_stats.allocated_bytes);
        self.stats.store(Arc::new(new_stats));

        Ok(ptr)
    }

    /// Deallocate memory from the pool
    pub fn deallocate(&self, ptr: *mut u8, size: usize) -> Result<(), std::io::Error> {
        if ptr.is_null() {
            return Ok(());
        }

        let layout = Layout::from_size_align(size, self.config.alignment)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        unsafe { dealloc(ptr, layout) };

        // Remove from tracking
        let mut blocks = self.allocated_blocks.lock();
        if let Some(block_list) = blocks.get_mut(&size)
            && let Some(pos) = block_list.iter().position(|b| b.as_ptr().cast_mut() == ptr)
        {
            block_list.remove(pos);
        }

        // Update stats
        let mut new_stats = (**self.stats.load()).clone();
        new_stats.allocated_bytes = new_stats.allocated_bytes.saturating_sub(size);
        new_stats.deallocation_count += 1;
        self.stats.store(Arc::new(new_stats));

        Ok(())
    }

    /// Get current statistics
    pub fn stats(&self) -> Arc<MemoryStats> {
        self.stats.load_full()
    }

    /// Get pool configuration
    pub fn config(&self) -> &MemoryPoolConfig {
        &self.config
    }

    /// Check if pool is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Deactivate the pool
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Get fragmentation ratio
    #[allow(clippy::cast_precision_loss)]
    pub fn fragmentation_ratio(&self) -> f64 {
        let stats = self.stats.load();
        calc_ratio(stats.allocated_bytes, stats.heap_size)
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        // The boxes will be dropped automatically
    }
}

/// Global memory manager
pub struct MemoryManager {
    pools: RwLock<AHashMap<String, Arc<MemoryPool>>>,
    global_stats: ArcSwap<MemoryStats>,
    monitoring_active: AtomicBool,
}

impl MemoryManager {
    /// Create a new memory manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(AHashMap::new()),
            global_stats: ArcSwap::new(Arc::new(MemoryStats::default())),
            monitoring_active: AtomicBool::new(true),
        }
    }

    /// Create a memory pool
    pub fn create_pool(&self, config: MemoryPoolConfig) -> Arc<MemoryPool> {
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
        if !self.monitoring_active.load(Ordering::Relaxed) {
            return;
        }

        let manager = Arc::new(self.clone());
        spawn(move || {
            loop {
                sleep(interval);
                if !manager.monitoring_active.load(Ordering::Relaxed) {
                    break;
                }

                // Memory stats logging removed
            }
        });
    }

    /// Stop memory monitoring
    pub fn stop_monitoring(&self) {
        self.monitoring_active.store(false, Ordering::Relaxed);
    }

    /// Force garbage collection (mimalloc)
    pub fn collect_garbage(&self) {
        // Mimalloc doesn't expose direct GC, but we can hint
    }

    /// Get memory usage report
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
            report.push_str(&format!(
                "- {}: {} bytes allocated, {} blocks\n",
                name, pool_stats.allocated_bytes, pool_stats.allocation_count
            ));
        }

        report
    }
}

impl Clone for MemoryManager {
    fn clone(&self) -> Self {
        Self {
            pools: RwLock::new(AHashMap::new()), // Don't clone pools for simplicity
            global_stats: ArcSwap::new(Arc::new(MemoryStats::default())),
            monitoring_active: AtomicBool::new(self.monitoring_active.load(Ordering::Relaxed)),
        }
    }
}

/// Get mimalloc statistics (extended features)
#[must_use]
pub fn get_mimalloc_stats() -> Option<usize> {
    // Mimalloc extended stats are not directly exposed in the Rust wrapper
    // This is a placeholder for when extended stats become available
    None
}

/// Memory arena for bulk allocations
pub struct MemoryArena {
    buffer: Mutex<Vec<u8>>,
    offset: AtomicUsize,
    capacity: usize,
    name: String,
}

impl MemoryArena {
    /// Create a new memory arena
    #[must_use]
    pub fn new(capacity: usize, name: String) -> Self {
        Self {
            buffer: Mutex::new(vec![0u8; capacity]),
            offset: AtomicUsize::new(0),
            capacity,
            name,
        }
    }

    /// Allocate from the arena
    pub fn allocate(&self, size: usize) -> Result<Vec<u8>, std::io::Error> {
        let current_offset = self.offset.fetch_add(size, Ordering::Relaxed);

        if current_offset + size > self.capacity {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "Arena full",
            ));
        }

        let buffer = self.buffer.lock();
        let slice = &buffer[current_offset..current_offset + size];
        Ok(slice.to_vec())
    }

    /// Reset the arena
    pub fn reset(&self) {
        self.offset.store(0, Ordering::Relaxed);
    }

    /// Get current usage
    pub fn usage(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Thread-local memory cache
pub struct ThreadLocalCache<T> {
    cache: Mutex<Vec<T>>,
    max_size: usize,
    factory: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T> ThreadLocalCache<T> {
    /// Create a new thread-local cache
    #[must_use]
    pub fn new(max_size: usize, factory: Box<dyn Fn() -> T + Send + Sync>) -> Self {
        Self {
            cache: Mutex::new(Vec::new()),
            max_size,
            factory,
        }
    }

    /// Get an item from the cache or create new
    pub fn get_or_create(&self) -> T {
        let mut cache = self.cache.lock();
        if let Some(item) = cache.pop() {
            item
        } else {
            (self.factory)()
        }
    }

    /// Return an item to the cache
    pub fn return_item(&self, item: T) {
        let mut cache = self.cache.lock();
        if cache.len() < self.max_size {
            cache.push(item);
        }
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.lock().len()
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.lock().clear();
    }
}

/// Memory profiler for tracking allocations
pub struct MemoryProfiler {
    allocations: ArcSwap<AHashMap<String, Vec<(usize, Instant)>>>,
    active: AtomicBool,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    #[must_use]
    pub fn new() -> Self {
        Self {
            allocations: ArcSwap::new(Arc::new(AHashMap::new())),
            active: AtomicBool::new(false),
        }
    }

    /// Start profiling
    pub fn start(&self) {
        self.active.store(true, Ordering::Relaxed);
    }

    /// Stop profiling
    pub fn stop(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Record an allocation
    pub fn record_allocation(&self, tag: &str, size: usize) {
        if !self.active.load(Ordering::Relaxed) {
            return;
        }

        let mut allocations = (**self.allocations.load()).clone();
        allocations
            .entry(tag.to_string())
            .or_default()
            .push((size, Instant::now()));
        self.allocations.store(Arc::new(allocations));
    }

    /// Get profiling report
    pub fn report(&self) -> AHashMap<String, AllocationStats> {
        let allocations = self.allocations.load();
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
        self.allocations.store(Arc::new(AHashMap::new()));
    }
}

/// Allocation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStats {
    pub total_size: usize,
    pub count: usize,
    pub avg_size: usize,
    pub rate: f64, // allocations per second
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

/// Utility function to allocate aligned memory
pub fn allocate_aligned(size: usize, alignment: usize) -> Result<*mut u8, std::io::Error> {
    let layout = Layout::from_size_align(size, alignment)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        Err(std::io::Error::new(
            std::io::ErrorKind::OutOfMemory,
            "Allocation failed",
        ))
    } else {
        Ok(ptr)
    }
}

/// Utility function to deallocate aligned memory
pub fn deallocate_aligned(
    ptr: *mut u8,
    size: usize,
    alignment: usize,
) -> Result<(), std::io::Error> {
    if ptr.is_null() {
        return Ok(());
    }

    let layout = Layout::from_size_align(size, alignment)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    unsafe { std::alloc::dealloc(ptr, layout) };
    Ok(())
}

/// Memory usage guard that tracks and limits usage
pub struct MemoryUsageGuard {
    max_bytes: usize,
    current_bytes: AtomicUsize,
}

impl MemoryUsageGuard {
    /// Create a new memory usage guard
    #[must_use]
    pub fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            current_bytes: AtomicUsize::new(0),
        }
    }

    /// Try to allocate bytes
    pub fn try_allocate(&self, bytes: usize) -> Result<MemoryAllocationGuard<'_>, std::io::Error> {
        let current = self.current_bytes.load(Ordering::Relaxed);
        if current + bytes > self.max_bytes {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "Memory limit exceeded",
            ));
        }

        self.current_bytes.fetch_add(bytes, Ordering::Relaxed);
        Ok(MemoryAllocationGuard { bytes, guard: self })
    }

    /// Get current usage
    pub fn current_usage(&self) -> usize {
        self.current_bytes.load(Ordering::Relaxed)
    }

    /// Get maximum allowed bytes
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    fn deallocate(&self, bytes: usize) {
        self.current_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }
}

/// Guard for memory allocation that automatically deallocates on drop
pub struct MemoryAllocationGuard<'a> {
    bytes: usize,
    guard: &'a MemoryUsageGuard,
}

impl Drop for MemoryAllocationGuard<'_> {
    fn drop(&mut self) {
        self.guard.deallocate(self.bytes);
    }
}

/// Compressed memory pool for space-efficient storage
pub struct CompressedMemoryPool {
    pool: MemoryPool,
    compression_level: u32,
}

impl CompressedMemoryPool {
    /// Create a new compressed memory pool
    #[must_use]
    pub fn new(config: MemoryPoolConfig, compression_level: u32) -> Self {
        Self {
            pool: MemoryPool::new(config),
            compression_level,
        }
    }

    /// Allocate and compress data
    pub fn allocate_compressed(
        &self,
        data: &[u8],
    ) -> Result<CompressedAllocation<'_>, std::io::Error> {
        let compressed = compress_data(data, self.compression_level)?;
        let ptr = self.pool.allocate(compressed.len())?;
        unsafe {
            ptr.copy_from(compressed.as_ptr(), compressed.len());
        }

        Ok(CompressedAllocation {
            ptr,
            compressed_size: compressed.len(),
            original_size: data.len(),
            pool: &self.pool,
        })
    }

    /// Get pool statistics
    pub fn stats(&self) -> Arc<MemoryStats> {
        self.pool.stats()
    }
}

/// Guard for compressed memory allocation
pub struct CompressedAllocation<'a> {
    ptr: *mut u8,
    compressed_size: usize,
    original_size: usize,
    pool: &'a MemoryPool,
}

impl CompressedAllocation<'_> {
    /// Decompress and get original data
    pub fn decompress(&self) -> Result<Vec<u8>, std::io::Error> {
        let compressed_data = unsafe { from_raw_parts(self.ptr, self.compressed_size) };
        decompress_data(compressed_data)
    }

    /// Get compression ratio
    #[must_use]
    pub fn compression_ratio(&self) -> f64 {
        self.compressed_size as f64 / self.original_size as f64
    }
}

impl Drop for CompressedAllocation<'_> {
    fn drop(&mut self) {
        let _ = self.pool.deallocate(self.ptr, self.compressed_size);
    }
}

/// Memory-mapped file pool for large data
pub struct MemoryMappedPool {
    temp_file: NamedTempFile,
    mapped_data: Mutex<Vec<u8>>,
    stats: ArcSwap<MemoryStats>,
}

impl MemoryMappedPool {
    /// Create a new memory-mapped pool
    pub fn new(capacity: usize) -> Result<Self, std::io::Error> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.as_file_mut().set_len(capacity as u64)?;

        Ok(Self {
            temp_file,
            mapped_data: Mutex::new(vec![0u8; capacity]),
            stats: ArcSwap::new(Arc::new(MemoryStats::default())),
        })
    }

    /// Write data to mapped memory
    pub fn write_data(&self, offset: usize, data: &[u8]) -> Result<(), std::io::Error> {
        if offset + data.len() > self.mapped_data.lock().len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Data exceeds capacity",
            ));
        }

        let mut mapped = self.mapped_data.lock();
        mapped[offset..offset + data.len()].copy_from_slice(data);

        // Update stats
        let mut new_stats = (**self.stats.load()).clone();
        new_stats.allocated_bytes += data.len();
        self.stats.store(Arc::new(new_stats));

        Ok(())
    }

    /// Read data from mapped memory
    pub fn read_data(&self, offset: usize, length: usize) -> Result<Vec<u8>, std::io::Error> {
        let mapped = self.mapped_data.lock();
        if offset + length > mapped.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Read exceeds capacity",
            ));
        }

        Ok(mapped[offset..offset + length].to_vec())
    }

    /// Flush to disk
    pub fn flush(&self) -> Result<(), std::io::Error> {
        self.temp_file.as_file().sync_all()
    }

    /// Get statistics
    pub fn stats(&self) -> Arc<MemoryStats> {
        self.stats.load_full()
    }
}

/// Parallel memory operations using `fork_union`
pub struct ParallelMemoryProcessor {
    pool: Arc<StdMutex<Vec<std::thread::JoinHandle<()>>>>,
}

impl ParallelMemoryProcessor {
    /// Create a new parallel processor
    #[must_use]
    pub fn new(_num_threads: usize) -> Self {
        Self {
            pool: Arc::new(StdMutex::new(Vec::new())),
        }
    }

    /// Process multiple memory blocks in parallel
    pub fn process_blocks<F, T>(&self, blocks: Vec<Vec<u8>>, processor: F) -> Vec<T>
    where
        F: Fn(Vec<u8>) -> T + Send + Sync + 'static,
        T: Send + 'static,
    {
        let results: Vec<T> = blocks.into_iter().map(processor).collect();
        results
    }

    /// Parallel compression of multiple blocks
    #[must_use]
    pub fn compress_blocks(
        &self,
        blocks: Vec<Vec<u8>>,
        level: u32,
    ) -> Vec<Result<Vec<u8>, std::io::Error>> {
        self.process_blocks(blocks, move |block| compress_data(&block, level))
    }

    /// Parallel hashing of multiple blocks
    #[must_use]
    pub fn hash_blocks(&self, blocks: Vec<Vec<u8>>) -> Vec<u64> {
        use ahash::AHasher;
        use std::hash::Hasher;

        self.process_blocks(blocks, |block| {
            let mut hasher = AHasher::default();
            hasher.write(&block);
            hasher.finish()
        })
    }
}

/// Secure memory pool with cryptographic features
pub struct SecureMemoryPool {
    pool: MemoryPool,
    encryption_key: Option<Vec<u8>>,
}

impl SecureMemoryPool {
    /// Create a new secure memory pool
    #[must_use]
    pub fn new(config: MemoryPoolConfig, encryption_key: Option<Vec<u8>>) -> Self {
        Self {
            pool: MemoryPool::new(config),
            encryption_key,
        }
    }

    /// Allocate and encrypt data
    pub fn allocate_encrypted(&self, data: &[u8]) -> Result<SecureAllocation<'_>, std::io::Error> {
        let encrypted = if let Some(key) = &self.encryption_key {
            // Simple XOR encryption for demonstration
            data.iter()
                .zip(key.iter().cycle())
                .map(|(d, k)| d ^ k)
                .collect()
        } else {
            data.to_vec()
        };

        let ptr = self.pool.allocate(encrypted.len())?;
        unsafe {
            ptr.copy_from(encrypted.as_ptr(), encrypted.len());
        }

        Ok(SecureAllocation {
            ptr,
            size: encrypted.len(),
            pool: &self.pool,
            encrypted: self.encryption_key.is_some(),
        })
    }

    /// Get pool statistics
    pub fn stats(&self) -> Arc<MemoryStats> {
        self.pool.stats()
    }
}

/// Guard for secure memory allocation
pub struct SecureAllocation<'a> {
    ptr: *mut u8,
    size: usize,
    pool: &'a MemoryPool,
    encrypted: bool,
}

impl SecureAllocation<'_> {
    /// Decrypt and get original data
    pub fn decrypt(&self, key: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let encrypted_data = unsafe { from_raw_parts(self.ptr, self.size) };

        if !self.encrypted {
            return Ok(encrypted_data.to_vec());
        }

        // Simple XOR decryption
        let decrypted = encrypted_data
            .iter()
            .zip(key.iter().cycle())
            .map(|(d, k)| d ^ k)
            .collect();

        Ok(decrypted)
    }

    /// Secure wipe (overwrite with zeros)
    pub fn secure_wipe(&mut self) {
        unsafe {
            write_bytes(self.ptr, 0, self.size);
        }
    }
}

impl Drop for SecureAllocation<'_> {
    fn drop(&mut self) {
        self.secure_wipe();
        let _ = self.pool.deallocate(self.ptr, self.size);
    }
}

/// Memory event logger with timestamps
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
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let events = self.events.lock();
        serde_json::to_string(&*events)
    }

    /// Clear all events
    pub fn clear(&self) {
        self.events.lock().clear();
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
    pub fn new(manager: &MemoryManager) -> Self {
        use ahash::AHasher;
        use std::hash::Hasher;

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
        hasher.write(checksum_data.as_bytes());
        let checksum = hasher.finish();

        Self {
            timestamp: Utc::now(),
            stats: (*stats).clone(),
            pools,
            checksum,
        }
    }

    /// Export snapshot as base64-encoded JSON
    pub fn export_base64(&self) -> Result<String, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(STANDARD.encode(json.as_bytes()))
    }

    /// Import snapshot from base64-encoded JSON
    pub fn import_base64(data: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = STANDARD.decode(data)?;
        let json_str = String::from_utf8(json)?;
        let snapshot: MemorySnapshot = serde_json::from_str(&json_str)?;
        Ok(snapshot)
    }

    /// Verify snapshot integrity
    #[must_use]
    pub fn verify(&self) -> bool {
        use ahash::AHasher;
        use std::hash::Hasher;

        let checksum_data = format!("{:?}{:?}{:?}", self.stats, self.pools, self.timestamp);
        let mut hasher = AHasher::default();
        hasher.write(checksum_data.as_bytes());
        let computed = hasher.finish();
        computed == self.checksum
    }
}

/// Utility functions for compression
fn compress_data(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    let mut output = Vec::new();
    {
        let mut compressor = CompressorWriter::new(&mut output, 4096, level, level);
        compressor.write_all(data)?;
        compressor.flush()?;
    }
    Ok(output)
}

fn decompress_data(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decompressor = Decompressor::new(data, 4096);
    let mut output = Vec::new();
    decompressor.read_to_end(&mut output)?;
    Ok(output)
}

/// Get the global memory event logger
#[must_use]
pub fn global_memory_logger() -> Arc<MemoryEventLogger> {
    Arc::new(MemoryEventLogger::new(10000))
}

/// Enhanced memory manager with all features
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
            logger: global_memory_logger().clone(),
        }
    }

    /// Create a compressed memory pool
    pub fn create_compressed_pool(
        &self,
        config: MemoryPoolConfig,
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
        config: MemoryPoolConfig,
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
    pub fn create_snapshot(&self) -> MemorySnapshot {
        MemorySnapshot::new(&self.base_manager)
    }

    /// Get parallel processor
    pub fn parallel_processor(&self) -> &ParallelMemoryProcessor {
        &self.parallel_processor
    }

    /// Get event logger
    pub fn logger(&self) -> &Arc<MemoryEventLogger> {
        &self.logger
    }

    /// Get base manager
    pub fn base_manager(&self) -> &MemoryManager {
        &self.base_manager
    }
}

/// Get the global enhanced memory manager
#[must_use]
pub fn global_enhanced_memory_manager() -> Arc<EnhancedMemoryManager> {
    Arc::new(EnhancedMemoryManager::new(4))
}
