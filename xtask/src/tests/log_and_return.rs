//! Logging a failure directly before returning it from the same block.

use super::findings;

const RULE: &str = "no-log-and-return";

#[test]
fn test_reports_recognized_logs_followed_by_returned_failures() {
    for body in [
        "error!(\"read failed\"); return Err(failure);",
        "warn!(\"read failed\"); return Err(failure)",
        "log::error!(\"read failed\"); Err(failure)",
        "tracing::error!(%failure, \"read failed\"); Err(failure.into())",
        "tracing::warn!(\"read failed\"); return Err(Failure::Read);",
        "eprintln!(\"read failed: {failure}\"); return Err(failure);",
        "std::eprintln!(\"read failed\"); Result::Err(failure)",
        "error!(\"read failed\"); bail!(\"read failed\");",
        "error!(\"read failed\"); anyhow::bail!(\"read failed\")",
        "error!(\"read failed\"); return std::result::Result::Err(failure);",
    ] {
        let source = format!("fn f() {{ {body} }}");
        let diagnostics = findings(RULE, &source);
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert!(diagnostics[0].fix.is_none());
    }
}

#[test]
fn test_reports_nested_blocks_and_match_arms_at_the_log() {
    let source = r#"
        fn f() -> Result<(), Failure> {
            match read() {
                Ok(value) => Ok(value),
                Err(failure) => {
                    tracing::error!("read failed");
                    Err(failure)
                }
            }
        }
    "#;
    let diagnostics = findings(RULE, source);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        &source[diagnostics[0].start.byte..diagnostics[0].end.byte],
        "tracing::error"
    );
}

#[test]
fn test_accepts_handled_failures_and_other_statements() {
    for body in [
        "error!(\"read failed\"); return Ok(fallback);",
        "error!(\"read failed\"); fallback()",
        "error!(\"read failed\"); Err(failure);",
        "error!(\"read failed\"); retry(); return Err(failure);",
        "info!(\"read failed\"); return Err(failure);",
        "println!(\"read failed\"); return Err(failure);",
        "custom::error!(\"read failed\"); return Err(failure);",
        "return Err(failure); error!(\"unreachable\");",
        "error!(\"read failed\"); other::bail!(\"read failed\");",
        "error!(\"read failed\"); return;",
        "custom! { error!(\"read failed\"); return Err(failure); }",
    ] {
        let source = format!("fn f() {{ {body} }}");
        assert!(findings(RULE, &source).is_empty(), "{source}");
    }
}

#[test]
fn test_test_scopes_are_exempt() {
    assert!(
        findings(
            RULE,
            "#[cfg(test)] mod tests { fn f() { error!(\"read failed\"); return Err(failure); } }"
        )
        .is_empty()
    );
}
