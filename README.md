# trash_parallelism

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Azzybana Raccoon's comprehensive parallelism library.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Building](#building)
- [Testing](#testing)
- [Documentation](#documentation)
- [Project Structure](#project-structure)
- [Performance Characteristics](#performance-characteristics)
- [Safety & Reliability](#safety--reliability)
- [Platform Support](#platform-support)
- [Dependencies](#dependencies)
- [Contributing](#contributing)
- [Authors](#authors)
- [License](#license)

A high-performance Rust library providing comprehensive async, threading, memory management, and utility functions for building efficient applications. Built with performance and ergonomics in mind.

## Features

### Core Modules

- **`parallel`** - High-performance parallel processing utilities with monitoring and async support
- **`async`** - Asynchronous operation helpers, task management, and concurrency utilities
- **`channels`** - Advanced channel implementations with monitoring, compression, and batching
- **`chars`** - String and character processing utilities with performance optimizations
- **`common`** - Shared utilities for hashing, serialization, and data manipulation
- **`data`** - Data processing and transformation functions
- **`io`** - File I/O operations with async support and performance monitoring
- **`memory`** - Memory management utilities with pools, monitoring, and optimization
- **`serde`** - Serialization utilities with JSON, base64, and custom formats
- **`sys`** - System-level utilities for environment, paths, and time handling

### Key Capabilities

- **Async Operations**: High-performance async utilities with smol, futures-lite, and crossfire
- **Threading**: Parallel processing with work-stealing schedulers and fork-join patterns
- **Memory Management**: Efficient allocation with mimalloc and custom memory pools
- **Channel Communication**: Advanced channel communication with monitoring and serialization
- **System Utilities**: Time handling, environment variables, file system operations
- **Data Processing**: Parsing, serialization, and data manipulation
- **I/O Operations**: File and network I/O with async support
- **Utilities**: Compression, hashing, JSON handling, and more

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
trash_parallelism = { git = "https://github.com/Azzybana/trash_parallelism" }
```

Note: This library is currently set to `publish = false` in Cargo.toml, so it's intended for local use only.

## Usage

### Basic Usage

```rust
use trash_parallelism::*;

// Async task spawning
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
spawn_task!("my_task", async {
    println!("Hello from async task!");
    Ok(())
});
# Ok(())
# }

// Parallel processing
let data = vec![1, 2, 3, 4, 5];
let result = parallel_map(data, |x| x * 2);
assert_eq!(result, vec![2, 4, 6, 8, 10]);

// System utilities
let now = current_utc_time();
let home = read_env_var("HOME").unwrap_or_default();
println!("Current time: {}, Home: {}", now, home);
# }
```

### Advanced Example: High-Performance Data Processing

```rust,no_run
use trash_parallelism::*;
use smol;

// Parallel data processing pipeline
# smol::block_on(async {
let raw_data = vec![1u32; 10000];

// Process in parallel
let processed = parallel_map(raw_data, |x| x * x + 1);

// Serialize to JSON asynchronously
let json_data = serde::serialize_to_json(&processed).unwrap();

// Compress and save
let compressed = io::utils::compress_data_brotli(json_data.as_bytes(), 6).await.unwrap();
io::utils::write_file_async("processed_data.gz", &compressed).await.unwrap();

println!("Processed {} elements", processed.len());
# });
```

### Memory Management Example

```rust
use trash_parallelism::memory::*;

// Custom memory pool allocation
let pool_name = "processing_pool";
create_memory_pool(pool_name, 1024 * 1024).unwrap(); // 1MB pool

// Allocate from pool
let ptr = alloc_from_pool!(pool_name, 1024).unwrap();

// Use allocated memory safely
unsafe {
    std::ptr::write_bytes(ptr, 0, 1024); // Initialize to zero
}

// Pool automatically manages cleanup
```

### Channel-Based Communication

```rust,no_run
use trash_parallelism::channels::*;
use smol;

// Create monitored channel
# smol::block_on(async {
let (tx, rx, monitor) = create_monitored_channel::<String>(10);

// Send messages
for i in 0..5 {
    send_async(&tx, format!("Message {}", i)).await.unwrap();
}

// Receive and process
for _ in 0..5 {
    let msg = recv_async(&rx).await.unwrap();
    println!("Received: {}", msg);
}

// Check performance stats
let stats = monitor.get_stats();
println!("Processed {} messages", stats.messages_sent);
# });
```

## Project Structure

```
src/
├── lib.rs              # Main library interface
├── parallel.rs         # Parallel processing utilities
├── async.rs           # Async operation helpers
├── channels.rs        # Channel implementations
├── chars.rs           # String processing utilities
├── common.rs          # Common utilities
├── data.rs            # Data processing functions
├── io.rs              # I/O operations
├── memory.rs          # Memory management
├── serde.rs           # Serialization utilities
└── sys.rs             # System utilities
async/
├── core.rs
├── data.rs
├── patterns.rs
└── tasks.rs
channels/
├── core.rs
├── monitoring.rs
├── multiplexor.rs
├── parsers.rs
├── queue.rs
└── specialist.rs
chars/
├── core.rs
└── processing.rs
common/
├── crypto.rs
├── json.rs
└── utils.rs
io/
├── parallelism.rs
├── streams.rs
├── utils.rs
└── writers.rs
memory/
├── features.rs
├── manager.rs
├── pool.rs
└── stats.rs
parallel/
├── advanced.rs
├── core.rs
├── data.rs
└── organize.rs
serde/
├── base64.rs
├── bytes.rs
├── json.rs
├── streaming.rs
└── utility.rs
sys/
├── datetime.rs
├── env.rs
└── path.rs
```

## Performance Characteristics

- **Zero-Copy Operations**: Where possible, avoids unnecessary allocations
- **Async-First Design**: Built for non-blocking I/O and concurrency
- **Memory Efficient**: Custom allocators and pooling reduce overhead
- **Parallel Processing**: Automatic parallelization for CPU-bound tasks
- **Monitoring**: Built-in performance tracking and statistics

## Safety & Reliability

- **Memory Safe**: All operations are memory-safe with no undefined behavior
- **Thread Safe**: Concurrent operations are properly synchronized
- **Error Handling**: Comprehensive error propagation with context
- **Resource Management**: Automatic cleanup and RAII patterns
- **Testing**: Extensive test coverage for reliability

## Platform Support

- **Linux**: Full support with optimized system calls
- **macOS**: Full support with native optimizations
- **Windows**: Full support with Windows-specific implementations
- **Cross-Platform**: Consistent API across all platforms

## Dependencies

Key external crates:
- `crossfire`: Channel communication
- `parking_lot`: Efficient synchronization primitives
- `smol`: Async runtime
- `smol-cancellation-token`: Cancellation for async tasks
- `futures-lite`: Lightweight futures utilities
- `serde`: Serialization framework
- `serde_json`: JSON serialization
- `schemars`: JSON schema generation
- `memchr`: High-performance string search
- `bytes`: Byte buffer management
- `ahash`: High-performance hashing
- `mimalloc`: Memory allocator
- `chrono`: Date and time handling
- `base64`: Base64 encoding/decoding
- `tempfile`: Temporary file creation
- `fork_union`: Thread pool and parallel execution
- `brotli`: Compression
- `arc-swap`: Atomic reference counting
- `once_cell`: Lazy initialization

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Authors

- **Azzybana Raccoon** - *Initial work* - [GitHub](https://github.com/azzybana)

## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.