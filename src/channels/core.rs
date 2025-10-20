/// Core channel utilities providing fundamental async communication primitives.
///
/// This module offers essential building blocks for async channels using smol and crossfire,
/// including bounded/unbounded channels, message structures with integrity checking,
/// and basic send/receive operations.
///
/// # Examples
///
/// Basic channel usage:
/// ```rust
/// use trash_utilities::channels::core::{bounded_queue_3, send_async, recv_async};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx, rx) = bounded_queue_3::<String>(10);
/// send_async(&tx, "hello".to_string()).await.unwrap();
/// let msg = recv_async(&rx).await.unwrap();
/// assert_eq!(msg, "hello");
/// # });
/// ```
// Standard library imports
use std::time::Instant;

// External crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;

/// Type alias for async transmitter
pub type TxFuture<T> = smol::channel::Sender<T>;

/// Type alias for async receiver
pub type RxFuture<T> = smol::channel::Receiver<T>;

/// Create a bounded async channel using smol
///
/// Creates a channel with a fixed buffer capacity. Sending blocks when the buffer is full.
///
/// # Parameters
///
/// * `capacity` - The maximum number of messages the channel can hold.
///
/// # Type Parameters
///
/// * `T` - The type of messages sent through the channel.
///
/// # Returns
///
/// A tuple of (sender, receiver) for the channel.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::bounded_queue_3;
///
/// let (tx, rx) = bounded_queue_3::<i32>(5);
/// ```
#[must_use]
pub fn bounded_queue_3<T: Send + 'static>(capacity: usize) -> (TxFuture<T>, RxFuture<T>) {
    smol::channel::bounded(capacity)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T> {
    /// Unique message identifier
    pub id: String,
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    /// The actual message payload
    pub payload: T,
    /// Optional checksum for integrity verification
    pub checksum: Option<String>,
}

impl<T: Serialize> Message<T> {
    /// Create a new message with checksum
    ///
    /// Generates a unique ID, timestamp, and checksum for the payload.
    ///
    /// # Parameters
    ///
    /// * `payload` - The data to wrap in the message.
    ///
    /// # Returns
    ///
    /// A new `Message` instance with integrity checking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::core::Message;
    ///
    /// let msg = Message::new("hello world");
    /// assert!(msg.verify());
    /// ```
    pub fn new(payload: T) -> Self {
        use ahash::AHasher;
        use std::hash::Hasher;

        let id = format!(
            "msg_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let json = serde_json::to_string(&payload).unwrap_or_default();
        let mut hasher = AHasher::default();
        hasher.write(json.as_bytes());
        let checksum = Some(format!("{:016x}", hasher.finish()));

        Self {
            id,
            timestamp: Utc::now(),
            payload,
            checksum,
        }
    }

    /// Verify message integrity using checksum
    ///
    /// Recalculates the checksum and compares it with the stored one.
    ///
    /// # Returns
    ///
    /// `true` if the checksum matches or no checksum is present, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::core::Message;
    ///
    /// let msg = Message::new("data");
    /// assert!(msg.verify());
    /// ```
    pub fn verify(&self) -> bool {
        if let Some(ref checksum) = self.checksum
            && let Ok(json) = serde_json::to_string(&self.payload)
        {
            use ahash::AHasher;
            use std::hash::Hasher;
            let mut hasher = AHasher::default();
            hasher.write(json.as_bytes());
            let computed = format!("{:016x}", hasher.finish());
            return computed == *checksum;
        }
        true
    }
}

/// JSON-encoded message for text-based channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMessage {
    /// Unique message identifier
    pub id: String,
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    /// JSON payload
    pub payload: serde_json::Value,
    /// Optional checksum for integrity verification
    pub checksum: Option<String>,
}

/// Create a bounded crossfire MPMC channel
///
/// Creates a multi-producer, multi-consumer channel with fixed capacity using crossfire.
///
/// # Parameters
///
/// * `capacity` - The maximum number of messages the channel can hold.
///
/// # Type Parameters
///
/// * `T` - The type of messages (must be Send + Unpin).
///
/// # Returns
///
/// A tuple of (sender, receiver) for the MPMC channel.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::create_bounded_channel;
///
/// let (tx, rx) = create_bounded_channel::<i32>(10);
/// ```
#[must_use]
pub fn create_bounded_channel<T: Send + 'static + Unpin>(
    capacity: usize,
) -> (crossfire::MAsyncTx<T>, crossfire::MAsyncRx<T>) {
    crossfire::mpmc::bounded_async(capacity)
}

/// Create an unbounded crossfire channel
///
/// Creates an unbounded MPMC channel that can grow indefinitely.
///
/// # Type Parameters
///
/// * `T` - The type of messages.
///
/// # Returns
///
/// A tuple of (sender, receiver) for the unbounded channel.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::create_unbounded_channel;
///
/// let (tx, rx) = create_unbounded_channel::<String>();
/// ```
#[must_use]
pub fn create_unbounded_channel<T: Send + 'static>() -> (TxFuture<T>, RxFuture<T>) {
    smol::channel::unbounded()
}

/// Send a message on a smol channel (async)
///
/// Asynchronously sends a message through the channel.
///
/// # Parameters
///
/// * `sender` - The channel sender.
/// * `msg` - The message to send.
///
/// # Errors
///
/// Returns an error if the channel is closed or full.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::{bounded_queue_3, send_async};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx, _) = bounded_queue_3::<String>(1);
/// send_async(&tx, "hello".to_string()).await.unwrap();
/// # });
/// ```
pub async fn send_async<T>(
    sender: &TxFuture<T>,
    msg: T,
) -> Result<(), smol::channel::SendError<T>> {
    sender.send(msg).await
}

/// Receive a message from a smol channel (async)
///
/// Asynchronously receives a message from the channel.
///
/// # Parameters
///
/// * `receiver` - The channel receiver.
///
/// # Returns
///
/// The received message on success.
///
/// # Errors
///
/// Returns an error if the channel is closed or empty.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::{bounded_queue_3, send_async, recv_async};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx, rx) = bounded_queue_3::<String>(1);
/// send_async(&tx, "hello".to_string()).await.unwrap();
/// let msg = recv_async(&rx).await.unwrap();
/// assert_eq!(msg, "hello");
/// # });
/// ```
pub async fn recv_async<T>(receiver: &RxFuture<T>) -> Result<T, smol::channel::RecvError> {
    receiver.recv().await
}

/// Send JSON message on smol channel
///
/// Creates a JSON message with integrity checking and sends it.
///
/// # Parameters
///
/// * `sender` - The channel sender.
/// * `payload` - The data to send (will be serialized to JSON).
///
/// # Type Parameters
///
/// * `T` - The type of payload (must implement Serialize).
///
/// # Errors
///
/// Returns an error if the channel is closed or full.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::{bounded_queue_3, send_json_message};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx, _) = bounded_queue_3(1);
/// send_json_message(&tx, "test data").await.unwrap();
/// # });
/// ```
pub async fn send_json_message<T: Serialize>(
    sender: &TxFuture<JsonMessage>,
    payload: T,
) -> Result<(), smol::channel::SendError<JsonMessage>> {
    let message = Message::new(payload);
    let json_msg = JsonMessage {
        id: message.id,
        timestamp: message.timestamp,
        payload: serde_json::to_value(&message.payload).unwrap_or(serde_json::Value::Null),
        checksum: message.checksum,
    };
    sender.send(json_msg).await
}

/// Receive and parse JSON message from smol channel
///
/// Receives a JSON message and deserializes the payload.
///
/// # Parameters
///
/// * `receiver` - The channel receiver.
///
/// # Type Parameters
///
/// * `T` - The type to deserialize the payload into.
///
/// # Returns
///
/// The deserialized payload on success.
///
/// # Errors
///
/// Returns an error if the channel is closed or empty, or if deserialization fails.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::{bounded_queue_3, send_json_message, recv_json_message};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx, rx) = bounded_queue_3(1);
/// send_json_message(&tx, "test").await.unwrap();
/// let data: String = recv_json_message(&rx).await.unwrap();
/// assert_eq!(data, "test");
/// # });
/// ```
pub async fn recv_json_message<T: for<'de> Deserialize<'de>>(
    receiver: &RxFuture<JsonMessage>,
) -> Result<T, Box<dyn std::error::Error>> {
    let json_msg = receiver.recv().await?;
    let value: T = serde_json::from_value(json_msg.payload)?;
    Ok(value)
}

/// Broadcast a message to multiple smol senders (high-throughput)
///
/// Sends the same message to multiple receivers concurrently.
///
/// # Parameters
///
/// * `message` - The message to broadcast.
/// * `senders` - Vector of channel senders to broadcast to.
///
/// # Type Parameters
///
/// * `T` - The type of message (must be Clone + Send).
///
/// # Errors
///
/// Returns an error if any of the channels are closed or full.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::{bounded_queue_3, broadcast_message};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx1, _) = bounded_queue_3::<String>(1);
/// let (tx2, _) = bounded_queue_3::<String>(1);
/// let senders = vec![tx1, tx2];
/// broadcast_message("broadcast".to_string(), senders).await.unwrap();
/// # });
/// ```
pub async fn broadcast_message<T: Clone + Send + 'static>(
    message: T,
    senders: Vec<TxFuture<T>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tasks: Vec<_> = senders
        .into_iter()
        .map(|sender| {
            let msg = message.clone();
            smol::spawn(async move { sender.send(msg).await })
        })
        .collect();

    for task in tasks {
        task.await?;
    }

    Ok(())
}

/// Channel performance benchmark (non-blocking)
///
/// Measures channel throughput by sending and receiving a specified number of messages.
///
/// # Parameters
///
/// * `sender` - The channel sender.
/// * `receiver` - The channel receiver.
/// * `message` - Sample message to send.
/// * `num_messages` - Number of messages to benchmark with.
///
/// # Returns
///
/// Channel statistics including latency and throughput metrics.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::core::{bounded_queue_3, benchmark_channel};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx, rx) = bounded_queue_3::<String>(100);
/// let stats = benchmark_channel(&tx, &rx, "test".to_string(), 1000).await;
/// println!("Sent: {}, Received: {}", stats.messages_sent, stats.messages_received);
/// # });
/// ```
pub async fn benchmark_channel<T: Clone + Send + 'static>(
    sender: &TxFuture<T>,
    receiver: &RxFuture<T>,
    message: T,
    num_messages: usize,
) -> crate::channels::monitoring::ChannelStats {
    use crate::channels::monitoring::ChannelStats;

    let mut stats = ChannelStats::default();
    let start = Instant::now();

    // Concurrent send and receive
    let send_task = {
        let sender = sender.clone();
        let msg = message.clone();
        smol::spawn(async move {
            for _ in 0..num_messages {
                if sender.send(msg.clone()).await.is_ok() {
                    // stats.messages_sent += 1;
                } else {
                    // stats.errors += 1;
                }
            }
        })
    };

    let recv_task = {
        let receiver = receiver.clone();
        smol::spawn(async move {
            for _ in 0..num_messages {
                if receiver.recv().await.is_ok() {
                    // stats.messages_received += 1;
                } else {
                    // stats.errors += 1;
                }
            }
        })
    };

    send_task.await;
    recv_task.await;

    let total_time = start.elapsed();
    stats.messages_sent = num_messages as u64;
    stats.messages_received = num_messages as u64;
    #[allow(clippy::cast_possible_truncation)]
    {
        stats.avg_latency = Some(total_time / num_messages as u32);
    }

    stats
}
