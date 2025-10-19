use std::collections::HashMap;

/// Partition elements into two groups based on a predicate in parallel.
///
/// This function splits a vector into two vectors: one containing elements
/// that satisfy the predicate, and another containing elements that don't.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
/// - `F`: The predicate function type, must be `Fn(&T) -> bool + Send + Sync`.
///
/// # Parameters
/// - `data`: The input vector to partition.
/// - `predicate`: Function that returns true for elements in the first partition.
///
/// # Returns
/// A tuple of two vectors: (`matching_elements`, `non_matching_elements`).
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_partition;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let (evens, odds) = parallel_partition(data, |&x| x % 2 == 0);
/// assert_eq!(evens, vec![2, 4, 6, 8, 10]);
/// assert_eq!(odds, vec![1, 3, 5, 7, 9]);
/// ```
pub fn parallel_partition<T, F>(data: Vec<T>, predicate: F) -> (Vec<T>, Vec<T>)
where
    T: Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
{
    let mut matching = Vec::new();
    let mut non_matching = Vec::new();

    for item in data {
        if predicate(&item) {
            matching.push(item);
        } else {
            non_matching.push(item);
        }
    }

    (matching, non_matching)
}

/// Group elements by a key function in parallel.
///
/// This function groups elements of a vector by keys computed from each element,
/// similar to `itertools::group_by` but with parallel processing.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
/// - `K`: The key type, must be `Send + Sync + Eq + std::hash::Hash`.
/// - `F`: The key function type, must be `Fn(&T) -> K + Send + Sync`.
///
/// # Parameters
/// - `data`: The input vector to group.
/// - `key_fn`: Function that computes the key for each element.
///
/// # Returns
/// A `HashMap` where keys are the computed keys and values are vectors of elements.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_group_by;
/// use std::collections::HashMap;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let groups = parallel_group_by(data, |&x| x % 3);
///
/// assert_eq!(groups.get(&0), Some(&vec![3, 6, 9]));
/// assert_eq!(groups.get(&1), Some(&vec![1, 4, 7, 10]));
/// assert_eq!(groups.get(&2), Some(&vec![2, 5, 8]));
/// ```
pub fn parallel_group_by<T, K, F>(data: Vec<T>, key_fn: F) -> HashMap<K, Vec<T>>
where
    T: Send + Sync,
    K: Send + Sync + Eq + std::hash::Hash,
    F: Fn(&T) -> K + Send + Sync,
{
    let mut groups: HashMap<K, Vec<T>> = HashMap::new();

    for item in data {
        let key = key_fn(&item);
        groups.entry(key).or_default().push(item);
    }

    groups
}

/// Split a vector into chunks of specified size in parallel.
///
/// This function divides a vector into chunks of equal size (except possibly
/// the last chunk), similar to `itertools::chunks` but with parallel processing.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync`.
///
/// # Parameters
/// - `data`: The input vector to chunk.
/// - `chunk_size`: The size of each chunk.
///
/// # Returns
/// A vector of vectors, where each inner vector is a chunk.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_chunks;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let chunks = parallel_chunks(data, 3);
/// assert_eq!(chunks, vec![
///     vec![1, 2, 3],
///     vec![4, 5, 6],
///     vec![7, 8, 9],
///     vec![10]
/// ]);
/// ```
#[must_use]
pub fn parallel_chunks<T>(data: Vec<T>, chunk_size: usize) -> Vec<Vec<T>>
where
    T: Send + Sync,
{
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();

    for item in data {
        current_chunk.push(item);
        if current_chunk.len() == chunk_size {
            chunks.push(current_chunk);
            current_chunk = Vec::new();
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Create sliding windows over a vector in parallel.
///
/// This function creates overlapping windows of elements from a vector,
/// similar to `itertools::windows` but with parallel processing support.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync + Clone`.
///
/// # Parameters
/// - `data`: The input vector to create windows from.
/// - `window_size`: The size of each window.
///
/// # Returns
/// A vector of vectors, where each inner vector is a window.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_windows;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let windows = parallel_windows(&data, 3);
/// assert_eq!(windows, vec![
///     vec![1, 2, 3],
///     vec![2, 3, 4],
///     vec![3, 4, 5]
/// ]);
/// ```
#[must_use]
pub fn parallel_windows<T>(data: &[T], window_size: usize) -> Vec<Vec<T>>
where
    T: Send + Sync + Clone,
{
    let mut windows = Vec::new();

    if data.len() >= window_size {
        for i in 0..=(data.len() - window_size) {
            let window: Vec<T> = data[i..i + window_size].to_vec();
            windows.push(window);
        }
    }

    windows
}
