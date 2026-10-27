//! Standard-error fields are recognized by written paths, not type inference.

use std::path::Path;

use crate::no_slop::{config::Config, diagnostic::Report};

use super::{findings, scan_source};

const RULE: &str = "require-error-sources";

#[test]
fn test_recognizes_each_supported_standard_error_spelling() {
    for ty in [
        "std::io::Error",
        "::std::io::Error",
        "io::Error",
        "std::num::ParseIntError",
        "core::num::ParseIntError",
        "num::ParseIntError",
        "ParseIntError",
        "std::num::ParseFloatError",
        "core::num::ParseFloatError",
        "num::ParseFloatError",
        "ParseFloatError",
        "std::str::Utf8Error",
        "core::str::Utf8Error",
        "str::Utf8Error",
        "Utf8Error",
        "std::string::FromUtf8Error",
        "alloc::string::FromUtf8Error",
        "string::FromUtf8Error",
        "FromUtf8Error",
        "(io::Error)",
    ] {
        for source in [
            format!(
                "#[derive(Debug, thiserror::Error)] #[error(\"reading config\")] struct ReadFailure({ty});"
            ),
            format!(
                "#[derive(Error)] enum ReadFailure {{ #[error(\"reading config\")] Read {{ cause: {ty} }} }}"
            ),
        ] {
            let diagnostics = findings(RULE, &source);
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert!(diagnostics[0].fix.is_none());
        }
    }
}

#[test]
fn test_existing_source_declarations_and_transparent_wrappers_are_accepted() {
    for source in [
        "#[derive(Error)] #[error(\"read config\")] struct Fault(#[source] io::Error);",
        "#[derive(Error)] #[error(\"read config\")] struct Fault(#[from] io::Error);",
        "#[derive(Error)] #[error(\"read config\")] struct Fault { source: io::Error }",
        "#[derive(Error)] #[error(\"read config\")] struct Fault { r#source: io::Error }",
        "#[derive(Error)] #[error(transparent)] struct Fault(io::Error);",
        "#[derive(Error)] enum Fault { #[error(transparent)] Read(io::Error) }",
        "#[derive(Error)] enum Fault { #[error(\"read config\")] Read(#[source] io::Error) }",
        "#[derive(Error)] enum Fault { #[error(\"read config\")] Read { source: io::Error } }",
        "#[derive(Error)] struct Fault { #[source] primary: CustomError, secondary: io::Error }",
    ] {
        assert!(findings(RULE, source).is_empty(), "{source}");
    }
}

#[test]
fn test_multiple_candidates_produce_one_finding_per_wrapper() {
    let source = "#[derive(Error)] enum Fault { Read(io::Error, ParseIntError), Other { first: io::Error, second: Utf8Error } }";
    let diagnostics = findings(RULE, source);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        &source[diagnostics[0].start.byte..diagnostics[0].end.byte],
        "Read"
    );
    assert_eq!(
        &source[diagnostics[1].start.byte..diagnostics[1].end.byte],
        "Other"
    );
}

#[test]
fn test_custom_alias_container_and_conditional_forms_stay_outside_coverage() {
    for ty in [
        "Error",
        "IoError",
        "CustomError",
        "serde_json::Error",
        "Box<io::Error>",
        "Option<io::Error>",
        "&'static io::Error",
        "<Factory as Trait>::Error",
    ] {
        let source = format!("#[derive(Error)] struct Fault({ty});");
        assert!(findings(RULE, &source).is_empty(), "{source}");
    }
    for source in [
        "struct Fault(io::Error);",
        "#[derive(Debug)] #[error(\"read config\")] struct Fault(io::Error);",
        "#[derive(errors::Error)] struct Fault(io::Error);",
        "#[cfg_attr(test, derive(Error))] struct Fault(io::Error);",
        "#[derive(Error)] #[cfg_attr(test, error(transparent))] struct Fault(io::Error);",
        "#[derive(Error)] struct Fault(#[cfg_attr(test, source)] io::Error);",
        "#[derive(Error)] enum Fault { #[cfg_attr(test, error(transparent))] Read(io::Error) }",
        "#[derive(Error)] #[cfg_attr(test, allow(dead_code))] enum Fault { Read(io::Error) }",
        "custom! { #[derive(Error)] struct Fault(io::Error); }",
    ] {
        assert!(findings(RULE, source).is_empty(), "{source}");
    }
    assert_eq!(
        findings(
            RULE,
            "#[cfg(any())] #[derive(Error)] struct Fault(io::Error);"
        )
        .len(),
        1
    );
}

#[test]
fn test_variant_exceptions_do_not_leak_to_siblings_or_other_rules() {
    let source = "#[derive(Error)] enum Fault {\n// no-slop-allow: require-error-sources -- preserve the published source chain\n#[error(\"read config\")] First(io::Error),\n#[error(\"read state\")] Second(io::Error),\n}";
    let mut report = Report::default();
    scan_source(
        Path::new("src/lib.rs"),
        source,
        false,
        &Config::default(),
        &mut report,
    );
    assert!(report.failures.is_empty(), "{report:?}");
    assert_eq!(report.summary.suppressed, 1);
    assert_eq!(report.diagnostics.len(), 1, "{report:?}");
    let diagnostic = &report.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start.byte..diagnostic.end.byte],
        "Second"
    );
    let source = "#[derive(Error)] enum Fault {\n// no-slop-allow: require-lowercase-error-messages -- quote a protocol's wording\n#[error(\"Read configuration\")] First(io::Error),\n}";
    let mut report = Report::default();
    scan_source(
        Path::new("src/lib.rs"),
        source,
        false,
        &Config::default(),
        &mut report,
    );
    assert!(report.failures.is_empty());
    assert_eq!(report.summary.suppressed, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].rule, RULE);
}
