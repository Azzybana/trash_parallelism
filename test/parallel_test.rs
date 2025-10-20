//! Tests for the parallel module
use trash_parallelism::parallel::*;

#[test]
pub fn test_parallel_map() {
    let data = vec![1, 2, 3, 4, 5];
    let result = parallel_map(data, |x| x * 2);
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
}

#[test]
pub fn test_parallel_for_each() {
    let data = vec![1, 2, 3, 4, 5];
    let sum = std::sync::Mutex::new(0);
    parallel_for_each(data, |x| {
        let mut s = sum.lock().unwrap();
        *s += x;
    });
    assert_eq!(*sum.lock().unwrap(), 15);
}

#[test]
pub fn test_parallel_fold() {
    let data = vec![1, 2, 3, 4, 5];
    let sum = parallel_fold(data, 0, |acc, x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
pub fn test_parallel_filter() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let evens = parallel_filter(data, |&x| x % 2 == 0);
    assert_eq!(evens, vec![2, 4, 6, 8, 10]);
}

#[test]
pub fn test_parallel_partition() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let (evens, odds) = parallel_partition(data, |&x| x % 2 == 0);
    assert_eq!(evens, vec![2, 4, 6, 8, 10]);
    assert_eq!(odds, vec![1, 3, 5, 7, 9]);
}

#[test]
pub fn test_parallel_group_by() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let groups = parallel_group_by(data, |&x| x % 3);
    assert_eq!(groups.get(&0), Some(&vec![3, 6, 9]));
    assert_eq!(groups.get(&1), Some(&vec![1, 4, 7, 10]));
    assert_eq!(groups.get(&2), Some(&vec![2, 5, 8]));
}

#[test]
pub fn test_parallel_chunks() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let chunks = parallel_chunks(data, 3);
    assert_eq!(
        chunks,
        vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9], vec![10]]
    );
}

#[test]
pub fn test_parallel_windows() {
    let data = vec![1, 2, 3, 4, 5];
    let windows = parallel_windows(&data, 3);
    assert_eq!(windows, vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]);
}

#[test]
pub fn test_parallel_dedup() {
    let data = vec![1, 1, 2, 3, 3, 3, 4, 5, 5];
    let deduped = parallel_dedup(data);
    assert_eq!(deduped, vec![1, 2, 3, 4, 5]);
}

#[test]
pub fn test_parallel_sort() {
    let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];
    parallel_sort(&mut data);
    assert_eq!(data, vec![1, 1, 2, 3, 4, 5, 6, 9]);
}

#[test]
pub fn test_parallel_search() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let index = parallel_search(&data, |&x| x == 7);
    assert_eq!(index, Some(6));
    let not_found = parallel_search(&data, |&x| x == 42);
    assert_eq!(not_found, None);
}

#[test]
pub fn test_thread_pool_monitor() {
    let monitor = ThreadPoolMonitor::new();
    let stats = monitor.stats();
    assert_eq!(stats.total_operations, 0);
    assert_eq!(stats.active_operations, 0);
}

#[test]
pub fn test_monitored_execute() {
    let monitor = ThreadPoolMonitor::new();
    let result = monitored_execute(&monitor, "test", || 42);
    assert_eq!(result, 42);
    let stats = monitor.stats();
    assert_eq!(stats.total_operations, 1);
}

#[test]
pub fn test_distribute_work() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let results = distribute_work(&data, |chunk| chunk.iter().sum::<i32>());
    let total: i32 = results.iter().sum();
    assert_eq!(total, 55);
}

#[test]
pub fn test_parallel_map_with_cancellation() {
    let data = vec![1, 2, 3, 4, 5];
    let token = smol_cancellation_token::CancellationToken::new();
    let result = parallel_map_with_cancellation(data, |x| x * 2, &token);
    assert_eq!(result, Some(vec![2, 4, 6, 8, 10]));
}

#[test]
pub fn test_parallel() {
    test_parallel_map();
    test_parallel_for_each();
    test_parallel_fold();
    test_parallel_filter();
    test_parallel_partition();
    test_parallel_group_by();
    test_parallel_chunks();
    test_parallel_windows();
    test_parallel_dedup();
    test_parallel_sort();
    test_parallel_search();
    test_thread_pool_monitor();
    test_monitored_execute();
    test_distribute_work();
    test_parallel_map_with_cancellation();
}
