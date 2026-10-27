//! Cross-rule behavior for policy levels, exceptions, offsets, and edit eligibility.

use std::path::Path;

use crate::no_slop::{config::Config, diagnostic::Report, fix};

use super::scan_source;

fn cases() -> [(&'static str, &'static str); 4] {
    [
        (
            "no-vague-error-messages",
            "fn f() { value.context(\"operation failed\"); }",
        ),
        (
            "no-duplicate-error-context",
            "fn f() { value.context(\"reading config\").with_context(|| \"reading config\"); }",
        ),
        (
            "require-error-sources",
            "#[derive(Error)] #[error(\"reading config\")] struct ReadFailure(io::Error);",
        ),
        ("no-vague-test-names", "#[test] fn test_basic() {}"),
    ]
}

#[test]
fn test_new_policies_honor_levels_exceptions_and_never_supply_edits() {
    for (id, source) in cases() {
        for (level, count, status) in [("error", 1, 1), ("warn", 1, 0), ("off", 0, 0)] {
            let config =
                Config::parse(&serde_json::json!({"no-slop":{"rules":{id:level}}})).unwrap();
            let mut report = Report::default();
            scan_source(Path::new("src/lib.rs"), source, true, &config, &mut report);
            assert!(report.failures.is_empty(), "{id}: {report:?}");
            assert_eq!(report.diagnostics.len(), count, "{id}: {report:?}");
            assert_eq!(report.exit_status(), status, "{id}");
            assert!(
                report
                    .diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.fix.is_none())
            );
            assert_eq!(
                fix::plan(Path::new("src/lib.rs"), source, true, &config).unwrap(),
                source
            );
        }
        let source =
            format!("// no-slop-allow: {id} -- preserve the external protocol fixture\n{source}");
        let mut report = Report::default();
        scan_source(
            Path::new("src/lib.rs"),
            &source,
            false,
            &Config::default(),
            &mut report,
        );
        assert!(report.failures.is_empty(), "{id}: {report:?}");
        assert!(report.diagnostics.is_empty(), "{id}: {report:?}");
        assert_eq!(report.summary.suppressed, 1, "{id}");
    }
}

#[test]
fn test_policy_diagnostics_keep_original_offsets_and_test_coverage() {
    for (id, body) in cases() {
        let source = format!("\u{feff}#!/usr/bin/env rust-script\r\n// café\r\n{body}\r\n");
        let mut report = Report::default();
        scan_source(
            Path::new("src/lib.rs"),
            &source,
            true,
            &Config::default(),
            &mut report,
        );
        assert!(report.failures.is_empty(), "{id}: {report:?}");
        assert_eq!(report.diagnostics.len(), 1, "{id}: {report:?}");
        let diagnostic = &report.diagnostics[0];
        let expected = match id {
            "no-vague-error-messages" => "\"operation failed\"",
            "no-duplicate-error-context" => "with_context",
            "require-error-sources" => "ReadFailure",
            "no-vague-test-names" => "test_basic",
            _ => unreachable!(),
        };
        assert_eq!(
            &source[diagnostic.start.byte..diagnostic.end.byte],
            expected
        );
        assert_eq!(diagnostic.start.line, 3);
    }
}

#[test]
fn test_error_message_policies_can_report_independently_at_the_same_literal() {
    let source = "fn f() { value.context(\"Operation failed\"); }";
    let mut report = Report::default();
    scan_source(
        Path::new("src/lib.rs"),
        source,
        false,
        &Config::default(),
        &mut report,
    );
    assert_eq!(report.diagnostics.len(), 2);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule == "require-lowercase-error-messages")
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule == "no-vague-error-messages")
    );
    assert_eq!(
        report.diagnostics[0].start.byte,
        report.diagnostics[1].start.byte
    );
}
