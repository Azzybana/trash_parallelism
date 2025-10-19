/// Task management utilities for spawning and coordinating async tasks.
///
/// This module provides task spawners and groups for managing concurrent async operations,
/// with built-in cancellation support and error handling using smol runtime.
// Standard library imports
// External crate imports
use parking_lot::Mutex;
use smol;
use smol_cancellation_token::CancellationToken;

/// Async task spawner with error handling
pub struct AsyncTaskSpawner {
    token: CancellationToken,
    handles: Mutex<Vec<smol::Task<()>>>,
}

impl AsyncTaskSpawner {
    /// Create a new task spawner
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Create a builder for more complex configuration
    #[must_use]
    pub fn builder() -> AsyncTaskSpawnerBuilder {
        AsyncTaskSpawnerBuilder::new()
    }

    /// Spawn an async task (non-blocking)
    pub fn spawn<F, Fut>(&self, task: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let token = self.token.clone();
        let handle = smol::spawn(async move {
            if !token.is_cancelled() {
                task().await;
            }
        });

        self.handles.lock().push(handle);
    }

    /// Cancel all tasks
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Wait for all tasks to complete (non-blocking)
    pub async fn wait_all(&self) {
        let handles = std::mem::take(&mut *self.handles.lock());
        for handle in handles {
            let () = handle.await;
        }
    }

    /// Chain method to spawn a task (consumes self)
    #[must_use]
    pub fn with_task<F, Fut>(self, task: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.spawn(task);
        self
    }

    /// Chain method to cancel tasks (consumes self)
    #[must_use]
    pub fn with_cancel(self) -> Self {
        self.cancel();
        self
    }
}

impl Default for AsyncTaskSpawner {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `AsyncTaskSpawner` with ergonomic configuration
pub struct AsyncTaskSpawnerBuilder {
    token: Option<CancellationToken>,
}

impl AsyncTaskSpawnerBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self { token: None }
    }

    /// Set a custom cancellation token
    #[must_use]
    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.token = Some(token);
        self
    }

    /// Build the `AsyncTaskSpawner`
    #[must_use]
    pub fn build(self) -> AsyncTaskSpawner {
        AsyncTaskSpawner {
            token: self.token.unwrap_or_default(),
            handles: Mutex::new(Vec::new()),
        }
    }
}

impl Default for AsyncTaskSpawnerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Async task group for managing multiple related tasks (non-blocking)
pub struct AsyncTaskGroup {
    token: CancellationToken,
    tasks: Mutex<Vec<smol::Task<()>>>,
}

impl AsyncTaskGroup {
    /// Create a new task group
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            tasks: Mutex::new(Vec::new()),
        }
    }

    /// Add a task to the group (non-blocking)
    pub fn add_task<F, Fut>(&self, task: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let token = self.token.clone();
        let task_handle = smol::spawn(async move {
            if !token.is_cancelled() {
                task().await;
            }
        });

        self.tasks.lock().push(task_handle);
    }

    /// Cancel all tasks in the group
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Wait for all tasks to complete (non-blocking)
    pub async fn wait_all(&self) {
        let tasks = std::mem::take(&mut *self.tasks.lock());
        for task in tasks {
            let () = task.await;
        }
    }
}

impl Default for AsyncTaskGroup {
    fn default() -> Self {
        Self::new()
    }
}
