use std::path::Path;

use crate::no_slop::{config::Config, diagnostic::Report};

use super::{findings, scan_source};

#[test]
fn test_reports_owned_types_and_traits() {
    assert_eq!(
        findings(
            "no-vague-type-names",
            "struct UserManager; enum FileData {} trait ParseHelper {} type RowProcessor = (); "
        )
        .len(),
        4
    );
}

#[test]
fn test_leaves_imports_values_and_specific_names_alone() {
    assert!(
        findings(
            "no-vague-type-names",
            "use foreign::UserManager; struct Metadata; struct Database; fn f() { let data = 0; }"
        )
        .is_empty()
    );
}

#[test]
fn test_rejects_generic_module_declarations_outside_test_scopes() {
    for name in ["utils", "util", "helpers", "helper", "common", "misc"] {
        for source in [
            format!("mod {name};"),
            format!("pub mod {name} {{}}"),
            format!("mod outer {{ mod {name}; }}"),
        ] {
            let diagnostics = findings("no-vague-module-names", &source);
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert_eq!(
                &source[diagnostics[0].start.byte..diagnostics[0].end.byte],
                name
            );
        }
    }
    for source in [
        "mod parser;",
        "mod test_utils;",
        "mod utilities_for_parsing;",
        "#[cfg(test)] mod helpers;",
        "#[cfg(test)] mod tests { mod common; }",
        "struct utils;",
    ] {
        assert!(
            findings("no-vague-module-names", source).is_empty(),
            "{source}"
        );
    }
}

fn vague_module_findings(path: &str, test_file: bool) -> usize {
    let mut report = Report::default();
    scan_source(
        Path::new(path),
        "mod common;",
        test_file,
        &Config::default(),
        &mut report,
    );
    report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule == "no-vague-module-names")
        .count()
}

#[test]
fn test_auxiliary_targets_keep_a_shared_common_module() {
    for (path, test_file) in [
        ("tests/integration.rs", true),
        ("crates/app/tests/integration.rs", true),
        ("examples/demo.rs", false),
        ("crates/app/examples/nested/demo.rs", false),
        ("benches/throughput.rs", false),
    ] {
        assert_eq!(vague_module_findings(path, test_file), 0, "{path}");
    }
    for path in ["src/lib.rs", "crates/app/src/examples_page.rs"] {
        assert_eq!(vague_module_findings(path, false), 1, "{path}");
    }
}
