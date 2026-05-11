use std::collections::HashSet;

use libtest_mimic_collect::{TestCollection, libtest_mimic};
use libtest_mimic_collect::test;

#[derive(Debug)]
pub struct Failing;

impl From<Failing> for libtest_mimic::Failed {
    fn from(value: Failing) -> Self {
        libtest_mimic::Failed::from(format!("{value:?}"))
    }
}

#[derive(Debug)]
pub struct Value(bool);

impl From<Value> for libtest_mimic::Completion {
    fn from(value: Value) -> Self {
        if value.0 {
            libtest_mimic::Completion::Completed
        } else {
            libtest_mimic::Completion::Ignored { reason: Some("Value was false".into()) }
        }
    }
}

#[test]
fn success1() {
    assert_eq!(1, 1);
}

#[test]
fn fail1() {
    assert_eq!(1, 2);
}

#[test]
fn fail2() -> Result<(), &'static str> {
    Err("This is a failing test")
}

#[test]
fn fail3() -> Result<(), String> {
    Err("This is a failing test".to_string())
}

#[test]
fn fail4() -> Result<(), libtest_mimic::Failed> {
    Err("This is a failing test".into())
}

#[test]
fn ignore1() -> Result<libtest_mimic::Completion, libtest_mimic::Failed> {
    Ok(libtest_mimic::Completion::Ignored { reason: Some("My ignore reason".into()) })
}

#[test]
fn success2() -> Result<libtest_mimic::Completion, libtest_mimic::Failed> {
    Ok(libtest_mimic::Completion::Completed)
}

#[test]
fn success3() -> Result<Value, Failing> {
    Ok(Value(true))
}

#[test]
fn ignore2() -> Result<Value, Failing> {
    Ok(Value(false))
}

#[test]
fn fail5() -> Result<Value, Failing> {
    Err(Failing)
}

pub fn main() {
    let tests = TestCollection::collect_tests();

    const EXPECTED_NAMES: [&str; 10] = [
        "fail1", "fail2", "fail3", "fail4", "fail5", "ignore1", "ignore2", "success1", "success2", "success3"
    ];
    let expected: HashSet<_> = EXPECTED_NAMES.into_iter().collect();
    let actual: HashSet<_> = tests.iter().map(libtest_mimic::Trial::name).collect();
    assert_eq!(actual, expected);

    let args = libtest_mimic::Arguments { test: true, quiet: true, ..Default::default() };

    // This does print a bunch of confusing stuff, but there is nothing that can be done. as we
    // want to ensure that tests fail when they should, tests are ignored when they should be and
    // pass when required
    let result = libtest_mimic::run(&args, tests);
    assert_eq!(result.num_failed, 5);
    assert_eq!(result.num_ignored, 2);
    assert_eq!(result.num_passed, 3);
    assert_eq!(result.num_measured, 0);
}
