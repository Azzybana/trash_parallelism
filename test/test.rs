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
mod lib_test;
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
    async_test::tests::test_async_basic();
    channels_test::tests::test_channels_basic();
    chars_test::tests::test_chars_basic();
    common_test::tests::test_common_basic();
    data_test::tests::test_data_basic();
    io_test::tests::test_io_basic();
    lib_test::tests::test_lib_basic();
    memory_test::tests::test_memory_basic();
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
