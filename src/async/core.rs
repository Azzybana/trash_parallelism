/// Core async utilities providing fundamental building blocks for asynchronous operations.
///
/// This module includes basic async helpers like sleep, race, join, cancellation tokens,
/// and mutex creation, leveraging smol runtime for optimal performance.
///
/// # Examples
///
/// Basic sleep and join:
/// ```rust
/// use trash_utilities::async::core::{sleep_for, join};
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// let (result1, result2) = join(
///     async { sleep_for(Duration::from_millis(10)).await; 1 },
///     async { 2 }
/// ).await;
/// assert_eq!((result1, result2), (1, 2));
/// # });
/// ```
// Standard library imports
use std::time::Duration;

// External crate imports
use futures_lite;
use parking_lot::Mutex;
use smol::Timer;
use smol_cancellation_token::CancellationToken;

/// Async sleep helper.
///
/// Pauses execution for the specified duration using the smol runtime.
///
/// # Parameters
///
/// * `duration` - The amount of time to sleep.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::core::sleep_for;
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// sleep_for(Duration::from_millis(100)).await;
/// println!("Slept for 100ms");
/// # });
/// ```
pub async fn sleep_for(duration: Duration) {
    Timer::after(duration).await;
}

/// Futures-lite helper: race two futures.
///
/// Returns the result of the first future to complete, cancelling the other.
///
/// # Parameters
///
/// * `f1` - The first future.
/// * `f2` - The second future.
///
/// # Type Parameters
///
/// * `T` - The output type of both futures.
/// * `F1` - The type of the first future.
/// * `F2` - The type of the second future.
///
/// # Returns
///
/// The result of whichever future completes first.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::core::{race, sleep_for};
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// let result = race(
///     async { sleep_for(Duration::from_millis(10)).await; "slow" },
///     async { "fast" }
/// ).await;
/// assert_eq!(result, "fast");
/// # });
/// ```
pub async fn race<T, F1, F2>(f1: F1, f2: F2) -> T
where
    F1: std::future::Future<Output = T>,
    F2: std::future::Future<Output = T>,
{
    futures_lite::future::race(f1, f2).await
}

/// Futures-lite helper: join two futures.
///
/// Runs both futures concurrently and returns their results as a tuple.
///
/// # Parameters
///
/// * `f1` - The first future.
/// * `f2` - The second future.
///
/// # Type Parameters
///
/// * `T1` - The output type of the first future.
/// * `T2` - The output type of the second future.
/// * `F1` - The type of the first future.
/// * `F2` - The type of the second future.
///
/// # Returns
///
/// A tuple containing the results of both futures.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::core::join;
/// use smol;
///
/// # smol::block_on(async {
/// let (result1, result2) = join(
///     async { 1 },
///     async { 2 }
/// ).await;
/// assert_eq!((result1, result2), (1, 2));
/// # });
/// ```
pub async fn join<T1, T2, F1, F2>(f1: F1, f2: F2) -> (T1, T2)
where
    F1: std::future::Future<Output = T1>,
    F2: std::future::Future<Output = T2>,
{
    futures_lite::future::zip(f1, f2).await
}

/// Cancellation token helper.
///
/// Creates a new cancellation token that can be used to cancel async operations.
///
/// # Returns
///
/// A new `CancellationToken` instance.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::core::create_cancellation_token;
///
/// let token = create_cancellation_token();
/// // Use token to cancel operations
/// ```
#[must_use]
pub fn create_cancellation_token() -> CancellationToken {
    CancellationToken::new()
}

/// Run a future with cancellation.
///
/// Executes the future, but returns `None` if the token is cancelled before completion.
///
/// # Parameters
///
/// * `token` - The cancellation token to monitor.
/// * `future` - The future to execute.
///
/// # Type Parameters
///
/// * `T` - The output type of the future.
/// * `F` - The type of the future.
///
/// # Returns
///
/// `Some(result)` if the future completes, or `None` if cancelled.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::core::{create_cancellation_token, with_cancellation};
/// use smol;
/// use std::time::Duration;
///
/// # smol::block_on(async {
/// let token = create_cancellation_token();
/// let result = with_cancellation(&token, async {
///     smol::Timer::after(Duration::from_millis(10)).await;
///     42
/// }).await;
/// assert_eq!(result, Some(42));
/// # });
/// ```
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

/// Parking lot mutex helper.
///
/// Creates a new mutex with the given initial value.
///
/// # Parameters
///
/// * `value` - The initial value to store in the mutex.
///
/// # Type Parameters
///
/// * `T` - The type of the value.
///
/// # Returns
///
/// A new `Mutex<T>` instance.
///
/// # Examples
///
/// ```rust
/// use trash_utilities::async::core::create_mutex;
///
/// let mutex = create_mutex(42);
/// let value = mutex.lock();
/// assert_eq!(*value, 42);
/// ```
pub fn create_mutex<T>(value: T) -> Mutex<T> {
    Mutex::new(value)
}
