/// Core memory pools and allocation structures.
///
/// This module provides the fundamental memory allocation components
/// including memory pools, arenas, and thread-local caches for efficient
/// memory management.
///
/// ## Features
///
/// - **Memory Pools**: Efficient allocation with size limits and statistics
/// - **Memory Arenas**: Bulk allocation with fast reset capabilities
/// - **Thread-Local Caches**: Per-thread object caching for performance
/// - **Statistics Tracking**: Detailed allocation/deallocation metrics
/// - **Safety Guards**: Automatic cleanup and bounds checking
///
/// ## Examples
///
/// ### Memory Pool Usage
/// ```rust
/// use trash_utilities::memory::*;
///
/// // Create a memory pool configuration
/// let config = MemoryPoolConfig {
///     initial_size: 1024 * 1024,         // 1MB
///     max_size: Some(10 * 1024 * 1024),  // 10MB limit
///     alignment: 8,
///     name: "my_pool".to_string(),
/// };
///
/// let pool = MemoryPool::new(config);
///
/// // Allocate memory
/// let ptr = pool.allocate(1024).unwrap();
/// // ... use the memory ...
///
/// // Deallocate when done
/// pool.deallocate(ptr, 1024).unwrap();
///
/// // Check statistics
/// let stats = pool.stats();
/// println!("Allocated: {} bytes", stats.allocated_bytes);
/// ```
///
/// ### Memory Arena for Bulk Operations
/// ```rust
/// use trash_utilities::memory::*;
///
/// let mut arena = MemoryArena::new(64 * 1024, "bulk_ops".to_string()); // 64KB
///
/// // Allocate multiple items quickly
/// let data1 = arena.allocate(1024).unwrap();
/// let data2 = arena.allocate(2048).unwrap();
///
/// // Use the data...
///
/// // Reset arena for reuse (much faster than individual deallocations)
/// arena.reset();
/// println!("Arena usage: {} bytes", arena.usage());
/// ```
///
/// ### Thread-Local Cache
/// ```rust
/// use trash_utilities::memory::*;
///
/// let cache = ThreadLocalCache::new(10, || Vec::new());
///
/// // Get a vector from cache or create new
/// let mut vec = cache.get_or_create();
/// vec.push(42);
///
/// // Return to cache for reuse
/// cache.return_item(vec);
///
/// println!("Cache size: {}", cache.size());
/// ```

// Standard library imports
use std::{
    alloc::{Layout, dealloc},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

// External crate imports
use ahash::AHashMap;
use arc_swap::ArcSwap;
use parking_lot::Mutex;

// Local imports
use super::{MemoryPoolConfig, MemoryStats, calc_ratio};

/// Memory pool for efficient allocations
#[derive(Debug)]
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
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the pool is inactive or size limits are exceeded.
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
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the layout is invalid.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
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
    #[must_use]
    pub fn stats(&self) -> Arc<MemoryStats> {
        self.stats.load_full()
    }

    /// Get pool configuration
    #[must_use]
    pub fn config(&self) -> &MemoryPoolConfig {
        &self.config
    }

    /// Check if pool is active
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Deactivate the pool
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Get fragmentation ratio
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn fragmentation_ratio(&self) -> f64 {
        let stats = self.stats.load();
        calc_ratio(stats.allocated_bytes, stats.heap_size)
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new(MemoryPoolConfig {
            initial_size: 1024 * 1024,
            max_size: Some(100 * 1024 * 1024),
            alignment: 8,
            name: "default".to_string(),
        })
    }
}

/// Memory arena for bulk allocations
#[derive(Debug)]
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
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the arena is full.
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
    #[must_use]
    pub fn usage(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }

    /// Get capacity
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get arena name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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
    #[must_use]
    pub fn size(&self) -> usize {
        self.cache.lock().len()
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.lock().clear();
    }

    /// Check if cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.lock().is_empty()
    }
}
