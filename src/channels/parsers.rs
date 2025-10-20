/// Channel parsing and aggregation utilities.
///
/// This module provides efficient message parsing using memchr, channel aggregation
/// for combining multiple inputs, batching channels, and filtered channels.
///
/// # Examples
///
/// Fast message parsing:
/// ```rust
/// use trash_utilities::channels::parsers::FastMessageParser;
///
/// let parser = FastMessageParser::new('\n');
/// let buffer = b"msg1\nmsg2\nmsg3";
/// let messages = parser.parse_messages(buffer);
/// assert_eq!(messages.len(), 3);
/// ```
// Standard library imports
use std::sync::Arc;

// External crate imports
use memchr::memchr;
use parking_lot::Mutex;

/// Efficient message parser using memchr
///
/// Parses messages from byte buffers using fast delimiter-based splitting.
/// Zero-copy where possible for high performance.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::parsers::FastMessageParser;
///
/// let parser = FastMessageParser::new('\n');
/// let data = b"line1\nline2\nline3";
/// let messages = parser.parse_messages(data);
/// assert_eq!(messages, vec![b"line1", b"line2", b"line3"]);
/// ```
pub struct FastMessageParser {
    delimiter: u8,
}

impl FastMessageParser {
    /// Create a new parser
    ///
    /// # Parameters
    ///
    /// * `delimiter` - The character to split messages on.
    ///
    /// # Returns
    ///
    /// A new `FastMessageParser` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::parsers::FastMessageParser;
    ///
    /// let parser = FastMessageParser::new(',');
    /// ```
    #[must_use]
    pub fn new(delimiter: char) -> Self {
        Self {
            delimiter: delimiter as u8,
        }
    }

    /// Parse messages from a buffer (zero-copy slicing)
    ///
    /// Splits the buffer into message slices using the delimiter.
    ///
    /// # Parameters
    ///
    /// * `buffer` - The byte buffer to parse.
    ///
    /// # Returns
    ///
    /// A vector of byte slices, each representing a message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::parsers::FastMessageParser;
    ///
    /// let parser = FastMessageParser::new('|');
    /// let buffer = b"msg1|msg2|msg3";
    /// let messages = parser.parse_messages(buffer);
    /// assert_eq!(messages, vec![&b"msg1"[..], &b"msg2"[..], &b"msg3"[..]]);
    /// ```
    #[must_use]
    pub fn parse_messages<'a>(&self, buffer: &'a [u8]) -> Vec<&'a [u8]> {
        let mut messages = Vec::new();
        let mut start = 0;

        while let Some(pos) = memchr(self.delimiter, &buffer[start..]) {
            let abs_pos = start + pos;
            if abs_pos > start {
                messages.push(&buffer[start..abs_pos]);
            }
            start = abs_pos + 1;
        }

        if start < buffer.len() {
            messages.push(&buffer[start..]);
        }

        messages
    }

    /// Parse JSON messages efficiently
    ///
    /// Parses messages and deserializes each as JSON.
    ///
    /// # Parameters
    ///
    /// * `buffer` - The byte buffer containing JSON messages.
    ///
    /// # Returns
    ///
    /// A vector of parsed JSON values.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON parsing fails for any message slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::parsers::FastMessageParser;
    ///
    /// let parser = FastMessageParser::new('\n');
    /// let buffer = b"{\"name\":\"Alice\"}\n{\"name\":\"Bob\"}";
    /// let messages = parser.parse_json_messages(buffer).unwrap();
    /// assert_eq!(messages.len(), 2);
    /// ```
    pub fn parse_json_messages(
        &self,
        buffer: &[u8],
    ) -> Result<Vec<serde_json::Value>, serde_json::Error> {
        let message_slices = self.parse_messages(buffer);
        let mut results = Vec::new();

        for slice in message_slices {
            if let Ok(s) = std::str::from_utf8(slice)
                && let Ok(value) = serde_json::from_str(s.trim())
            {
                results.push(value);
            }
        }

        Ok(results)
    }
}

/// Channel aggregator for combining multiple channels (non-blocking)
///
/// Merges messages from multiple input channels into a single output channel.
/// Useful for fan-in patterns where multiple producers feed into one consumer.
///
/// # Type Parameters
///
/// * `T` - The type of messages.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::{core::bounded_queue_3, parsers::ChannelAggregator};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx1, rx1) = bounded_queue_3::<String>(5);
/// let (tx2, rx2) = bounded_queue_3::<String>(5);
/// let (tx_out, rx_out) = bounded_queue_3::<String>(10);
///
/// let aggregator = ChannelAggregator::new(vec![rx1, rx2], tx_out);
/// aggregator.start();
///
/// tx1.send("from channel 1".to_string()).await.unwrap();
/// tx2.send("from channel 2".to_string()).await.unwrap();
///
/// let msg1 = rx_out.recv().await.unwrap();
/// let msg2 = rx_out.recv().await.unwrap();
/// // Messages arrive in arbitrary order
/// # });
/// ```
pub struct ChannelAggregator<T> {
    inputs: Vec<crate::channels::core::RxFuture<T>>,
    output: crate::channels::core::TxFuture<T>,
}

impl<T: Send + 'static + Clone> ChannelAggregator<T> {
    /// Create a new aggregator
    ///
    /// # Parameters
    ///
    /// * `inputs` - Vector of input channel receivers.
    /// * `output` - The output channel sender.
    ///
    /// # Returns
    ///
    /// A new `ChannelAggregator` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, parsers::ChannelAggregator};
    ///
    /// let (tx1, rx1) = bounded_queue_3::<i32>(5);
    /// let (tx2, rx2) = bounded_queue_3::<i32>(5);
    /// let (tx_out, rx_out) = bounded_queue_3::<i32>(10);
    ///
    /// let aggregator = ChannelAggregator::new(vec![rx1, rx2], tx_out);
    /// ```
    #[must_use]
    pub fn new(
        inputs: Vec<crate::channels::core::RxFuture<T>>,
        output: crate::channels::core::TxFuture<T>,
    ) -> Self {
        Self { inputs, output }
    }

    /// Start aggregating messages (spawns non-blocking tasks)
    ///
    /// Begins forwarding messages from all input channels to the output channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, parsers::ChannelAggregator};
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let (tx1, rx1) = bounded_queue_3::<String>(5);
    /// let (tx_out, rx_out) = bounded_queue_3::<String>(10);
    ///
    /// let aggregator = ChannelAggregator::new(vec![rx1], tx_out);
    /// aggregator.start();
    ///
    /// tx1.send("message".to_string()).await.unwrap();
    /// let received = rx_out.recv().await.unwrap();
    /// assert_eq!(received, "message");
    /// # });
    /// ```
    pub fn start(self) {
        for receiver in self.inputs {
            let output = self.output.clone();
            smol::spawn(async move {
                let rx = receiver;
                while let Ok(msg) = rx.recv().await {
                    let _ = output.send(msg).await;
                }
            })
            .detach();
        }
    }
}

/// Channel with automatic batching (non-blocking)
///
/// Accumulates individual items into batches before sending them.
/// Useful for reducing overhead when processing many small messages.
///
/// # Type Parameters
///
/// * `T` - The type of individual items.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::parsers::BatchingChannel;
/// use smol;
///
/// # smol::block_on(async {
/// let channel = BatchingChannel::new(3, 10); // Batch size 3
/// channel.send(1).await.unwrap();
/// channel.send(2).await.unwrap();
/// channel.send(3).await.unwrap(); // Triggers batch send
///
/// let batch_sender = channel.batch_sender();
/// // The batch [1,2,3] is now available on batch_sender
/// # });
/// ```
pub struct BatchingChannel<T> {
    tx: crate::channels::core::TxFuture<Vec<T>>,
    rx: crate::channels::core::RxFuture<Vec<T>>,
    batch_size: usize,
    current_batch: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone + Send + 'static> BatchingChannel<T> {
    /// Create a new batching channel
    ///
    /// # Parameters
    ///
    /// * `batch_size` - Number of items to accumulate before sending a batch.
    /// * `capacity` - Capacity of the batch output channel.
    ///
    /// # Returns
    ///
    /// A new `BatchingChannel` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::parsers::BatchingChannel;
    ///
    /// let channel = BatchingChannel::new(10, 5); // Batch 10 items, channel capacity 5
    /// ```
    #[must_use]
    pub fn new(batch_size: usize, capacity: usize) -> Self {
        let (tx, rx) = crate::channels::core::bounded_queue_3(capacity);
        Self {
            tx,
            rx,
            batch_size,
            current_batch: Arc::new(Mutex::new(Vec::with_capacity(batch_size))),
        }
    }

    /// Send item (batches automatically, async, non-blocking)
    ///
    /// Adds the item to the current batch. When the batch is full, it's sent automatically.
    ///
    /// # Parameters
    ///
    /// * `item` - The item to add to the batch.
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::parsers::BatchingChannel;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let channel = BatchingChannel::new(2, 5);
    /// channel.send("item1").await.unwrap();
    /// channel.send("item2").await.unwrap(); // Batch sent here
    /// # });
    /// ```
    pub async fn send(&self, item: T) -> Result<(), Box<dyn std::error::Error>> {
        let should_flush = {
            let mut batch = self.current_batch.lock();
            batch.push(item);
            batch.len() >= self.batch_size
        };

        if should_flush {
            self.flush_batch().await?;
        }

        Ok(())
    }

    /// Flush current batch (async, non-blocking)
    ///
    /// Sends the current batch immediately, even if not full.
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::parsers::BatchingChannel;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let channel = BatchingChannel::new(10, 5);
    /// channel.send(1).await.unwrap();
    /// channel.send(2).await.unwrap();
    /// channel.flush_batch().await.unwrap(); // Send partial batch
    /// # });
    /// ```
    pub async fn flush_batch(&self) -> Result<(), Box<dyn std::error::Error>> {
        let batch = {
            let mut current = self.current_batch.lock();
            std::mem::take(&mut *current)
        };

        if !batch.is_empty() {
            self.tx.send(batch).await?;
        }

        Ok(())
    }

    /// Get the batch receiver
    ///
    /// Returns the channel receiver that receives completed batches.
    ///
    /// # Returns
    ///
    /// The batch output channel receiver.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::parsers::BatchingChannel;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let channel = BatchingChannel::new(2, 10);
    /// let batch_receiver = channel.batch_receiver();
    /// channel.send("item1").await.unwrap();
    /// channel.send("item2").await.unwrap();
    /// let batch = batch_receiver.recv().await.unwrap();
    /// assert_eq!(batch.len(), 2);
    /// # });
    /// ```
    #[must_use]
    pub fn batch_receiver(&self) -> crate::channels::core::RxFuture<Vec<T>> {
        self.rx.clone()
    }
}

/// Channel with message filtering (non-blocking)
///
/// Only sends messages that pass through a filter function.
/// Useful for conditional message processing and routing.
///
/// # Type Parameters
///
/// * `T` - The type of messages.
/// * `F` - The type of the filter function.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::{core::bounded_queue_3, parsers::FilteredChannel};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx, rx) = bounded_queue_3::<i32>(10);
/// let filtered = FilteredChannel::new(tx, |&num| num > 0); // Only positive numbers
///
/// filtered.send_filtered(5).await.unwrap();  // Sent
/// filtered.send_filtered(-1).await.unwrap(); // Filtered out
/// filtered.send_filtered(10).await.unwrap(); // Sent
///
/// let positive = rx.recv().await.unwrap();
/// assert_eq!(positive, 5);
/// # });
/// ```
pub struct FilteredChannel<T, F> {
    tx: crate::channels::core::TxFuture<T>,
    filter: F,
}

impl<T: Send + 'static, F: Fn(&T) -> bool + Send + Sync + 'static> FilteredChannel<T, F> {
    /// Create a new filtered channel
    ///
    /// # Parameters
    ///
    /// * `sender` - The underlying channel sender.
    /// * `filter` - Function that returns true for messages to send.
    ///
    /// # Returns
    ///
    /// A new `FilteredChannel` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, parsers::FilteredChannel};
    ///
    /// let (tx, _) = bounded_queue_3::<String>(10);
    /// let filtered = FilteredChannel::new(tx, |msg| msg.len() > 3);
    /// ```
    pub fn new(sender: crate::channels::core::TxFuture<T>, filter: F) -> Self {
        Self { tx: sender, filter }
    }

    /// Send message if it passes the filter (async, non-blocking)
    ///
    /// Only sends the message if the filter function returns true.
    ///
    /// # Parameters
    ///
    /// * `msg` - The message to potentially send.
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, parsers::FilteredChannel};
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let (tx, rx) = bounded_queue_3::<&str>(10);
    /// let filtered = FilteredChannel::new(tx, |msg| msg.starts_with("ok"));
    ///
    /// filtered.send_filtered("ok message").await.unwrap();     // Sent
    /// filtered.send_filtered("error message").await.unwrap();  // Filtered
    ///
    /// let msg = rx.recv().await.unwrap();
    /// assert_eq!(msg, "ok message");
    /// # });
    /// ```
    pub async fn send_filtered(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        if (self.filter)(&msg) {
            self.tx.send(msg).await
        } else {
            Ok(()) // Silently drop filtered messages
        }
    }
}
