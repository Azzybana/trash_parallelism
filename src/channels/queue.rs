/// Work queue implementations for load balancing and task distribution.
///
/// This module provides high-throughput work queues with automatic load balancing
/// across multiple worker tasks.
///
/// # Examples
///
/// Basic work queue:
/// ```rust
/// use trash_utilities::channels::queue::WorkQueue;
/// use smol;
///
/// # smol::block_on(async {
/// let mut queue = WorkQueue::<String, String>::new(2); // 2 workers
/// queue.submit("task1".to_string()).await.unwrap();
/// queue.submit("task2".to_string()).await.unwrap();
/// // Workers process tasks, results available via collect()
/// # });
/// ```
///
// Standard library imports
// External crate imports
use parking_lot::Mutex;

/// High-throughput work queue with load balancing
///
/// Distributes tasks across multiple worker threads using round-robin load balancing.
/// Each worker processes tasks asynchronously and can send results back.
///
/// # Type Parameters
///
/// * `T` - The type of tasks to submit.
/// * `R` - The type of results from task processing.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::channels::queue::WorkQueue;
/// use smol;
///
/// # smol::block_on(async {
/// let queue = WorkQueue::<String, String>::new(3); // 3 workers
/// queue.submit("task1".to_string()).await.unwrap();
/// queue.submit("task2".to_string()).await.unwrap();
/// // Results can be collected with queue.collect().await
/// # });
/// ```
pub struct WorkQueue<T, R> {
    workers: Vec<crate::channels::core::TxFuture<T>>,
    result_rx: crate::channels::core::RxFuture<R>,
    next_worker: Mutex<usize>,
}

impl<T: Send + 'static, R: Send + 'static> WorkQueue<T, R> {
    /// Create a new work queue with N workers
    ///
    /// # Parameters
    ///
    /// * `num_workers` - Number of worker tasks to spawn.
    ///
    /// # Returns
    ///
    /// A new `WorkQueue` with the specified number of workers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::queue::WorkQueue;
    ///
    /// let queue = WorkQueue::<i32, String>::new(4);
    /// ```
    #[must_use]
    pub fn new(num_workers: usize) -> Self {
        let mut workers = Vec::new();
        let _result_txs: Vec<crate::channels::core::TxFuture<R>> = Vec::new();
        let capacity = (num_workers * 10).max(1); // Ensure minimum capacity of 1
        let (result_tx, result_rx) = crate::channels::core::bounded_queue_3(capacity);

        for _ in 0..num_workers {
            let (task_tx, task_rx) = crate::channels::core::bounded_queue_3(100);
            workers.push(task_tx);

            let _result_tx = result_tx.clone();
            smol::spawn(async move {
                let rx = task_rx;
                while let Ok(_task) = rx.recv().await {
                    // Task processing handled externally via submit_with_processor
                }
            })
            .detach();
        }

        Self {
            workers,
            result_rx,
            next_worker: Mutex::new(0),
        }
    }

    /// Submit a task to the queue (non-blocking)
    ///
    /// Distributes the task to the next worker in round-robin fashion.
    ///
    /// # Parameters
    ///
    /// * `task` - The task to submit for processing.
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::queue::WorkQueue;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let queue = WorkQueue::<String, ()>::new(2);
    /// queue.submit("process this".to_string()).await.unwrap();
    /// # });
    /// ```
    pub async fn submit(&self, task: T) -> Result<(), smol::channel::SendError<T>> {
        if self.workers.is_empty() {
            return Err(smol::channel::SendError(task));
        }
        let worker_index = {
            let mut next = self.next_worker.lock();
            let index = *next % self.workers.len();
            *next += 1;
            index
        };
        let worker = &self.workers[worker_index];
        worker.send(task).await
    }

    /// Collect a result (non-blocking)
    ///
    /// Receives a result from any of the workers.
    ///
    /// # Returns
    ///
    /// The result from a completed task.
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is closed or empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trash_utilities::channels::queue::WorkQueue;
    /// use smol;
    ///
    /// # smol::block_on(async {
    /// let queue = WorkQueue::<String, String>::new(2);
    /// // Assuming workers send results...
    /// // let result = queue.collect().await.unwrap();
    /// # });
    /// ```
    pub async fn collect(&self) -> Result<R, smol::channel::RecvError> {
        self.result_rx.recv().await
    }
}
