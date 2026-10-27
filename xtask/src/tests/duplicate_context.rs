//! Only adjacent literal contexts are compared; formatting and dataflow are opaque.

use super::findings;

const RULE: &str = "no-duplicate-error-context";

#[test]
fn test_direct_lazy_mixed_and_wrapped_literal_pairs() {
    for chain in [
        "read().context(\"reading config\").context(\"reading config\")",
        "read().wrap_err(r\"reading config\").context(\"reading config\")",
        "read().with_context(|| \"reading config\").wrap_err_with(|| { \"reading config\" })",
        "read().context(\"reading config\".to_owned()).with_context((|| (\"reading config\".into())))",
        "(read().context(\"reading config\")).wrap_err(\"reading config\")",
        "{ read().context(\"reading config\") }.context(\"reading config\")",
        "read().context(\"read\\u{69}ng config\").context(r\"reading config\")",
    ] {
        let source = format!("fn f() {{ {chain}; }}");
        let diagnostics = findings(RULE, &source);
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert!(diagnostics[0].fix.is_none());
        assert!(matches!(
            &source[diagnostics[0].start.byte..diagnostics[0].end.byte],
            "context" | "with_context" | "wrap_err" | "wrap_err_with"
        ));
    }
}

#[test]
fn test_one_diagnostic_per_adjacent_duplicate_pair() {
    let source = "fn f() { value.context(\"reading config\").context(\"reading config\").context(\"reading config\"); }";
    assert_eq!(findings(RULE, source).len(), 2);
}

#[test]
fn test_distinct_dynamic_formatted_and_interrupted_contexts_are_accepted() {
    for chain in [
        "value.context(\"loading config\").context(\"reading config\")",
        "value.context(\"reading config\").context(\"Reading config\")",
        "value.context(\" reading config\").context(\"reading config\")",
        "value.context(\"reading config\").map(convert).context(\"reading config\")",
        "value.context(\"reading config\").await.context(\"reading config\")",
        "value.context(MESSAGE).context(MESSAGE)",
        "value.context(format!(\"reading config\")).context(\"reading config\")",
        "value.context(\"reading config\").with_context(|| format!(\"reading config\"))",
        "value.with_context(|| { side_effect(); \"reading config\" }).context(\"reading config\")",
        "value.with_context(async || \"reading config\").context(\"reading config\")",
        "value.with_context(|arg| \"reading config\").context(\"reading config\")",
        "value.ok_or(\"reading config\").context(\"reading config\")",
    ] {
        assert!(
            findings(RULE, &format!("async fn f() {{ {chain}; }}")).is_empty(),
            "{chain}"
        );
    }
    assert!(findings(RULE, "fn f() { let first = value.context(\"reading config\"); first.context(\"reading config\"); }").is_empty());
    assert!(
        findings(
            RULE,
            "custom! { value.context(\"reading config\").context(\"reading config\"); }"
        )
        .is_empty()
    );
}
