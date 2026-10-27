//! Literal error-message openings and the limits of syntax-only recognition.

use std::path::Path;

use crate::no_slop::{config::Config, diagnostic::Report, fix};

use super::{findings, scan_source};

const RULE: &str = "require-lowercase-error-messages";

#[test]
fn test_recognizes_common_error_macros_and_the_message_argument() {
    for expression in [
        "anyhow!(\"Failed to read\")",
        "anyhow::anyhow!(\"Failed: {path}\")",
        "eyre!(r#\"Failed to read\"#)",
        "eyre::eyre!(\"Failed: {}\", path)",
        "bail!(\"Failed to read\")",
        "anyhow::bail!(\"Failed\",)",
        "eyre::bail!(\"Failed\")",
        "color_eyre::eyre::bail!(\"Failed\")",
        "format_err!(\"Failed\")",
        "anyhow::format_err!(\"Failed\")",
        "ensure!(condition, \"Failed\")",
        "anyhow::ensure!(condition, \"Failed\",)",
        "eyre::ensure!(condition, \"Failed {}\", value)",
        "anyhow!(format!(\"Failed {}\", path))",
    ] {
        let source = format!("fn f() {{ {expression}; }}");
        let diagnostics = findings(RULE, &source);
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert!(diagnostics[0].fix.is_none());
    }
    for expression in [
        "ensure!(\"Failed\" == input, \"invalid input\")",
        "ensure!(condition)",
        "anyhow!(existing_error)",
        "bail!(concat!(\"Failed\", suffix))",
        "other::bail!(\"Failed\")",
        "anyhow!(\"failed {}\", \"Uppercase argument\")",
        "anyhow!()",
        "ensure!(condition,)",
        "anyhow!(not valid syntax)",
    ] {
        assert!(
            findings(RULE, &format!("fn f() {{ {expression}; }}")).is_empty(),
            "{expression}"
        );
    }
}

#[test]
fn test_context_result_and_io_error_messages() {
    for expression in [
        "value.context(\"Failed\")",
        "value.wrap_err(r\"Failed\")",
        "value.context(std::format!(\"Failed: {}\", path))",
        "value.with_context(|| format!(\"Failed: {path}\"))",
        "value.wrap_err_with(|| { (\"Failed\".to_owned()) })",
        "value.with_context((|| { { \"Failed\" } }))",
        "Err(\"Failed\")",
        "Result::Err(\"Failed\".into())",
        "std::result::Result::Err(\"Failed\".to_string())",
        "core::result::Result::Err(alloc::format!(\"Failed\"))",
        "value.ok_or(\"Failed\")",
        "value.ok_or_else(|| \"Failed\")",
        "std::io::Error::new(kind, \"Failed\")",
        "io::Error::other(\"Failed\")",
        "Error::new(kind, format!(\"Failed\"))",
        "Error::other(\"Failed\")",
    ] {
        let diagnostics = findings(RULE, &format!("fn f() {{ {expression}; }}"));
        assert_eq!(diagnostics.len(), 1, "{expression}");
    }
    for expression in [
        "value.context(message)",
        "value.context(\"failed HTTP request\")",
        "value.with_context(callback)",
        "value.with_context(|input| \"Failed\")",
        "value.with_context(async || \"Failed\")",
        "value.with_context(|| { let text = \"Failed\"; text })",
        "value.with_context(|| { return \"Failed\"; })",
        "IoError::other(\"Failed\")",
        "custom::Error::new(kind, \"Failed\")",
        "Error::new(\"Failed\")",
        "Error::new(\"Failed\", message)",
        "anyhow::Context::context(value, \"Failed\")",
        "Err(MESSAGE)",
        "value.ok_or_else(|| MESSAGE)",
    ] {
        assert!(
            findings(RULE, &format!("fn f() {{ {expression}; }}")).is_empty(),
            "{expression}"
        );
    }
}

#[test]
fn test_thiserror_struct_and_variant_attributes_without_expansion() {
    let source = r#"
        #[derive(thiserror::Error)]
        #[error("Failed to open {path}", path = .0)]
        struct OpenFailure(String);

        enum ReadFailure {
            #[error("Failed at {line}", line = count(.0))]
            Line(u32),
            #[error(transparent)]
            Source(Other),
            #[error("invalid HTTP response")]
            Response,
            #[error("{UppercaseField}")]
            Dynamic { UppercaseField: String },
            #[cfg_attr(feature = "detailed", error("Failed"))]
            Conditional,
        }

        #[error("Failed")]
        fn unrelated_attribute() {}
    "#;
    assert_eq!(findings(RULE, source).len(), 2);
}

#[test]
fn test_checks_the_initial_static_text_and_preserves_identifiers_later() {
    for literal in [
        r#""Failed HTTP request""#,
        r#""  'Failed'""#,
        r#""404: Failed""#,
        r#""\u{46}ailed""#,
        r#""Échec""#,
        r#""ǅamaged""#,
        r##"r#"Failed"#"##,
        r#""{{Failed}}""#,
        r#""Failed {context}""#,
        r#""HTTP request failed""#,
    ] {
        let source = format!("fn f() {{ bail!({literal}); }}");
        assert_eq!(findings(RULE, &source).len(), 1, "{source}");
    }
    for literal in [
        r#""failed HTTP request""#,
        r#""invalid UserId""#,
        r#""échec""#,
        r#""""#,
        r#""123 !!!""#,
        r#""读取失败""#,
        r#""{UppercaseName} Failed""#,
        r#""{} Failed""#,
        r#""{{}} {UppercaseName} Failed""#,
        r#""failed. Another sentence.""#,
    ] {
        let source = format!("fn f() {{ bail!({literal}); }}");
        assert!(findings(RULE, &source).is_empty(), "{source}");
    }
    // Braces in an unformatted context string are ordinary message text.
    assert_eq!(
        findings(RULE, "fn f() { value.context(\"{UppercaseName}\"); }").len(),
        1
    );
}

#[test]
fn test_does_not_scan_unrelated_strings_display_or_opaque_macro_payloads() {
    let source = r#"
        const MESSAGE: &str = "Failed";

        macro_rules! custom { () => { anyhow!("Failed"); } }

        impl std::fmt::Display for Failure {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Failed")
            }
        }

        fn f() {
            error!("Failed");
            tracing::error!("Failed");
            panic!("Failed");
            assert!(false, "Failed");
            value.expect("Failed");
            custom! { bail!("Failed"); }
            let text = "bail!(\"Failed\")";
            let formatted = format!("Failed");
            let error = anyhow!(MESSAGE);
        }
    "#;
    assert!(findings(RULE, source).is_empty());
}

#[test]
fn test_levels_exceptions_test_scopes_and_diagnostic_only_fixing() {
    let source = "fn f() { value.context(\"Failed to read configuration\"); }";
    let mut report = Report::default();
    scan_source(
        Path::new("src/lib.rs"),
        source,
        false,
        &Config::default(),
        &mut report,
    );
    assert_eq!(report.exit_status(), 1);
    assert_eq!(
        fix::plan(Path::new("src/lib.rs"), source, false, &Config::default()).unwrap(),
        source
    );
    for (level, count) in [("warn", 1), ("off", 0)] {
        let config = Config::parse(&serde_json::json!({"no-slop":{"rules":{RULE:level}}})).unwrap();
        let mut report = Report::default();
        scan_source(Path::new("src/lib.rs"), source, false, &config, &mut report);
        assert_eq!(report.diagnostics.len(), count);
        assert_eq!(report.exit_status(), 0);
    }
    for source in [
        "// no-slop-allow: require-lowercase-error-messages -- preserve the remote system's wording\nfn f() { bail!(\"Failed to read configuration\"); }",
        "fn f() {\n// no-slop-allow: require-lowercase-error-messages -- HTTP is the protocol name\nvalue.context(\"HTTP request failed\");\n}",
    ] {
        let mut report = Report::default();
        scan_source(
            Path::new("src/lib.rs"),
            source,
            false,
            &Config::default(),
            &mut report,
        );
        assert!(report.diagnostics.is_empty(), "{report:?}");
        assert!(report.failures.is_empty());
        assert_eq!(report.summary.suppressed, 1);
    }
    assert_eq!(
        findings(
            RULE,
            "#[cfg(test)] mod tests { #[test] fn test_error() { bail!(\"Failed\"); } }"
        )
        .len(),
        1
    );
}

#[test]
fn test_original_offsets_survive_headers_unicode_and_crlf() {
    let source =
        "\u{feff}#!/usr/bin/env rust-script\r\nfn café() { value.context(\"Échec\"); }\r\n";
    let mut report = Report::default();
    scan_source(
        Path::new("src/lib.rs"),
        source,
        false,
        &Config::default(),
        &mut report,
    );
    assert!(report.failures.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    let diagnostic = &report.diagnostics[0];
    assert_eq!(diagnostic.start.byte, source.find('"').unwrap());
    assert_eq!(diagnostic.start.line, 2);
    assert_eq!(
        &source[diagnostic.start.byte..diagnostic.end.byte],
        "\"Échec\""
    );
}
