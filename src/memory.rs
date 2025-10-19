/// Memory management utilities and data structures.
///
/// This module provides comprehensive memory management capabilities
/// including pools, arenas, statistics, compression, and security features.
// Standard library imports
use std::alloc::Layout;

// External crate imports
use mimalloc::MiMalloc;

// Submodule declarations
pub mod features;
pub mod manager;
pub mod pool;
pub mod stats;

// Re-exports for backward compatibility
pub use features::*;
pub use manager::*;
pub use pool::*;
pub use stats::*;

// Global mimalloc allocator instance
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Memory pool configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryPoolConfig {
    pub initial_size: usize,
    pub max_size: Option<usize>,
    pub alignment: usize,
    pub name: String,
}

/// Calculate ratio after reducing both numerator and denominator by 20%.
///
/// This uses integer arithmetic (multiply by 4, divide by 5) to avoid floating-point
/// operations, and reduces the values to help fit within f64's 53-bit mantissa precision
/// (values up to ~9e15). The 20% reduction is arbitrary but helps prevent overflow
/// and precision loss when casting large usize values to f64.
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

/// Get mimalloc statistics (extended features)
#[must_use]
pub fn get_mimalloc_stats() -> Option<usize> {
    // Mimalloc extended stats are not directly exposed in the Rust wrapper
    // This is a placeholder for when extended stats become available
    None
}

/// Utility function to allocate aligned memory
///
/// # Errors
///
/// Returns an `std::io::Error` if allocation fails.
pub fn allocate_aligned(size: usize, alignment: usize) -> Result<*mut u8, std::io::Error> {
    let layout = Layout::from_size_align(size, alignment)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let ptr = unsafe { std::alloc::alloc(layout) };
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
///
/// # Errors
///
/// Returns an `std::io::Error` if the layout is invalid.
///
/// # Safety
///
/// This function is `unsafe` because it dereferences a raw pointer and requires the
/// caller to ensure the following:
/// - `ptr` must be either null or a pointer previously returned by `allocate_aligned`
///   (or otherwise allocated with the same `Layout`).
/// - `size` and `alignment` must match the original allocation's layout.
/// - The memory referenced by `ptr` must not be used after calling this function.
///
/// Callers must call this function inside an `unsafe` block.
pub unsafe fn deallocate_aligned(
    ptr: *mut u8,
    size: usize,
    alignment: usize,
) -> Result<(), std::io::Error> {
    if ptr.is_null() {
        return Ok(());
    }

    let layout = Layout::from_size_align(size, alignment)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    // SAFETY: Caller guarantees `ptr` was allocated with `layout` or is null.
    unsafe {
        std::alloc::dealloc(ptr, layout);
    }
    Ok(())
}

/// Memory usage guard that tracks and limits usage
#[derive(Debug)]
pub struct MemoryUsageGuard {
    max_bytes: usize,
    current_bytes: std::sync::atomic::AtomicUsize,
}

impl MemoryUsageGuard {
    /// Create a new memory usage guard
    #[must_use]
    pub fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            current_bytes: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Try to allocate bytes
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the memory limit is exceeded.
    pub fn try_allocate(&self, bytes: usize) -> Result<MemoryAllocationGuard<'_>, std::io::Error> {
        let current = self
            .current_bytes
            .load(std::sync::atomic::Ordering::Relaxed);
        if current + bytes > self.max_bytes {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "Memory limit exceeded",
            ));
        }

        self.current_bytes
            .fetch_add(bytes, std::sync::atomic::Ordering::Relaxed);
        Ok(MemoryAllocationGuard { bytes, guard: self })
    }

    /// Get current usage
    #[must_use]
    pub fn current_usage(&self) -> usize {
        self.current_bytes
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get maximum allowed bytes
    #[must_use]
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    fn deallocate(&self, bytes: usize) {
        self.current_bytes
            .fetch_sub(bytes, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Guard for memory allocation that automatically deallocates on drop
#[derive(Debug)]
pub struct MemoryAllocationGuard<'a> {
    bytes: usize,
    guard: &'a MemoryUsageGuard,
}

impl Drop for MemoryAllocationGuard<'_> {
    fn drop(&mut self) {
        self.guard.deallocate(self.bytes);
    }
}
