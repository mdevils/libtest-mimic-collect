# libtest-mimic-collect

Provides a global storage for tests to run them using `libtest-mimic` crate.

Can be used to collect tests from different modules and run them all at once.

You can use [ctor](https://crates.io/crates/ctor) to collect tests explicitly
or use `libtest-mimic-collect-macro` to collect tests automatically.

## Installation

* Add `libtest-mimic-collect` to the dev-dependencies.

## Example

Specify your test target in `Cargo.toml`:

```toml
[[test]]
name = "test"
harness = false
path = "lib/test.rs"
```

You might also disable the default tests:

```toml
[lib]
test = false
```

Create a test module `lib/test.rs`:

```rust
mod my_mod1;
mod my_mod2;
// ...

#[macro_use]
extern crate libtest_mimic_collect;

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
