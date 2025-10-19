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
#[must_use]
pub fn bounded_queue_3<T: Send + 'static>(capacity: usize) -> (TxFuture<T>, RxFuture<T>) {
    smol::channel::bounded(capacity)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T> {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub payload: T,
    pub checksum: Option<String>,
}

impl<T: Serialize> Message<T> {
    /// Create a new message with checksum
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
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub checksum: Option<String>,
}

/// Create a bounded crossfire MPMC channel
#[must_use]
pub fn create_bounded_channel<T: Send + 'static + Unpin>(
    capacity: usize,
) -> (crossfire::MAsyncTx<T>, crossfire::MAsyncRx<T>) {
    crossfire::mpmc::bounded_async(capacity)
}

/// Create an unbounded crossfire channel
#[must_use]
pub fn create_unbounded_channel<T: Send + 'static>() -> (TxFuture<T>, RxFuture<T>) {
    smol::channel::unbounded()
}

/// Send a message on a smol channel (async)
pub async fn send_async<T>(
    sender: &TxFuture<T>,
    msg: T,
) -> Result<(), smol::channel::SendError<T>> {
    sender.send(msg).await
}

/// Receive a message from a smol channel (async)
pub async fn recv_async<T>(receiver: &RxFuture<T>) -> Result<T, smol::channel::RecvError> {
    receiver.recv().await
}

/// Send JSON message on smol channel
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
pub async fn recv_json_message<T: for<'de> Deserialize<'de>>(
    receiver: &RxFuture<JsonMessage>,
) -> Result<T, Box<dyn std::error::Error>> {
    let json_msg = receiver.recv().await?;
    let value: T = serde_json::from_value(json_msg.payload)?;
    Ok(value)
}

/// Broadcast a message to multiple smol senders (high-throughput)
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
    stats.avg_latency = Some(total_time / num_messages as u32);

    stats
}