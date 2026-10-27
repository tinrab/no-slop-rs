use super::findings;

#[test]
fn test_reports_direct_closure_and_qualified_filter_patterns() {
    let result = findings(
        "no-silent-error-discard",
        r#"
        fn f() { let saved = value.ok(); iter.filter_map(|x| x.ok());
            iter.filter_map(Result::ok); iter.flat_map(std::result::Result::ok); }
    "#,
    );
    assert_eq!(result.len(), 4);
}

#[test]
fn test_allows_preserved_failures_and_unrelated_filters() {
    assert!(
        findings(
            "no-silent-error-discard",
            r#"
        fn f() { value.ok(); (value.ok()); let _ = value; iter.map(parse).collect::<Result<Vec<_>, _>>();
            option.ok_or(Missing); iter.filter_map(lookup); }
    "#
        )
        .is_empty()
    );
}
