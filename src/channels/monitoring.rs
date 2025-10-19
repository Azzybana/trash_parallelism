// Standard library imports
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

// External crate imports
use crossfire::mpmc::bounded_async;
use crossfire::{MAsyncRx, MAsyncTx};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json;

/// Channel statistics and monitoring
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChannelStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
    pub avg_latency: Option<Duration>,
}

impl ChannelStats {
    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get statistics as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if serialization to JSON fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// High-performance monitored channel using crossfire channels
pub struct MonitoredChannel<T> {
    tx: MAsyncTx<T>,
    rx: MAsyncRx<T>,
    stats: Arc<Mutex<ChannelStats>>,
}

impl<T: Send + 'static + Unpin> MonitoredChannel<T> {
    /// Create a new monitored channel with default capacity
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a new monitored channel with custom capacity
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = bounded_async(capacity);
        Self {
            tx,
            rx,
            stats: Arc::new(Mutex::new(ChannelStats::default())),
        }
    }

    /// Create a builder for advanced configuration
    #[must_use]
    pub fn builder() -> MonitoredChannelBuilder<T> {
        MonitoredChannelBuilder::new()
    }

    /// Send a message asynchronously with monitoring
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    pub async fn send_async(&self, msg: T) -> Result<(), crossfire::SendError<T>> {
        let start = Instant::now();
        let result = self.tx.send(msg).await;
        let latency = start.elapsed();

        let mut stats = self.stats.lock();
        match &result {
            Ok(()) => {
                stats.messages_sent += 1;
                if let Some(ref mut avg) = stats.avg_latency {
                    *avg = (*avg + latency) / 2;
                } else {
                    stats.avg_latency = Some(latency);
                }
            }
            Err(_) => stats.errors += 1,
        }

        result
    }

    /// Receive a message asynchronously with monitoring
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or empty.
    pub async fn recv_async(&self) -> Result<T, crossfire::RecvError> {
        let start = Instant::now();
        let result = self.rx.recv().await;
        let latency = start.elapsed();

        let mut stats = self.stats.lock();
        match &result {
            Ok(_) => {
                stats.messages_received += 1;
                if let Some(ref mut avg) = stats.avg_latency {
                    *avg = (*avg + latency) / 2;
                } else {
                    stats.avg_latency = Some(latency);
                }
            }
            Err(_) => stats.errors += 1,
        }

        result
    }

    /// Get current statistics (non-blocking snapshot)
    #[must_use]
    pub fn stats(&self) -> ChannelStats {
        self.stats.lock().clone()
    }
}

impl<T: Send + 'static + Unpin> Default for MonitoredChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `MonitoredChannel` with ergonomic configuration
pub struct MonitoredChannelBuilder<T> {
    capacity: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> MonitoredChannelBuilder<T> {
    /// Create a new builder with defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            capacity: 100,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the channel capacity
    #[must_use]
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Build the `MonitoredChannel`
    #[must_use]
    pub fn build(self) -> MonitoredChannel<T>
    where
        T: Send + 'static + Unpin,
    {
        MonitoredChannel::with_capacity(self.capacity)
    }
}

impl<T> Default for MonitoredChannelBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a monitored bounded channel
#[must_use]
pub fn create_monitored_channel<T: Send + 'static + Unpin>(capacity: usize) -> MonitoredChannel<T> {
    MonitoredChannel::with_capacity(capacity)
}
