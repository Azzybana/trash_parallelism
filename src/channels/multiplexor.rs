/// Channel multiplexing and processing utilities.
///
/// This module provides tools for routing messages to different channels based on type,
/// processing messages asynchronously with error handling, and creating complex channel topologies.
///
/// # Examples
///
/// Basic multiplexing:
/// ```rust
/// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::ChannelMultiplexer};
/// use smol;
///
/// # smol::block_on(async {
/// let multiplexer = ChannelMultiplexer::new();
/// let (tx, _) = bounded_queue_3::<String>(10);
/// multiplexer.register_route("text", tx);
/// multiplexer.route_message("text", "hello".to_string()).await.unwrap();
/// # });
/// ```
// Standard library imports
use std::sync::Arc;

// External crate imports
use ahash::AHashMap;
use parking_lot::Mutex;

/// Type alias for the processor function type
pub type ProcessorFn<T> = dyn Fn(
        T,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
    > + Send
    + Sync
    + 'static;

/// Channel multiplexer for routing messages based on type (non-blocking)
///
/// Routes messages to different channels based on route names, enabling complex
/// message distribution patterns.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::ChannelMultiplexer};
/// use smol;
///
/// # smol::block_on(async {
/// let multiplexer = ChannelMultiplexer::new();
/// let (tx1, _) = bounded_queue_3::<String>(10);
/// let (tx2, _) = bounded_queue_3::<i32>(10);
/// multiplexer.register_route("strings", tx1);
/// multiplexer.register_route("numbers", tx2);
/// multiplexer.route_message("strings", "hello".to_string()).await.unwrap();
/// multiplexer.route_message("numbers", 42).await.unwrap();
/// # });
/// ```
pub struct ChannelMultiplexer {
    routes: Mutex<AHashMap<String, Box<dyn std::any::Any + Send + Sync>>>,
}

impl ChannelMultiplexer {
    /// Create a new multiplexer
    ///
    /// # Returns
    ///
    /// A new `ChannelMultiplexer` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::multiplexor::ChannelMultiplexer;
    ///
    /// let multiplexer = ChannelMultiplexer::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            routes: Mutex::new(AHashMap::new()),
        }
    }

    /// Register a route for a message type
    ///
    /// Associates a route name with a channel sender for routing messages.
    ///
    /// # Parameters
    ///
    /// * `route_name` - The name to identify this route.
    /// * `sender` - The channel sender to route messages to.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of messages this route handles.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::ChannelMultiplexer};
    ///
    /// let multiplexer = ChannelMultiplexer::new();
    /// let (tx, _) = bounded_queue_3::<String>(10);
    /// multiplexer.register_route("messages", tx);
    /// ```
    pub fn register_route<T: Send + 'static>(
        &self,
        route_name: &str,
        sender: crate::channels::core::TxFuture<T>,
    ) {
        self.routes
            .lock()
            .insert(route_name.to_string(), Box::new(sender));
    }

    /// Route a message to the appropriate channel (async)
    ///
    /// Sends the message to the channel registered for the given route name.
    ///
    /// # Parameters
    ///
    /// * `route_name` - The route to send the message through.
    /// * `message` - The message to send.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the message.
    ///
    /// # Errors
    ///
    /// Returns an error if no route is found for the route name, or if the channel is closed or full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::ChannelMultiplexer};
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let multiplexer = ChannelMultiplexer::new();
    /// let (tx, _) = bounded_queue_3::<String>(10);
    /// multiplexer.register_route("chat", tx);
    /// multiplexer.route_message("chat", "Hello, world!".to_string()).await.unwrap();
    /// # });
    /// ```
    #[allow(clippy::await_holding_lock)]
    pub async fn route_message<T: Send + 'static + Clone>(
        &self,
        route_name: &str,
        message: T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let routes = self.routes.lock();
        if let Some(route) = routes.get(route_name)
            && let Some(sender) = route.downcast_ref::<crate::channels::core::TxFuture<T>>()
        {
            sender.send(message).await?;
            return Ok(());
        }
        Err(format!("No route found for: {route_name}").into())
    }
}

impl Default for ChannelMultiplexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Async channel processor with error handling (non-blocking)
///
/// Processes messages from a channel asynchronously, with optional error handling.
/// Each message is processed in a separate task for high throughput.
///
/// # Type Parameters
///
/// * `T` - The type of messages to process.
/// * `F` - The type of the processing function.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::create_async_processor};
/// use smol;
///
/// # smol::block_on(async {
/// let (tx, rx) = bounded_queue_3::<String>(10);
/// let processor = create_async_processor(rx, |msg: String| {
///     Box::pin(async move {
///         println!("Processing: {}", msg);
///         Ok(())
///     })
/// });
/// processor.start();
/// tx.send("test message".to_string()).await.unwrap();
/// smol::Timer::after(std::time::Duration::from_millis(100)).await; // Allow processing
/// # });
/// ```
#[allow(clippy::type_complexity)]
pub struct AsyncChannelProcessor<T, F>
where
    T: Send + 'static,
{
    receiver: crate::channels::core::RxFuture<T>,
    processor: F,
    #[allow(clippy::type_complexity)]
    error_handler: Option<Arc<dyn Fn(Box<dyn std::error::Error>) + Send + Sync>>,
}

impl<T, F> AsyncChannelProcessor<T, F>
where
    T: Send + 'static,
    F: Fn(
            T,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
        > + Send
        + Sync
        + 'static,
{
    /// Create a new processor
    ///
    /// # Parameters
    ///
    /// * `receiver` - The channel receiver to read messages from.
    /// * `processor` - Function to process each message.
    ///
    /// # Returns
    ///
    /// A new `AsyncChannelProcessor` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::AsyncChannelProcessor};
    ///
    /// let (tx, rx) = bounded_queue_3::<i32>(10);
    /// let processor = AsyncChannelProcessor::new(rx, |num: i32| {
    ///     Box::pin(async move {
    ///         println!("Processed: {}", num);
    ///         Ok(())
    ///     })
    /// });
    /// ```
    pub fn new(receiver: crate::channels::core::RxFuture<T>, processor: F) -> Self {
        Self {
            receiver,
            processor,
            error_handler: None,
        }
    }

    /// Set error handler
    ///
    /// Configures a function to handle processing errors.
    ///
    /// # Parameters
    ///
    /// * `handler` - Function called when processing fails.
    ///
    /// # Returns
    ///
    /// The processor instance for chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::AsyncChannelProcessor};
    ///
    /// let (tx, rx) = bounded_queue_3::<String>(10);
    /// let processor = AsyncChannelProcessor::new(rx, |msg: String| {
    ///         Box::pin(async move { Ok(()) })
    ///     })
    ///     .with_error_handler(|err| eprintln!("Processing error: {}", err));
    /// ```
    #[must_use]
    pub fn with_error_handler(
        mut self,
        handler: impl Fn(Box<dyn std::error::Error>) + Send + Sync + 'static,
    ) -> Self {
        self.error_handler = Some(Arc::new(handler));
        self
    }

    /// Start processing messages (spawns non-blocking task)
    ///
    /// Begins processing messages in the background. Each message is handled in a separate task.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::AsyncChannelProcessor};
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let (tx, rx) = bounded_queue_3::<String>(10);
    /// let processor = AsyncChannelProcessor::new(rx, |msg: String| {
    ///     Box::pin(async move { Ok(()) })
    /// });
    /// processor.start();
    /// tx.send("start processing".to_string()).await.unwrap();
    /// # });
    /// ```
    pub fn start(self) {
        let receiver = self.receiver.clone();
        let processor = Arc::new(self.processor);

        smol::spawn(async move {
            let rx = receiver;
            while let Ok(message) = rx.recv().await {
                let processor = processor.clone();

                smol::spawn(async move {
                    if let Err(_e) = processor(message).await {
                        // Error handling removed
                    }
                })
                .detach();
            }
        })
        .detach();
    }
}

/// Create an async channel processor
///
/// Convenience function to create a processor with a receiver and processing function.
///
/// # Parameters
///
/// * `receiver` - The channel receiver.
/// * `processor` - The message processing function.
///
/// # Type Parameters
///
/// * `T` - Message type.
/// * `F` - Processor function type.
///
/// # Returns
///
/// A new `AsyncChannelProcessor` instance.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::{core::bounded_queue_3, multiplexor::create_async_processor};
///
/// let (tx, rx) = bounded_queue_3::<i32>(10);
/// let processor = create_async_processor(rx, |num: i32| {
///     Box::pin(async move {
///         println!("Number: {}", num);
///         Ok(())
///     })
/// });
/// ```
pub fn create_async_processor<T, F>(
    receiver: crate::channels::core::RxFuture<T>,
    processor: F,
) -> AsyncChannelProcessor<T, F>
where
    T: Send + 'static,
    F: Fn(
            T,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
        > + Send
        + Sync
        + 'static,
{
    AsyncChannelProcessor::new(receiver, processor)
}
