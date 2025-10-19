# trash_utilities

A comprehensive Rust utility library providing high-performance computing tools, async operations, data processing utilities, and more.

## Overview

This library offers a collection of modular utilities designed for modern Rust applications, with a focus on performance, composability, and ease of use. The library is organized into specialized modules, each providing specific functionality.

## Features

### Core Modules

- **`parallel`** - High-performance parallel processing utilities with monitoring and async support
- **`async`** - Asynchronous operation helpers, task management, and concurrency utilities
- **`channels`** - Advanced channel implementations with monitoring, compression, and batching
- **`chars`** - String and character processing utilities with performance optimizations
- **`common`** - Shared utilities for hashing, serialization, and data manipulation
- **`data`** - Data processing and transformation functions
- **`io`** - File I/O operations with async support and performance monitoring
- **`logging`** - Structured logging with performance tracking and metrics
- **`memory`** - Memory management utilities with pools, monitoring, and optimization
- **`serde`** - Serialization utilities with JSON, base64, and custom formats
- **`sys`** - System-level utilities for environment, paths, and time handling

### Key Capabilities

- **Parallel Processing**: CPU-intensive operations with monitoring and cancellation support
- **Async Operations**: Non-blocking I/O and concurrent task execution
- **Data Processing**: Efficient filtering, mapping, grouping, and transformation
- **Memory Management**: Custom pools, monitoring, and optimization strategies
- **Channel Communication**: High-performance message passing with compression
- **Performance Monitoring**: Built-in timing, statistics, and observability

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
trash_utilities = { path = "path/to/trash_utilities" }
```

Or for external use:

```toml
[dependencies]
trash_utilities = "0.1.0"
```

## Usage

### Parallel Processing

```rust
use trash_utilities::parallel::*;

// Transform data in parallel
let data = vec![1, 2, 3, 4, 5];
let doubled = parallel_map(data, |x| x * 2);

// Filter with monitoring
let monitor = ThreadPoolMonitor::new();
let result = monitored_execute(&monitor, "filter_operation", || {
    parallel_filter(vec![1, 2, 3, 4, 5], |&x| x > 3)
});
```

### Async Operations

```rust
use trash_utilities::async::*;

// Create and manage async tasks
let spawner = AsyncTaskSpawner::new();
spawner.spawn(|| async {
    // Your async work here
});

// Process with timeout
let result = with_timeout(Duration::from_secs(5), async {
    // Operation that might take time
    42
}).await;
```

### Channel Communication

```rust
use trash_utilities::channels::*;

// Create monitored channels
let (tx, rx) = create_monitored_channel(100);

// Send structured messages
let message = Message::new(Payload { data: "hello" });
tx.send_async(message).await?;
```

### Memory Management

```rust
use trash_utilities::memory::*;

// Create memory pools
let pool = MemoryPool::new(MemoryPoolConfig {
    initial_size: 1024,
    max_size: Some(10240),
    alignment: 8,
    name: "my_pool".to_string(),
});

// Monitor memory usage
let manager = MemoryManager::new();
let stats = manager.global_stats();
```

## Building

```bash
cargo build
```

For optimized release build:

```bash
cargo build --release
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with performance benchmarks:

```bash
cargo test --release -- --nocapture
```

## Documentation

Generate and view documentation:

```bash
cargo doc --open
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
├── logging.rs         # Logging utilities
├── memory.rs          # Memory management
├── serde.rs           # Serialization utilities
└── sys.rs             # System utilities
```

## Performance Considerations

- **Parallel Operations**: Currently use sequential processing but are designed for easy parallel backend integration
- **Memory Usage**: Most operations are streaming and memory-efficient
- **Async Operations**: Use Smol runtime for efficient async execution
- **Monitoring**: Built-in performance tracking and metrics collection

## Dependencies

Key external crates:
- `fork_union`: Thread pool and parallel execution
- `parking_lot`: Efficient synchronization primitives
- `smol`: Async runtime
- `tracing`: Structured logging
- `serde`: Serialization framework
- `ahash`: High-performance hashing
- `bytes`: Byte buffer management

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.