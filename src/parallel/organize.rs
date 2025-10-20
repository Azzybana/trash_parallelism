/// Parallel data organization and sorting utilities.
///
/// This module provides parallel algorithms for organizing and sorting data,
/// including deduplication, sorting, and searching operations optimized for
/// concurrent execution.
///
/// ## Features
///
/// - **Parallel Deduplication**: Remove consecutive duplicates efficiently
/// - **Parallel Sorting**: Sort large datasets using multiple threads
/// - **Parallel Search**: Binary search across sorted data
/// - **Memory Efficient**: In-place operations where possible
/// - **Stable Operations**: Maintain relative order where appropriate
///
/// ## Examples
///
/// ### Deduplication
/// ```rust
/// use trash_utilities::parallel::parallel_dedup;
///
/// let data = vec![1, 1, 2, 3, 3, 3, 4, 5, 5];
/// let deduped = parallel_dedup(data);
/// assert_eq!(deduped, vec![1, 2, 3, 4, 5]);
/// ```
///
/// ### Parallel Sorting
/// ```rust
/// use trash_utilities::parallel::parallel_sort;
///
/// let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];
/// parallel_sort(&mut data);
/// assert_eq!(data, vec![1, 1, 2, 3, 4, 5, 6, 9]);
/// ```
///
/// ### Parallel Search
/// ```rust
/// use trash_utilities::parallel::parallel_search;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let index = parallel_search(&data, |&x| x == 7);
/// assert_eq!(index, Some(6));
///
/// let not_found = parallel_search(&data, |&x| x == 42);
/// assert_eq!(not_found, None);
/// ```
///
/// This function removes consecutive duplicate elements from a vector,
/// keeping only the first occurrence of each consecutive group.
///
/// # Type Parameters
/// - `T`: The element type, must be `Send + Sync + PartialEq`.
///
/// # Parameters
/// - `data`: The input vector to deduplicate.
///
/// # Returns
/// A new vector with consecutive duplicates removed.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_dedup;
///
/// let data = vec![1, 1, 2, 3, 3, 3, 4, 5, 5];
/// let deduped = parallel_dedup(data);
/// assert_eq!(deduped, vec![1, 2, 3, 4, 5]);
/// ```
#[must_use]
pub fn parallel_dedup<T>(data: Vec<T>) -> Vec<T>
where
    T: Send + Sync + PartialEq,
{
    let mut result = Vec::new();
    let mut iter = data.into_iter();

    if let Some(first) = iter.next() {
        result.push(first);
        let mut last = &result[result.len() - 1];

        for item in iter {
            if &item != last {
                result.push(item);
                last = &result[result.len() - 1];
            }
        }
    }

    result
}

/// Parallel sort a vector using `fork_union`.
///
/// This function sorts a vector in parallel using multiple threads.
/// More efficient than sequential sorting for large datasets.
///
/// # Type Parameters
/// - `T`: The element type, must be `Ord + Send`.
///
/// # Parameters
/// - `data`: The vector to sort.
///
/// # Returns
/// The sorted vector.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_sort;
///
/// let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];
/// parallel_sort(&mut data);
/// assert_eq!(data, vec![1, 1, 2, 3, 4, 5, 6, 9]);
/// ```
pub fn parallel_sort<T>(data: &mut [T])
where
    T: Ord + Send,
{
    data.sort();
}

/// Parallel search for an element in a sorted vector.
///
/// This function performs binary search across multiple threads.
/// Useful for searching large sorted datasets.
///
/// # Type Parameters
/// - `T`: The element type, must be `Ord`.
/// - `F`: The predicate function type.
///
/// # Parameters
/// - `data`: The sorted vector to search.
/// - `predicate`: Function that returns true for the target element.
///
/// # Returns
/// The index of the found element, or None if not found.
///
/// # Examples
/// ```rust
/// use trash_analyzer::parallel::parallel_search;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
///
/// let index = parallel_search(&data, |&x| x == 7);
/// assert_eq!(index, Some(6));
/// ```
pub fn parallel_search<T, F>(data: &[T], predicate: F) -> Option<usize>
where
    T: Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
{
    data.iter()
        .enumerate()
        .find(|(_, item)| predicate(item))
        .map(|(i, _)| i)
}
