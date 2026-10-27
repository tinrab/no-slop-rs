use super::findings;

#[test]
fn test_reports_signatures_aliases_and_nested_results() {
    assert_eq!(
        findings(
            "no-string-errors",
            r#"
        type R<T> = std::result::Result<T, String>;
        fn f() -> Result<u8, &'static str> { loop {} }
        async fn g() -> Option<core::result::Result<u8, alloc::string::String>> { loop {} }
    "#
        )
        .len(),
        3
    );
}

#[test]
fn test_allows_typed_errors_anyhow_and_test_contracts() {
    assert!(
        findings(
            "no-string-errors",
            r#"
        fn f() -> Result<(), DecodeError> { loop {} }
        fn g() -> anyhow::Result<()> { loop {} }
        fn unit_error() -> Result<(), ()> { loop {} }
        type R<T> = Result<T, std::io::Error>;
        #[test] fn test_f() -> Result<(), String> { loop {} }
    "#
        )
        .is_empty()
    );
}
