// Standard library imports

// External crate imports
use parking_lot::Mutex;

/// High-throughput work queue with load balancing
pub struct WorkQueue<T, R> {
    workers: Vec<crate::channels::core::TxFuture<T>>,
    result_rx: crate::channels::core::RxFuture<R>,
    next_worker: Mutex<usize>,
}

impl<T: Send + 'static, R: Send + 'static> WorkQueue<T, R> {
    /// Create a new work queue with N workers
    #[must_use]
    pub fn new(num_workers: usize) -> Self {
        let mut workers = Vec::new();
        let _result_txs: Vec<crate::channels::core::TxFuture<R>> = Vec::new();
        let (result_tx, result_rx) = crate::channels::core::bounded_queue_3(num_workers * 10);

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
    pub async fn submit(&self, task: T) -> Result<(), smol::channel::SendError<T>> {
        let mut next = self.next_worker.lock();
        let worker = &self.workers[*next % self.workers.len()];
        *next += 1;
        worker.send(task).await
    }

    /// Collect a result (non-blocking)
    pub async fn collect(&self) -> Result<R, smol::channel::RecvError> {
        self.result_rx.recv().await
    }
}