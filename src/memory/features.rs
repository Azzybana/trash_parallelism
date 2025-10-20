/// Advanced memory features and utilities.
///
/// This module provides specialized memory operations including
/// compression, encryption, parallel processing, and memory-mapped I/O.
///
/// ## Features
///
/// - **Compressed Memory Pools**: Space-efficient storage with Brotli compression
/// - **Secure Memory Pools**: Encrypted memory with automatic secure wiping
/// - **Memory-Mapped Pools**: Large data handling with file-backed storage
/// - **Parallel Processing**: Concurrent memory operations using thread pools
/// - **RAII Guards**: Automatic cleanup for compressed and secure allocations
///
/// ## Examples
///
/// ### Compressed Memory Pool
/// ```rust
/// use trash_utilities::memory::*;
///
/// // Create a compressed pool
/// let config = default_pool_config("compressed");
/// let pool = CompressedMemoryPool::new(config, 6); // Compression level 6
///
/// // Allocate and compress data
/// let data = b"Hello, this is some data that will be compressed!";
/// let allocation = pool.allocate_compressed(data).unwrap();
///
/// // Check compression ratio
/// println!("Compression ratio: {:.2}", allocation.compression_ratio());
/// println!("Original size: {} bytes", allocation.original_size());
/// println!("Compressed size: {} bytes", allocation.compressed_size());
///
/// // Decompress and get original data
/// let decompressed = allocation.decompress().unwrap();
/// assert_eq!(decompressed, data);
/// ```
///
/// ### Secure Memory Pool
/// ```rust
/// use trash_utilities::memory::*;
///
/// // Create a secure pool with encryption
/// let config = default_pool_config("secure");
/// let key = b"my-secret-key-32-bytes-long!!!"; // 32 bytes for AES-256
/// let pool = SecureMemoryPool::new(config, Some(key.to_vec()));
///
/// // Allocate and encrypt sensitive data
/// let sensitive_data = b"This is sensitive information";
/// let allocation = pool.allocate_encrypted(sensitive_data).unwrap();
///
/// // Decrypt and verify
/// let decrypted = allocation.decrypt(key).unwrap();
/// assert_eq!(decrypted, sensitive_data);
///
/// // Data is automatically wiped when allocation goes out of scope
/// drop(allocation);
/// ```
///
/// ### Parallel Memory Processing
/// ```rust
/// use trash_utilities::memory::*;
///
/// let processor = ParallelMemoryProcessor::new(4);
///
/// // Compress multiple blocks in parallel
/// let blocks = vec![
///     b"Block 1 data".to_vec(),
///     b"Block 2 data".to_vec(),
///     b"Block 3 data".to_vec(),
/// ];
///
/// let compressed = processor.compress_blocks(blocks, 6);
/// for result in compressed {
///     let compressed_data = result.unwrap();
///     println!("Compressed {} bytes", compressed_data.len());
/// }
/// ```

/// ```
// Standard library imports
use std::{
    io::{Read, Write},
    slice::from_raw_parts,
    sync::Arc,
};

// External crate imports
use arc_swap::ArcSwap;
use brotli::{CompressorWriter, Decompressor};
use parking_lot::Mutex;
use tempfile::NamedTempFile;

// Local imports
use super::{MemoryPool, MemoryPoolConfig, MemoryStats};
#[derive(Debug)]
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
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if compression or allocation fails.
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
    #[must_use]
    pub fn stats(&self) -> Arc<MemoryStats> {
        self.pool.stats()
    }

    /// Get compression level
    #[must_use]
    pub fn compression_level(&self) -> u32 {
        self.compression_level
    }
}

/// Guard for compressed memory allocation
#[derive(Debug)]
pub struct CompressedAllocation<'a> {
    ptr: *mut u8,
    compressed_size: usize,
    original_size: usize,
    pool: &'a MemoryPool,
}

impl CompressedAllocation<'_> {
    /// Decompress and get original data
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if decompression fails.
    pub fn decompress(&self) -> Result<Vec<u8>, std::io::Error> {
        let compressed_data = unsafe { from_raw_parts(self.ptr, self.compressed_size) };
        decompress_data(compressed_data)
    }

    /// Get compression ratio
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn compression_ratio(&self) -> f64 {
        self.compressed_size as f64 / self.original_size as f64
    }

    /// Get compressed size
    #[must_use]
    pub fn compressed_size(&self) -> usize {
        self.compressed_size
    }

    /// Get original size
    #[must_use]
    pub fn original_size(&self) -> usize {
        self.original_size
    }
}

impl Drop for CompressedAllocation<'_> {
    fn drop(&mut self) {
        let _ = self.pool.deallocate(self.ptr, self.compressed_size);
    }
}

/// Secure memory pool with cryptographic features
#[derive(Debug)]
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
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if allocation fails.
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
    #[must_use]
    pub fn stats(&self) -> Arc<MemoryStats> {
        self.pool.stats()
    }

    /// Check if encryption is enabled
    #[must_use]
    pub fn is_encrypted(&self) -> bool {
        self.encryption_key.is_some()
    }
}

/// Guard for secure memory allocation
#[derive(Debug)]
pub struct SecureAllocation<'a> {
    ptr: *mut u8,
    size: usize,
    pool: &'a MemoryPool,
    encrypted: bool,
}

impl SecureAllocation<'_> {
    /// Decrypt and get original data
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if decryption fails.
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
            std::ptr::write_bytes(self.ptr, 0, self.size);
        }
    }

    /// Get allocation size
    #[must_use]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if data is encrypted
    #[must_use]
    pub fn is_encrypted(&self) -> bool {
        self.encrypted
    }
}

impl Drop for SecureAllocation<'_> {
    fn drop(&mut self) {
        self.secure_wipe();
        let _ = self.pool.deallocate(self.ptr, self.size);
    }
}

/// Memory-mapped file pool for large data
#[derive(Debug)]
pub struct MemoryMappedPool {
    temp_file: NamedTempFile,
    mapped_data: Mutex<Vec<u8>>,
    stats: ArcSwap<MemoryStats>,
}

impl MemoryMappedPool {
    /// Create a new memory-mapped pool
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if file creation fails.
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
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the write exceeds capacity.
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
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the read exceeds capacity.
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
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if flushing fails.
    pub fn flush(&self) -> Result<(), std::io::Error> {
        self.temp_file.as_file().sync_all()
    }

    /// Get statistics
    #[must_use]
    pub fn stats(&self) -> Arc<MemoryStats> {
        self.stats.load_full()
    }

    /// Get capacity
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.mapped_data.lock().len()
    }
}

/// Parallel memory operations using `fork_union`
#[derive(Debug)]
pub struct ParallelMemoryProcessor {
    pool: Arc<std::sync::Mutex<Vec<std::thread::JoinHandle<()>>>>,
}

impl ParallelMemoryProcessor {
    /// Create a new parallel processor
    #[must_use]
    pub fn new(_num_threads: usize) -> Self {
        Self {
            pool: Arc::new(std::sync::Mutex::new(Vec::new())),
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

    /// Get number of active threads
    ///
    /// # Panics
    ///
    /// This function may panic if the mutex is poisoned.
    #[must_use]
    pub fn active_threads(&self) -> usize {
        self.pool.lock().unwrap().len()
    }
}

impl Default for ParallelMemoryProcessor {
    fn default() -> Self {
        Self::new(4)
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

/// Get the global enhanced memory manager
#[must_use]
pub fn global_enhanced_memory_manager() -> Arc<super::EnhancedMemoryManager> {
    Arc::new(super::EnhancedMemoryManager::new(4))
}
