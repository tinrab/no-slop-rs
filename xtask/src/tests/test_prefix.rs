use super::findings;

#[test]
fn test_rejects_missing_prefix_and_points_to_the_function_name() {
    for name in [
        "works",
        "test",
        "Test_works",
        "checks_test_result",
        "r#works",
    ] {
        let source = format!("#[test]\nfn {name}() {{}}");
        let result = findings("require-test-prefix", &source);
        assert_eq!(result.len(), 1, "{source}");
        assert_eq!(&source[result[0].start.byte..result[0].end.byte], name);
        assert_eq!(result[0].start.line, 2);
        assert_eq!(result[0].start.column, 4);
    }
}

#[test]
fn test_accepts_prefixed_tests_and_leaves_helpers_alone() {
    for source in [
        "#[test] fn test_rejects_zero() {}",
        "#[test] fn r#test_rejects_zero() {}",
        "fn setup() {}",
        "#[cfg(test)] fn helper() {}",
        "#[cfg(test)] mod tests { fn setup() {} #[test] fn test_rejects_zero() {} }",
        "#[ignore] fn helper() {}",
        "#[cfg_attr(test, allow(dead_code, reason = \"test helper\"))] fn helper() {}",
    ] {
        assert!(
            findings("require-test-prefix", source).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn test_checks_qualified_async_tests_in_nested_modules() {
    for attribute in [
        "tokio::test",
        "tokio::test(flavor = \"current_thread\")",
        "async_std::test",
    ] {
        let source = format!("mod tests {{ #[{attribute}] async fn accepts_input() {{}} }}");
        assert_eq!(
            findings("require-test-prefix", &source).len(),
            1,
            "{source}"
        );
        let source = source.replace("fn accepts_input", "fn test_accepts_input");
        assert!(
            findings("require-test-prefix", &source).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn test_checks_conditional_and_ignored_tests_once_per_function() {
    for source in [
        "#[cfg_attr(test, test)] fn works() {}",
        "#[cfg_attr(feature = \"disabled\", tokio::test)] async fn works() {}",
        "#[cfg_attr(test, cfg_attr(unix, test))] fn works() {}",
        "#[test] #[cfg_attr(unix, test)] fn works() {}",
        "#[ignore] #[test] fn works() {}",
    ] {
        assert_eq!(findings("require-test-prefix", source).len(), 1, "{source}");
    }
    assert!(
        findings(
            "require-test-prefix",
            "#[cfg_attr(test, test)] fn test_works() {}"
        )
        .is_empty()
    );
}

#[test]
fn test_does_not_infer_aliases_or_expand_macro_generated_tests() {
    let source = r##"
        macro_rules! tests { () => { #[test] fn works() {} }; }
        fn fixture() { let source = "#[test] fn works() {}"; }
        #[test_case(1)] fn parameterized(value: u8) {}
        #[async_test] async fn imported_alias() {}
    "##;
    assert!(findings("require-test-prefix", source).is_empty());
}
