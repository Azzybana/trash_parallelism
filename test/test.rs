//! Main test runner for trash_utilities

mod async_test;
mod channels_test;
mod chars_test;
mod common_test;
mod data_test;
mod io_test;
mod lib_test;
mod memory_test;
mod parallel_test;
mod serde_test;
mod sys_test;

fn main() {
    println!("Running tests for trash_utilities...");

    // Run tests from each module
    async_test::tests::test_async_basic();
    channels_test::tests::test_channels_basic();
    chars_test::tests::test_chars_basic();
    common_test::tests::test_common_basic();
    data_test::tests::test_data_basic();
    io_test::tests::test_io_basic();
    lib_test::tests::test_lib_basic();
    memory_test::tests::test_memory_basic();
    parallel_test::tests::test_parallel_basic();
    serde_test::tests::test_serde_basic();
    sys_test::tests::test_sys_basic();

    println!("All tests passed!");
}