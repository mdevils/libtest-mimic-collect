# libtest-mimic-collect

Provides a global storage for tests to run them using `libtest-mimic` crate.

Can be used to collect tests from different modules and run them all at once.

You can use [ctor](https://crates.io/crates/ctor) to collect tests explicitly
or use `libtest-mimic-collect-macro` to collect tests automatically.

## Installation

* Add `libtest-mimic-collect` to the dependencies.
* Also add `libtest-mimic` if you want to import `Failed` type.
* Also add `libtest-mimic-collect-macro` if you want to use the macro.

## Example using `libtest-mimic-collect-macro`

```rust
mod my_mod1;
mod my_mod2;
// ...

#[macro_use]
extern crate libtest_mimic_collect_macro;

#[test]
fn test_success() {
    ()
}

#[test]
fn test_failure() -> Result<(), String> {
    Err("Something went wrong".into())
}

#[test]
fn test_assert() {
    assert_eq!(1, 2);
}

pub fn main() {
    libtest_mimic_collect::TestCollection::run();
}
```

## Example using `ctor`

```rust
mod my_mod1;
mod my_mod2;
// ...

use libtest_mimic::Failed;

fn test_success() -> Result<(), Failed> {
    Ok(())
}

fn test_failure() -> Result<(), Failed> {
    Err("Something went wrong".into())
}

fn test_assert() -> Result<(), Failed> {
    if 1 != 2 {
        Err("1 != 2".into())
    } else {
        Ok(())
    }
}

#[ctor::ctor]
fn register_tests() {
    use libtest_mimic_collect::TestCollection;

    TestCollection::add_test("test_success", test_success);
    TestCollection::add_test("test_failure", test_failure);
    TestCollection::add_test("test_assert", test_assert);
}

pub fn main() {
    libtest_mimic_collect::TestCollection::run();
}
```

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](../libtest-mimic-collect/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
  ([LICENSE-MIT](../libtest-mimic-collect/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
