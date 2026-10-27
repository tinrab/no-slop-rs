//! Constructor naming follows explicit outer return syntax, not inferred types.

use std::path::Path;

use crate::no_slop::{config::Config, diagnostic::Report, fix};

use super::{findings, scan_source};

const RULE: &str = "require-create-for-fallible-constructors";

#[test]
fn test_rejects_explicit_fallible_new_contracts() {
    for signature in [
        "fn new() -> Result<Self, Failure>",
        "pub fn new(input: u8) -> Option<Self>",
        "async fn new() -> anyhow::Result<Self>",
        "const fn new() -> core::option::Option<Self>",
        "unsafe fn new() -> std::result::Result<Self, Failure>",
        "fn new() -> crate::Result<Self>",
        "fn new() -> crate::Option<Self>",
        "fn r#new() -> (Option<Self>)",
        "fn new() -> Result",
        "fn new() -> Result<Other, Failure>",
    ] {
        let source = format!("impl Connection {{ {signature} {{ todo!() }} }}");
        let diagnostics = findings(RULE, &source);
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert!(diagnostics[0].fix.is_none());
        assert!(diagnostics[0].help.contains("create"));
        assert_eq!(
            &source[diagnostics[0].start.byte..diagnostics[0].end.byte],
            if signature.contains("r#new") {
                "r#new"
            } else {
                "new"
            }
        );
    }
}

#[test]
fn test_accepts_infallible_alternate_names_receivers_and_hidden_aliases() {
    for signature in [
        "fn new() -> Self",
        "fn new() -> Connection",
        "fn new()",
        "fn create() -> Result<Self, Failure>",
        "fn create_with_config() -> Option<Self>",
        "fn new_with_config() -> Result<Self, Failure>",
        "fn try_new() -> Option<Self>",
        "fn try_new_with_config() -> Option<Self>",
        "fn new(&self) -> Result<Self, Failure>",
        "fn new(self: Box<Self>) -> Option<Self>",
        "fn new() -> ConnectionResult<Self>",
        "fn new() -> Box<Result<Self, Failure>>",
        "fn new() -> impl Future<Output = Result<Self, Failure>>",
        "fn new() -> <Factory as Builder>::Result",
        "fn new() -> &Option<Self>",
    ] {
        let source = format!("impl Connection {{ {signature} {{ todo!() }} }}");
        assert!(findings(RULE, &source).is_empty(), "{source}");
    }
    assert!(findings(RULE, "fn new() -> Option<Connection> { None }").is_empty());
}

#[test]
fn test_checks_trait_declarations_but_preserves_trait_implementation_contracts() {
    let source = "trait Factory { fn new() -> Option<Self>; }\nimpl Factory for Connection { fn new() -> Option<Self> { None } }";
    assert_eq!(findings(RULE, source).len(), 1);
    assert!(findings(RULE, "impl external::Factory for Connection { fn new() -> Result<Self, Failure> { todo!() } }").is_empty());
    assert_eq!(
        findings(
            RULE,
            "trait Factory { async fn new() -> Option<Self> { None } }"
        )
        .len(),
        1
    );
    assert!(findings(RULE, "trait Factory { fn new(&self) -> Option<Self>; }").is_empty());
}

#[test]
fn test_checks_nested_inactive_and_test_code_without_entering_macros() {
    let source = "#[cfg(any())] mod disabled { impl Connection { fn new() -> Option<Self> { None } } }\n#[cfg(test)] mod tests { impl Connection { fn new() -> Option<Self> { None } } }\nmacro_rules! generated { () => { impl Connection { fn new() -> Option<Self> { None } } }; }";
    assert_eq!(findings(RULE, source).len(), 2);
    let source = "impl external::Factory for Connection { fn run() { struct Local; impl Local { fn new() -> Option<Self> { None } } } }";
    assert_eq!(findings(RULE, source).len(), 1);
}

#[test]
fn test_levels_local_exceptions_and_no_automatic_renames() {
    let source = "impl Connection { fn new() -> Option<Self> { None } }";
    assert_eq!(
        fix::plan(Path::new("src/lib.rs"), source, false, &Config::default()).unwrap(),
        source
    );
    for (level, count, status) in [("error", 1, 1), ("warn", 1, 0), ("off", 0, 0)] {
        let config = Config::parse(&serde_json::json!({"no-slop":{"rules":{RULE:level}}})).unwrap();
        let mut report = Report::default();
        scan_source(Path::new("src/lib.rs"), source, true, &config, &mut report);
        assert_eq!(report.diagnostics.len(), count);
        assert_eq!(report.exit_status(), status);
    }
    for source in [
        "// no-slop-allow: require-create-for-fallible-constructors -- preserve the published API\nimpl Connection { fn new() -> Option<Self> { None } }",
        "impl Connection {\n// no-slop-allow: require-create-for-fallible-constructors -- preserve the published API\nfn new() -> Option<Self> { None }\n}",
    ] {
        let mut report = Report::default();
        scan_source(
            Path::new("src/lib.rs"),
            source,
            false,
            &Config::default(),
            &mut report,
        );
        assert!(report.failures.is_empty());
        assert!(report.diagnostics.is_empty(), "{report:?}");
        assert_eq!(report.summary.suppressed, 1);
    }
}
