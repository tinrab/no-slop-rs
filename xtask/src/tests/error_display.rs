//! Printing a whole source field, without judging the surrounding message text.

use std::path::Path;

use crate::no_slop::{config::Config, diagnostic::Report};

use super::{findings, scan_source};

const RULE: &str = "no-source-in-error-display";

fn variant(body: &str) -> String {
    format!("#[derive(Debug, thiserror::Error)] enum Failure {{ {body} }}")
}

#[test]
fn test_reports_every_way_of_printing_the_whole_source() {
    for body in [
        r#"#[error("read failed: {0}")] Read(#[from] io::Error)"#,
        r#"#[error("read failed: {0:?}")] Read(#[source] io::Error)"#,
        r#"#[error("{0}")] Read(#[from] io::Error)"#,
        r#"#[error("read failed: {1}")] Read(String, #[source] io::Error)"#,
        r#"#[error("read failed: {source}")] Read { source: io::Error }"#,
        r#"#[error("read failed: {cause}")] Read { #[source] cause: io::Error }"#,
        r#"#[error("read failed: {}", .source)] Read { source: io::Error }"#,
        r#"#[error("read failed: {}", source)] Read { source: io::Error }"#,
        r#"#[error("read failed: {}", .0)] Read(#[from] io::Error)"#,
        r#"#[error("read failed: {}", _0)] Read(#[from] io::Error)"#,
        r#"#[error("{} failed: {}", .path, .source)] Read { path: String, source: io::Error }"#,
        r#"#[error("{path}: {why}", why = .source)] Read { path: String, source: io::Error }"#,
    ] {
        let source = variant(body);
        let diagnostics = findings(RULE, &source);
        assert_eq!(diagnostics.len(), 1, "{body}");
        assert!(diagnostics[0].fix.is_none(), "{body}");
    }
}

#[test]
fn test_accepts_message_text_other_fields_and_derived_details() {
    for body in [
        r#"#[error("read failed")] Read(#[from] io::Error)"#,
        r#"#[error("read failed: {{0}}")] Read(#[from] io::Error)"#,
        r#"#[error("reading {path} failed")] Read { path: String, source: io::Error }"#,
        r#"#[error("failed: {0}")] Read(String, #[source] io::Error)"#,
        r#"#[error("failed: {}", source.kind())] Read { source: io::Error }"#,
        r#"#[error("failed: {}", .0.kind())] Read(#[from] io::Error)"#,
        r#"#[error("failed: {source}", source = .path)] Read { path: String, source: io::Error }"#,
        r#"#[error("failed: {}", _source)] Read { _source: String, source: io::Error }"#,
        r#"#[error(transparent)] Read(#[from] io::Error)"#,
        r#"#[error("failed: {0}")] Read(String)"#,
        r#"#[error("failed: {reason}")] Read { reason: String }"#,
        r#"#[cfg_attr(feature = "x", error("failed: {0}"))] Read(#[from] io::Error)"#,
    ] {
        let source = variant(body);
        assert!(findings(RULE, &source).is_empty(), "{body}");
    }
}

#[test]
fn test_reports_structs_and_requires_an_explicit_derive() {
    let reported = r#"
        #[derive(thiserror::Error)]
        #[error("read failed: {0}")]
        struct ReadFailure(#[from] io::Error);

        #[derive(Error)]
        #[error("read failed: {source}")]
        struct OpenFailure { source: io::Error }
    "#;
    assert_eq!(findings(RULE, reported).len(), 2);
    for ignored in [
        r#"#[error("read failed: {0}")] struct ReadFailure(#[source] io::Error);"#,
        r#"#[cfg_attr(test, derive(Error))] #[error("{0}")] struct ReadFailure(#[source] io::Error);"#,
    ] {
        assert!(findings(RULE, ignored).is_empty(), "{ignored}");
    }
}

#[test]
fn test_diagnostic_targets_the_message_and_honors_exceptions() {
    let source = variant(r#"#[error("read failed: {0}")] Read(#[from] io::Error)"#);
    let diagnostic = &findings(RULE, &source)[0];
    assert_eq!(
        &source[diagnostic.start.byte..diagnostic.end.byte],
        r#""read failed: {0}""#
    );
    assert!(diagnostic.message.contains("field `0`"));

    let source = variant(
        "// no-slop-allow: no-source-in-error-display -- the API response sends only this string\n#[error(\"read failed: {0}\")] Read(#[from] io::Error)",
    );
    let mut report = Report::default();
    scan_source(
        Path::new("src/lib.rs"),
        &source,
        false,
        &Config::default(),
        &mut report,
    );
    assert!(report.failures.is_empty(), "{report:?}");
    assert!(report.diagnostics.iter().all(|d| d.rule != RULE));
    assert_eq!(report.summary.suppressed, 1);
}
