// Standard library imports
use std::sync::Arc;

// External crate imports
use ahash::AHashMap;
use parking_lot::Mutex;

/// Channel multiplexer for routing messages based on type (non-blocking)
pub struct ChannelMultiplexer {
    routes: Mutex<AHashMap<String, Box<dyn std::any::Any + Send + Sync>>>,
}

impl ChannelMultiplexer {
    /// Create a new multiplexer
    #[must_use]
    pub fn new() -> Self {
        Self {
            routes: Mutex::new(AHashMap::new()),
        }
    }

    /// Register a route for a message type
    pub fn register_route<T: Send + 'static>(&self, route_name: &str, sender: crate::channels::core::TxFuture<T>) {
        self.routes
            .lock()
            .insert(route_name.to_string(), Box::new(sender));
    }

    /// Route a message to the appropriate channel (async)
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
pub struct AsyncChannelProcessor<T, F>
where
    T: Send + 'static,
{
    receiver: crate::channels::core::RxFuture<T>,
    processor: F,
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
    pub fn new(receiver: crate::channels::core::RxFuture<T>, processor: F) -> Self {
        Self {
            receiver,
            processor,
            error_handler: None,
        }
    }

    /// Set error handler
    pub fn with_error_handler(
        mut self,
        handler: impl Fn(Box<dyn std::error::Error>) + Send + Sync + 'static,
    ) -> Self {
        self.error_handler = Some(Arc::new(handler));
        self
    }

    /// Start processing messages (spawns non-blocking task)
    pub fn start(self) {
        let receiver = self.receiver.clone();
        let processor = Arc::new(self.processor);

        smol::spawn(async move {
            let rx = receiver;
            loop {
                match rx.recv().await {
                    Ok(message) => {
                        let processor = processor.clone();

                        smol::spawn(async move {
                            if let Err(_e) = processor(message).await {
                                // Error handling removed
                            }
                        })
                        .detach();
                    }
                    Err(_) => break,
                }
            }
        })
        .detach();
    }
}

/// Create an async channel processor
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