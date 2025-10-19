/// Core async utilities providing fundamental building blocks for asynchronous operations.
///
/// This module includes basic async helpers like sleep, race, join, cancellation tokens,
/// and mutex creation, leveraging smol runtime for optimal performance.
// Standard library imports
use std::time::Duration;

// External crate imports
use futures_lite;
use parking_lot::Mutex;
use smol::Timer;
use smol_cancellation_token::CancellationToken;

/// Async sleep helper
pub async fn sleep_for(duration: Duration) {
    Timer::after(duration).await;
}

/// Futures-lite helper: race two futures
pub async fn race<T, F1, F2>(f1: F1, f2: F2) -> T
where
    F1: std::future::Future<Output = T>,
    F2: std::future::Future<Output = T>,
{
    futures_lite::future::race(f1, f2).await
}

/// Futures-lite helper: join two futures
pub async fn join<T1, T2, F1, F2>(f1: F1, f2: F2) -> (T1, T2)
where
    F1: std::future::Future<Output = T1>,
    F2: std::future::Future<Output = T2>,
{
    futures_lite::future::zip(f1, f2).await
}

/// Cancellation token helper
#[must_use]
pub fn create_cancellation_token() -> CancellationToken {
    CancellationToken::new()
}

/// Run a future with cancellation
pub async fn with_cancellation<T, F>(token: &CancellationToken, future: F) -> Option<T>
where
    F: std::future::Future<Output = T>,
{
    futures_lite::future::or(
        async {
            token.cancelled().await;
            None
        },
        async { Some(future.await) },
    )
    .await
}

/// Parking lot mutex helper
pub fn create_mutex<T>(value: T) -> Mutex<T> {
    Mutex::new(value)
}
