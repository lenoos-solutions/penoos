use proc_macro::TokenStream;

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
