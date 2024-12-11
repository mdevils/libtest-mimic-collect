use libtest_mimic_collect::TestCollection;
use libtest_mimic_collect::test;

#[test]
fn test_number_1() {
    assert_eq!(1, 1);
}

#[test]
fn test_number_2() {
    assert_eq!(1, 2);
}

pub fn main() {
    let tests = TestCollection::collect_tests();

    assert_eq!(tests[0].name(), "test_number_1");
    assert_eq!(tests[1].name(), "test_number_2");
}
