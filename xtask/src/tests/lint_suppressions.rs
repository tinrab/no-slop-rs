use super::findings;

#[test]
fn test_reports_broad_groups_even_with_reasons() {
    assert_eq!(
        findings(
            "no-blanket-lint-suppression",
            r#"
        #![allow(warnings, reason = "legacy")]
        #[cfg_attr(feature = "x", expect(clippy::all, reason = "legacy"))] fn f() {}
    "#
        )
        .len(),
        2
    );
}

#[test]
fn test_accepts_narrow_reasoned_exceptions() {
    let source = r#"#[allow(dead_code, reason = "used through FFI")] fn f() {}"#;
    assert!(findings("no-blanket-lint-suppression", source).is_empty());
}
