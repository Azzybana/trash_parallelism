/// Task management utilities for spawning and coordinating async tasks.
///
/// This module provides task spawners and groups for managing concurrent async operations,
/// with built-in cancellation support and error handling using smol runtime.
///
/// # Examples
///
/// Basic task spawning:
/// ```rust
/// use trash_utilities::async::tasks::AsyncTaskSpawner;
/// use smol;
///
/// # smol::block_on(async {
/// let spawner = AsyncTaskSpawner::new();
/// spawner.spawn(|| async {
///     // Your async task here
///     println!("Task executed!");
/// });
/// spawner.wait_all().await;
/// # });
/// ```
// Standard library imports
// External crate imports
use parking_lot::Mutex;
use smol;
use smol_cancellation_token::CancellationToken;

/// Async task spawner with error handling.
///
/// `AsyncTaskSpawner` allows spawning multiple asynchronous tasks concurrently,
/// managing their lifecycles with built-in cancellation support. Tasks are spawned
/// using the smol runtime and can be cancelled collectively.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::tasks::AsyncTaskSpawner;
/// use smol;
///
/// # smol::block_on(async {
/// let spawner = AsyncTaskSpawner::new();
/// spawner.spawn(|| async {
///     // Simulate work
///     smol::Timer::after(std::time::Duration::from_millis(100)).await;
///     println!("Task 1 done");
/// });
/// spawner.spawn(|| async {
///     println!("Task 2 done");
/// });
/// spawner.wait_all().await;
/// # });
/// ```
pub struct AsyncTaskSpawner {
    token: CancellationToken,
    handles: Mutex<Vec<smol::Task<()>>>,
}

impl AsyncTaskSpawner {
    /// Create a new task spawner with a default cancellation token.
    ///
    /// # Returns
    ///
    /// A new `AsyncTaskSpawner` instance ready to spawn tasks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskSpawner;
    ///
    /// let spawner = AsyncTaskSpawner::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Create a builder for more complex configuration.
    ///
    /// Use the builder to customize the cancellation token or other settings.
    ///
    /// # Returns
    ///
    /// An `AsyncTaskSpawnerBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskSpawner;
    /// use smol_cancellation_token::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// let spawner = AsyncTaskSpawner::builder()
    ///     .with_cancellation_token(token)
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> AsyncTaskSpawnerBuilder {
        AsyncTaskSpawnerBuilder::new()
    }

    /// Spawn an async task (non-blocking).
    ///
    /// The task will be executed asynchronously using the smol runtime.
    /// If the spawner's cancellation token is cancelled before the task starts,
    /// the task will not execute.
    ///
    /// # Parameters
    ///
    /// * `task` - A closure that returns a future representing the async task.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The type of the task closure.
    /// * `Fut` - The type of the future returned by the task.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskSpawner;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let spawner = AsyncTaskSpawner::new();
    /// spawner.spawn(|| async {
    ///     println!("Hello from async task!");
    /// });
    /// spawner.wait_all().await;
    /// # });
    /// ```
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

    /// Cancel all tasks managed by this spawner.
    ///
    /// This sets the cancellation token, which will prevent new tasks from starting
    /// and may interrupt running tasks if they check for cancellation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskSpawner;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let spawner = AsyncTaskSpawner::new();
    /// spawner.spawn(|| async {
    ///     // Task logic here
    /// });
    /// spawner.cancel(); // Cancels the task
    /// # });
    /// ```
    /// Wait for all tasks to complete (non-blocking)
    pub async fn wait_all(&self) {
        let handles = std::mem::take(&mut *self.handles.lock());
        for handle in handles {
            let () = handle.await;
        }
    }

    /// Wait for all tasks to complete (non-blocking).
    ///
    /// This method awaits all spawned tasks to finish. It consumes the task handles,
    /// so it can only be called once.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskSpawner;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let spawner = AsyncTaskSpawner::new();
    /// spawner.spawn(|| async {
    ///     smol::Timer::after(std::time::Duration::from_millis(10)).await;
    /// });
    /// spawner.wait_all().await; // Waits for the task to complete
    /// # });
    /// ```
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

/// Builder for `AsyncTaskSpawner` with ergonomic configuration.
///
/// Allows customizing the cancellation token and other settings before building
/// the spawner.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::tasks::AsyncTaskSpawner;
/// use smol_cancellation_token::CancellationToken;
///
/// let token = CancellationToken::new();
/// let spawner = AsyncTaskSpawner::builder()
///     .with_cancellation_token(token)
///     .build();
/// ```
pub struct AsyncTaskSpawnerBuilder {
    token: Option<CancellationToken>,
}

impl AsyncTaskSpawnerBuilder {
    /// Create a new builder with default settings.
    ///
    /// # Returns
    ///
    /// An `AsyncTaskSpawnerBuilder` instance with no custom token set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskSpawnerBuilder;
    ///
    /// let builder = AsyncTaskSpawnerBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { token: None }
    }

    /// Set a custom cancellation token.
    ///
    /// If not set, a default token will be created.
    ///
    /// # Parameters
    ///
    /// * `token` - The cancellation token to use for the spawner.
    ///
    /// # Returns
    ///
    /// The builder instance for chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskSpawnerBuilder;
    /// use smol_cancellation_token::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// let builder = AsyncTaskSpawnerBuilder::new()
    ///     .with_cancellation_token(token);
    /// ```
    #[must_use]
    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.token = Some(token);
        self
    }

    /// Build the `AsyncTaskSpawner` with the configured settings.
    ///
    /// # Returns
    ///
    /// A new `AsyncTaskSpawner` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskSpawnerBuilder;
    ///
    /// let spawner = AsyncTaskSpawnerBuilder::new().build();
    /// ```
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

/// Async task group for managing multiple related tasks (non-blocking).
///
/// `AsyncTaskGroup` provides a way to group related tasks together,
/// allowing collective cancellation and waiting. Similar to `AsyncTaskSpawner`,
/// but designed for tasks that are logically grouped.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::tasks::AsyncTaskGroup;
/// use smol;
///
/// # smol::block_on(async {
/// let group = AsyncTaskGroup::new();
/// group.add_task(|| async {
///     println!("Task in group");
/// });
/// group.wait_all().await;
/// # });
/// ```
pub struct AsyncTaskGroup {
    token: CancellationToken,
    tasks: Mutex<Vec<smol::Task<()>>>,
}

impl AsyncTaskGroup {
    /// Create a new task group with a default cancellation token.
    ///
    /// # Returns
    ///
    /// A new `AsyncTaskGroup` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskGroup;
    ///
    /// let group = AsyncTaskGroup::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            tasks: Mutex::new(Vec::new()),
        }
    }

    /// Add a task to the group (non-blocking).
    ///
    /// The task will be executed asynchronously. If the group's cancellation
    /// token is cancelled, the task may not start.
    ///
    /// # Parameters
    ///
    /// * `task` - A closure that returns a future for the task.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The type of the task closure.
    /// * `Fut` - The type of the future.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskGroup;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let group = AsyncTaskGroup::new();
    /// group.add_task(|| async {
    ///     println!("Group task executed");
    /// });
    /// group.wait_all().await;
    /// # });
    /// ```
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

    /// Cancel all tasks in the group.
    ///
    /// Sets the cancellation token, preventing new tasks from starting
    /// and potentially interrupting running ones.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskGroup;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let group = AsyncTaskGroup::new();
    /// group.add_task(|| async {
    ///     // Task logic
    /// });
    /// group.cancel();
    /// # });
    /// ```
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Wait for all tasks to complete (non-blocking).
    ///
    /// Awaits all tasks in the group to finish. Consumes the task handles.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::async::tasks::AsyncTaskGroup;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let group = AsyncTaskGroup::new();
    /// group.add_task(|| async {
    ///     smol::Timer::after(std::time::Duration::from_millis(10)).await;
    /// });
    /// group.wait_all().await;
    /// # });
    /// ```
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
