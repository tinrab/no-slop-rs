use super::findings;

#[test]
fn test_reports_direct_error_stringification_with_parentheses_and_tail_blocks() {
    let source = r#"
        fn f() {
            let a = read().map_err(|error| error.to_string());
            let b = read().map_err((|error| { (error).to_string() }));
            let c = read().map_err(|mut error| { { (error.to_string()) } });
            let d = read().map_err(|r#type| r#type.to_string());
        }
    "#;
    assert_eq!(findings("no-stringified-error-mapping", source).len(), 4);
}

#[test]
fn test_allows_wrapping_presentation_and_unsupported_indirection() {
    let source = r#"
        fn f() {
            let a = read().map_err(|error| Failure { source: error });
            let b = read().map_err(Failure::from);
            let c = read().map_err(|error| { let error = other(); error.to_string() });
            let d = read().map_err(|error| other.to_string());
            let e = read().map_err(|error| format!("{error}"));
            let f = read().map_err(convert_error);
            let g = read().map_err(async |error| error.to_string());
            let h = read().map(|error| error.to_string());
            println!("{}", error.to_string());
            arbitrary! { read().map_err(|error| error.to_string()); }
        }
    "#;
    assert!(findings("no-stringified-error-mapping", source).is_empty());
}
