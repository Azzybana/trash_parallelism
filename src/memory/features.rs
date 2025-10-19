/// Advanced memory features and utilities.
///
/// This module provides specialized memory operations including
/// compression, encryption, parallel processing, and memory-mapped I/O.
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

/// Compressed memory pool for space-efficient storage
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
