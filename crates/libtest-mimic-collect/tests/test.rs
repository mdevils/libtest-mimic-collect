use std::collections::HashSet;

use libtest_mimic_collect::test;
use libtest_mimic_collect::{libtest_mimic, ConvertResult, TestCollection};

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
            libtest_mimic::Completion::Ignored {
                reason: Some("Value was false".into()),
            }
        }
    }
}

pub struct CustomReturn(bool);

impl ConvertResult<CustomReturn> for TestCollection {
    fn convert_result(value: CustomReturn) -> Result<(), libtest_mimic::Failed> {
        if value.0 {
            Ok(())
        } else {
            Err(libtest_mimic::Failed::from("CustomReturn was false"))
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
    Ok(libtest_mimic::Completion::Ignored {
        reason: Some("My ignore reason".into()),
    })
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

#[test]
fn fail6() -> Result<(), libtest_mimic::Failed> {
    panic!("panicking inside a Result<(), Failed>");
}

#[test]
fn ignore3() -> Result<libtest_mimic::Completion, libtest_mimic::Failed> {
    Ok(libtest_mimic::Completion::Ignored { reason: None })
}

#[test]
fn success4() -> CustomReturn {
    CustomReturn(true)
}

#[test]
fn fail7() -> CustomReturn {
    CustomReturn(false)
}

pub fn main() {
    let tests = TestCollection::collect_tests();

    const EXPECTED_NAMES: [&str; 14] = [
        "fail1", "fail2", "fail3", "fail4", "fail5", "fail6", "fail7", "ignore1", "ignore2",
        "ignore3", "success1", "success2", "success3", "success4",
    ];
    let expected: HashSet<_> = EXPECTED_NAMES.into_iter().collect();
    let actual: HashSet<_> = tests.iter().map(libtest_mimic::Trial::name).collect();
    assert_eq!(actual, expected);

    let args = libtest_mimic::Arguments {
        test: true,
        quiet: true,
        ..Default::default()
    };

    // This does print a bunch of confusing stuff, but there is nothing that can be done. as we
    // want to ensure that tests fail when they should, tests are ignored when they should be and
    // pass when required
    let result = libtest_mimic::run(&args, tests);
    assert_eq!(result.num_failed, 7);
    assert_eq!(result.num_ignored, 3);
    assert_eq!(result.num_passed, 4);
    assert_eq!(result.num_measured, 0);
}
