// Standard library imports
use std::sync::Arc;

// External crate imports
use memchr::memchr;
use parking_lot::Mutex;

/// Efficient message parser using memchr
pub struct FastMessageParser {
    delimiter: u8,
}

impl FastMessageParser {
    /// Create a new parser
    #[must_use]
    pub fn new(delimiter: char) -> Self {
        Self {
            delimiter: delimiter as u8,
        }
    }

    /// Parse messages from a buffer (zero-copy slicing)
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
    /// # Errors
    ///
    /// Returns an error if JSON parsing fails for any message slice.
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
pub struct ChannelAggregator<T> {
    inputs: Vec<crate::channels::core::RxFuture<T>>,
    output: crate::channels::core::TxFuture<T>,
}

impl<T: Send + 'static + Clone> ChannelAggregator<T> {
    /// Create a new aggregator
    #[must_use]
    pub fn new(
        inputs: Vec<crate::channels::core::RxFuture<T>>,
        output: crate::channels::core::TxFuture<T>,
    ) -> Self {
        Self { inputs, output }
    }

    /// Start aggregating messages (spawns non-blocking tasks)
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
pub struct BatchingChannel<T> {
    tx: crate::channels::core::TxFuture<Vec<T>>,
    batch_size: usize,
    current_batch: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone + Send + 'static> BatchingChannel<T> {
    /// Create a new batching channel
    #[must_use]
    pub fn new(batch_size: usize, capacity: usize) -> Self {
        let (tx, _) = crate::channels::core::bounded_queue_3(capacity);
        Self {
            tx,
            batch_size,
            current_batch: Arc::new(Mutex::new(Vec::with_capacity(batch_size))),
        }
    }

    /// Send item (batches automatically, async, non-blocking)
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
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
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
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

    /// Get the batch sender
    #[must_use]
    pub fn batch_sender(&self) -> crate::channels::core::TxFuture<Vec<T>> {
        self.tx.clone()
    }
}

/// Channel with message filtering (non-blocking)
pub struct FilteredChannel<T, F> {
    tx: crate::channels::core::TxFuture<T>,
    filter: F,
}

impl<T: Send + 'static, F: Fn(&T) -> bool + Send + Sync + 'static> FilteredChannel<T, F> {
    /// Create a new filtered channel
    pub fn new(sender: crate::channels::core::TxFuture<T>, filter: F) -> Self {
        Self { tx: sender, filter }
    }

    /// Send message if it passes the filter (async, non-blocking)
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    pub async fn send_filtered(&self, msg: T) -> Result<(), smol::channel::SendError<T>> {
        if (self.filter)(&msg) {
            self.tx.send(msg).await
        } else {
            Ok(()) // Silently drop filtered messages
        }
    }
}
