//! Main test runner
#[cfg(test)]
mod async_test;
#[cfg(test)]
mod channels_test;
#[cfg(test)]
mod chars_test;
#[cfg(test)]
mod common_test;
#[cfg(test)]
mod data_test;
#[cfg(test)]
mod io_test;
#[cfg(test)]
mod memory_test;
#[cfg(test)]
mod parallel_test;
#[cfg(test)]
mod serde_test;
#[cfg(test)]
mod sys_test;

#[cfg(test)]
fn run_all_tests() {
    async_test::test_async();
    common_test::test_common();
    data_test::test_data();
    io_test::test_io();
    memory_test::test_memory();
    parallel_test::test_parallel();
    serde_test::test_serde();
    sys_test::test_sys();
}

fn main() {
    println!("Running tests for trash_utilities...");

    // Run tests from each module
    #[cfg(test)]
    run_all_tests();

    println!("All tests passed!");
}
