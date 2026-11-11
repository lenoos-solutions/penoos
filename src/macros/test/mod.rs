//! Test macros for Penoos, enabled by the `test_macros` feature.
//!
//! - [`test_case!`]: declares a `#[test]` function tagged with a test case ID.

/// Declares a `#[test]` function whose documentation carries a test case ID.
///
/// ```ignore
/// test_case!("TC-CFGMDL-001", fn my_test() {
///     assert!(true);
/// });
/// ```
#[cfg(feature = "test_macros")]
#[macro_export]
macro_rules! test_case {
    ($id:literal, fn $name:ident () $body:block) => {
        #[test]
        #[doc = concat!("Test Case ID: ", $id)]
        fn $name() $body
    };
}

#[cfg(feature = "test_macros")]
pub use crate::test_case;

#[cfg(all(test, feature = "test_macros"))]
mod tests {
    crate::test_case!(
        "TC-MACRO-001",
        fn declared_case_runs_as_test() {
            assert_eq!(2 + 2, 4);
        }
    );
}
