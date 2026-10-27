//! Test-name placeholders reuse exactly the existing test-attribute recognition.

use super::findings;

const RULE: &str = "no-vague-test-names";

#[test]
fn test_rejects_dictionary_empty_and_ascii_numeric_suffixes() {
    for name in [
        "test_basic",
        "test_works",
        "test_example",
        "test_simple",
        "test_test",
        "test_case",
        "test_",
        "test_1",
        "test_007",
        "r#test_basic",
    ] {
        let source = format!("#[test] fn {name}() {{}}");
        let diagnostics = findings(RULE, &source);
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert_eq!(
            &source[diagnostics[0].start.byte..diagnostics[0].end.byte],
            name
        );
        assert!(diagnostics[0].fix.is_none());
    }
}

#[test]
fn test_accepts_behavior_names_and_leaves_helpers_and_missing_prefixes_alone() {
    for source in [
        "#[test] fn test_parse() {}",
        "#[test] fn test_smoke() {}",
        "#[test] fn test_rejects_expired_token() {}",
        "#[test] fn test_basic_auth() {}",
        "#[test] fn test_case_1() {}",
        "#[test] fn works() {}",
        "#[test] fn test() {}",
        "fn test_basic() {}",
        "#[cfg(test)] fn test_basic() {}",
        "#[test_case(1)] fn test_basic() {}",
        "#[async_test] async fn test_basic() {}",
        "macro_rules! generated { () => { #[test] fn test_basic() {} }; }",
    ] {
        assert!(findings(RULE, source).is_empty(), "{source}");
    }
}

#[test]
fn test_recognizes_qualified_conditional_ignored_and_raw_test_names() {
    for attrs in [
        "#[tokio::test]",
        "#[async_std::test]",
        "#[ignore] #[test]",
        "#[cfg_attr(test, test)]",
        "#[cfg_attr(test, cfg_attr(unix, tokio::test))]",
        "#[test] #[cfg_attr(unix, test)]",
    ] {
        let source = format!("mod tests {{ {attrs} async fn r#test_basic() {{}} }}");
        assert_eq!(findings(RULE, &source).len(), 1, "{source}");
    }
}
