use libtest_mimic::{Arguments, Failed, Trial};
use std::sync::Mutex;

static TESTS: Mutex<Vec<Trial>> = Mutex::new(Vec::new());

/// This macro automatically adds tests marked with #[test] to the test collection.
/// Tests then can be run with libtest_mimic_collect::TestCollection::run().
pub use libtest_mimic_collect_macro::test;
/// This macro is used by #[test] to add tests to the test collection.
pub use ctor::ctor;

/// A global collection of tests.
/// Tests can be added to the collection from different modules and then run.
pub struct TestCollection {}

impl TestCollection {
    /// Adds a test trial to the test collection.
    pub fn add_test(trial: Trial) {
        TESTS.lock().unwrap().push(trial);
    }

    /// Returns the collected tests and clears the storage.
    pub fn collect_tests() -> Vec<Trial> {
        std::mem::take(TESTS.lock().unwrap().as_mut())
    }

    /// Runs all the collected tests.
    pub fn run() {
        let args = Arguments::from_args();
        libtest_mimic::run(&args, TestCollection::collect_tests()).exit();
    }
}

/// Converts typical test function results to the Result type used by libtest_mimic.
pub trait ConvertResult<T> {
    fn convert_result(result: T) -> Result<(), Failed>;
}

impl ConvertResult<()> for TestCollection {
    fn convert_result(_: ()) -> Result<(), Failed> {
        Ok(())
    }
}

impl ConvertResult<Result<(), Failed>> for TestCollection {
    fn convert_result(result: Result<(), Failed>) -> Result<(), Failed> {
        result
    }
}

impl ConvertResult<Result<(), &str>> for TestCollection {
    fn convert_result(result: Result<(), &str>) -> Result<(), Failed> {
        result.map_err(|e| e.into())
    }
}

impl ConvertResult<Result<(), String>> for TestCollection {
    fn convert_result(result: Result<(), String>) -> Result<(), Failed> {
        result.map_err(|e| e.into())
    }
}
