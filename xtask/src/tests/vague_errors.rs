//! Exact-message matching shares coverage with error-message casing.

use super::findings;

const RULE: &str = "no-vague-error-messages";

#[test]
fn test_rejects_only_complete_normalized_placeholder_messages() {
    for message in [
        "error",
        "failed",
        "failure",
        "error occurred",
        "an error occurred",
        "something went wrong",
        "operation failed",
        "unknown error",
        "unexpected error",
        "an unexpected error occurred",
        "  An   ERROR\nOccurred! : …  ",
    ] {
        let source = format!("fn f() {{ value.context({message:?}); }}");
        let diagnostics = findings(RULE, &source);
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert!(diagnostics[0].fix.is_none());
    }
    for message in [
        "",
        "unknown configuration key",
        "operation failed: configuration missing",
        "failed to read configuration",
        "operation failed: {path}",
        "operation failed {}",
        "{error}",
        "{{operation failed}}",
        "[operation failed]",
        "错误",
        "error reading config",
    ] {
        let source = format!("fn f() {{ bail!({message:?}); }}");
        assert!(findings(RULE, &source).is_empty(), "{source}");
    }
}

#[test]
fn test_recognizes_shared_macro_context_conversion_and_error_sites() {
    for expression in [
        "anyhow!(\"operation failed\")",
        "eyre::eyre!(r\"unknown error\")",
        "color_eyre::eyre::bail!(\"operation failed\")",
        "ensure!(condition, \"failed\")",
        "value.wrap_err(\"failure\".to_owned())",
        "value.with_context(|| { (\"operation failed\") })",
        "value.wrap_err_with(|| \"operation failed\".into())",
        "Err(\"unknown error\".to_string())",
        "Result::Err(format!(\"operation failed\"))",
        "value.ok_or(\"error\")",
        "value.ok_or_else(|| \"error\")",
        "std::io::Error::new(kind, \"failure\")",
        "Error::other(\"unknown error\")",
    ] {
        assert_eq!(
            findings(RULE, &format!("fn f() {{ {expression}; }}")).len(),
            1,
            "{expression}"
        );
    }
    let source = "#[error(\"unknown error\")] struct Failure;\nenum Fault { #[error(\"operation failed\")] Operation }";
    assert_eq!(findings(RULE, source).len(), 2);
}

#[test]
fn test_dynamic_messages_unrelated_apis_and_macro_payloads_are_opaque() {
    let source = r#"
        #[error(transparent)] struct Failure(Inner);

        macro_rules! generated { () => { bail!("error"); } }

        fn f() {
            value.context(MESSAGE);
            value.with_context(|| { let message = "error"; message });
            value.with_context(callback);
            value.context(concat!("operation", " failed"));
            log::error!("error");
            panic!("error");
            custom! { bail!("error"); }
            let text = "error";
        }
    "#;
    assert!(findings(RULE, source).is_empty());
}
