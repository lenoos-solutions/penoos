use proc_macro::TokenStream;

#[cfg(feature = "test_macros")]
#[proc_macro_attribute]
pub fn test_case(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Clean up the passed ID string (e.g., "TC-CFGMDL-001")
    let id = attr.to_string().trim_matches('"').to_string();
    let item_str = item.to_string();

    // Inject #[test] and a doc attribute containing the ID
    let output = format!(
        r#"
        #[test]
        #[doc = "Test Case ID: {id}"]
        {item_str}
        "#
    );

    output.parse().unwrap()
}

/// Test case macro for annotating tests with a specific ID.
/// This macro allows you to annotate a test function with a specific test case ID.
/// The ID is included as a doc attribute, making it easy to reference and identify test cases.
/// 
/// # Example
/// ```rust
/// use your_crate::test_case;
///
/// #[test_case("TC-CFGMDL-001")]
/// fn my_test() {
///     // Test logic here
/// }
/// ```
/// # Notes
/// The test case ID provided to the macro should be unique and descriptive, helping to identify and track individual test cases effectively.
/// It is recommended to follow a consistent naming convention for test case IDs to maintain clarity and organization across your test suite.
/// # Limitations
/// The macro does not enforce uniqueness of test case IDs at compile time.
/// It is the responsibility of the developer to ensure that each test case ID is unique within the test suite.
/// Developers should manually check for duplicate IDs to avoid conflicts and maintain the integrity of the test suite.
/// # Recommendations
/// Developers should establish a clear and consistent naming convention for test case IDs and maintain a registry of used IDs to prevent duplication and ensure traceability within the test suite.
/// Additionally, it is recommended to periodically review and clean up the registry to remove obsolete or retired test case IDs, keeping the test suite organized and up-to-date.
/// # Summary
/// The `test_case` macro provides a convenient way to annotate test functions with unique test case IDs, improving test traceability and organization.
/// By following the recommended practices and maintaining a consistent naming convention, developers can effectively manage and track test cases within their test suite.
/// Overall, the `test_case` macro enhances the maintainability and clarity of test suites by providing a standardized way to reference and document individual test cases.
/// This summary provides a concise overview of the purpose and benefits of using the `test_case` macro in your test suite.
/// # Conclusion
/// The `test_case` macro is a valuable tool for improving the organization, traceability, and maintainability of test suites by providing a standardized way to document and reference individual test cases.  
/// By consistently using this macro, developers can ensure that each test case is easily identifiable and properly documented, contributing to a more robust and manageable test suite.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro() {
        // Example test to ensure the macro works as expected
        #[test_case("TC-CFGMDL-001")]
        fn example_test() {
            // Test logic here
        }
    }
}
