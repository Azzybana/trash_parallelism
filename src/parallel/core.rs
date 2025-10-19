/// Execute a closure.
///
/// This function executes the closure.
///
/// # Type Parameters
/// - `F`: The type of the closure, must be `FnOnce() -> R`.
/// - `R`: The return type.
///
/// # Parameters
/// - `f`: The closure to execute.
///
/// # Returns
/// The result of executing the closure.
///
/// # Examples
/// ```rust,no_run
/// use trash_analyzer::parallel::execute;
///
/// let result = execute(|| {
///     // Some computation
///     42
/// });
/// assert_eq!(result, 42);
/// ```
pub fn execute<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

/// Apply a function to each element of a vector in parallel.
///
/// This function provides a parallel-ready interface for mapping operations.
/// Currently uses sequential processing but is designed to be easily extended
/// with actual parallel execution backends.
///
/// # Type Parameters
/// - `T`: The input element type, must be `Send`.
/// - `U`: The output element type, must be `Send`.
/// - `F`: The mapping function type.
///
/// # Parameters
/// - `data`: The input vector to map over.
/// - `f`: The function to apply to each element.
///
/// # Returns
/// A new vector containing the results of applying `f` to each element.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_map;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let result = parallel_map(data, |x| x * 2);
/// assert_eq!(result, vec![2, 4, 6, 8, 10]);
/// ```
pub fn parallel_map<T, U, F>(data: Vec<T>, f: F) -> Vec<U>
where
    T: Send,
    U: Send,
    F: Fn(T) -> U + Sync + Send,
{
    data.into_iter().map(f).collect()
}

/// Apply a function to each element of a vector in parallel without collecting results.
///
/// This function provides a parallel-ready interface for executing a function on each
/// element of the input vector in parallel, discarding the results. Useful for
/// side-effect operations like I/O or logging. Currently uses sequential processing
/// but is designed to be easily extended with actual parallel execution backends.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send`.
/// - `F`: The function type.
///
/// # Parameters
/// - `data`: The input vector to iterate over.
/// - `f`: The function to apply to each element.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_for_each;
/// use std::sync::Mutex;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let sum = Mutex::new(0);
/// parallel_for_each(data, |x| {
///     let mut s = sum.lock().unwrap();
///     *s += x;
/// });
/// assert_eq!(*sum.lock().unwrap(), 15);
/// ```
pub fn parallel_for_each<T, F>(data: Vec<T>, f: F)
where
    T: Send,
    F: Fn(T) + Sync + Send,
{
    data.into_iter().for_each(f);
}

/// Combine all elements of a vector using a folding function in parallel.
///
/// This function performs a parallel fold operation, first folding chunks of the
/// data in parallel, then combining the results using the same folding function.
/// Currently uses sequential processing but is designed to be easily extended
/// with actual parallel execution backends.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
/// - `F`: The folding function type.
///
/// # Parameters
/// - `data`: The input vector to fold.
/// - `init`: The initial value for the fold.
/// - `f`: The folding function that takes two values and combines them.
///
/// # Returns
/// The final folded value.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_fold;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let sum = parallel_fold(data, 0, |acc, x| acc + x);
/// assert_eq!(sum, 15);
/// ```
pub fn parallel_fold<T, F>(data: Vec<T>, init: T, f: F) -> T
where
    T: Send + Sync,
    F: Fn(T, T) -> T + Sync + Send,
{
    data.into_iter().fold(init, f)
}

/// Apply a predicate to filter elements in parallel.
///
/// This function uses parallel processing to filter elements from a vector
/// based on a predicate function, returning only elements that satisfy the condition.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
/// - `F`: The predicate function type, must be `Fn(&T) -> bool + Send + Sync`.
///
/// # Parameters
/// - `data`: The input vector to filter.
/// - `predicate`: Function that returns true for elements to keep.
///
/// # Returns
/// A new vector containing only elements that satisfy the predicate.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_filter;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let evens = parallel_filter(data, |&x| x % 2 == 0);
/// assert_eq!(evens, vec![2, 4, 6, 8, 10]);
/// ```
pub fn parallel_filter<T, F>(data: Vec<T>, predicate: F) -> Vec<T>
where
    T: Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
{
    data.into_iter().filter(predicate).collect()
}
