/// Advanced channel communication utilities for high-performance async messaging.
///
/// This module provides comprehensive channel-based communication primitives designed
/// for high-throughput, low-latency message passing between async tasks and threads.
/// Built on crossfire MPMC channels and smol async runtime for optimal performance.
///
/// ## Features
///
/// - **Multiple Channel Types**: Bounded, unbounded, and specialized channels
/// - **Message Monitoring**: Performance tracking and statistics collection
/// - **Multiplexing**: Route messages to multiple consumers efficiently
/// - **Queue Management**: Priority queues and specialized message handling
/// - **Specialized Channels**: Domain-specific channel implementations
/// - **Message Parsing**: Protocol-aware message processing and validation
/// - **Async Integration**: Seamless integration with smol async runtime
///
/// ## Architecture
///
/// ### Core Components
/// - **`core`**: Fundamental channel primitives and basic operations
/// - **`monitoring`**: Performance monitoring and statistics collection
/// - **`multiplexor`**: Message routing and fan-out capabilities
/// - **`queue`**: Advanced queue implementations with priorities
/// - **`specialist`**: Domain-specific channel specializations
/// - **`parsers`**: Message parsing and protocol handling
///
/// ## Examples
///
/// ### Basic Channel Communication
/// ```rust
/// use trash_utilities::channels::{bounded_queue_3, send_async, recv_async};
/// use smol;
///
/// # smol::block_on(async {
/// // Create a bounded channel
/// let (tx, rx) = bounded_queue_3::<String>(10);
///
/// // Send a message asynchronously
/// send_async(&tx, "Hello, channels!".to_string()).await.unwrap();
///
/// // Receive the message
/// let message = recv_async(&rx).await.unwrap();
/// assert_eq!(message, "Hello, channels!");
/// # });
/// ```
///
/// ### Message Monitoring
/// ```rust,no_run
/// use trash_utilities::channels::{create_monitored_channel, ChannelMonitor};
/// use smol;
///
/// # smol::block_on(async {
/// // Create a monitored channel
/// let (tx, rx, monitor) = create_monitored_channel::<String>(10);
///
/// // Send some messages
/// for i in 0..5 {
///     send_async(&tx, format!("Message {}", i)).await.unwrap();
/// }
///
/// // Check monitoring statistics
/// let stats = monitor.get_stats();
/// println!("Messages sent: {}", stats.messages_sent);
/// println!("Channel utilization: {:.1}%", stats.utilization_percent());
/// # });
/// ```
///
/// ### Message Multiplexing
/// ```rust,no_run
/// use trash_utilities::channels::{create_multiplexer, Multiplexer};
/// use smol;
///
/// # smol::block_on(async {
/// // Create a multiplexer for broadcasting messages
/// let mut multiplexer = create_multiplexer::<String>();
///
/// // Add multiple receivers
/// let rx1 = multiplexer.add_receiver();
/// let rx2 = multiplexer.add_receiver();
/// let tx = multiplexer.get_sender();
///
/// // Send a message (received by both consumers)
/// send_async(&tx, "Broadcast message".to_string()).await.unwrap();
///
/// // Both receivers get the message
/// let msg1 = recv_async(&rx1).await.unwrap();
/// let msg2 = recv_async(&rx2).await.unwrap();
/// assert_eq!(msg1, msg2);
/// # });
/// ```
///
/// ### Priority Queue
/// ```rust,no_run
/// use trash_utilities::channels::{create_priority_queue, PriorityMessage};
/// use smol;
///
/// # smol::block_on(async {
/// // Create a priority queue
/// let (tx, rx) = create_priority_queue::<String>(10);
///
/// // Send messages with different priorities
/// tx.send(PriorityMessage::high("Urgent!".to_string())).await.unwrap();
/// tx.send(PriorityMessage::low("Normal message".to_string())).await.unwrap();
/// tx.send(PriorityMessage::medium("Medium priority".to_string())).await.unwrap();
///
/// // High priority message received first
/// let urgent = rx.recv().await.unwrap();
/// assert_eq!(urgent.data, "Urgent!");
/// # });
/// ```
///
/// ### Protocol-Aware Parsing
/// ```rust,no_run
/// use trash_utilities::channels::{create_protocol_parser, ProtocolParser};
/// use smol;
///
/// # smol::block_on(async {
/// // Create a parser for a custom protocol
/// let (tx, rx) = create_protocol_parser::<String, ParsedMessage>(10);
///
/// // Send raw protocol messages
/// send_async(&tx, "CMD:LOGIN user=alice".to_string()).await.unwrap();
/// send_async(&tx, "DATA:Hello World".to_string()).await.unwrap();
///
/// // Receive parsed messages
/// let login_msg = recv_async(&rx).await.unwrap();
/// let data_msg = recv_async(&rx).await.unwrap();
///
/// match login_msg {
///     ParsedMessage::Command(cmd) => println!("Command: {}", cmd),
///     _ => {}
/// }
/// # });
/// ```
// Re-export core types and functions
pub mod core;
pub use core::*;

// Re-export monitoring types
pub mod monitoring;
pub use monitoring::*;

// Re-export multiplexer types
pub mod multiplexor;
pub use multiplexor::*;

// Re-export queue types
pub mod queue;
pub use queue::*;

// Re-export specialist types
pub mod specialist;
pub use specialist::*;

// Re-export parsers types
pub mod parsers;
pub use parsers::*;
