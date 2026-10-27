//! Grouping and import-equivalence safety checks use written syntax only.

use std::path::Path;

use super::findings;
use crate::no_slop::{config::Config, fix};

const RULE: &str = "require-grouped-imports";

fn fixed(source: &str) -> String {
    let result = fix::plan(Path::new("src/lib.rs"), source, false, &Config::default()).unwrap();
    assert!(findings_prepared(&result).is_empty(), "{result}");
    assert_eq!(
        fix::plan(Path::new("src/lib.rs"), &result, false, &Config::default()).unwrap(),
        result
    );
    result
}

fn findings_prepared(source: &str) -> Vec<crate::no_slop::diagnostic::Diagnostic> {
    let mut report = crate::no_slop::diagnostic::Report::default();
    super::scan_source(
        Path::new("src/lib.rs"),
        source,
        false,
        &Config::default(),
        &mut report,
    );
    report
        .diagnostics
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .collect()
}

#[test]
fn test_groups_shared_roots_and_nested_prefixes() {
    let source = "use std::env;\nuse std::io::{self, BufWriter, Write};\nuse std::path::PathBuf;\nuse std::sync::Arc;\nuse std::sync::Mutex;\nuse std::sync::atomic::{AtomicBool, Ordering};\n\nuse crate::error::{Error, Result};\nuse crate::frontend::error::Error as FrontendError;";
    assert_eq!(findings(RULE, source).len(), 1);
    let result = fixed(source);
    assert_eq!(
        result,
        "use std::{env, io::{self, BufWriter, Write}, path::PathBuf, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}};\n\nuse crate::{error::{Error, Result}, frontend::error::Error as FrontendError};"
    );
    for source in [
        "use a::X;\n\nuse a::Y;",
        "use a::{b::X,b::Y};",
        "use {a::X, b::Y, a::Z};",
    ] {
        assert_eq!(findings(RULE, source).len(), 1, "{source}");
        fixed(source);
    }
}

#[test]
fn test_accepts_layout_and_sibling_order_without_sorting_items() {
    for source in [
        "use std::{path::PathBuf, env};\n\nuse z::Z;\nuse a::A;\n\nuse crate::X;",
        "use std::{\n    path::PathBuf,\n    io::{Write, Read},\n};",
        "use a::{X};",
        "use a::{};\nuse a::X;",
        "use a::{b::{}, X};",
        "use a::X;\npub use a::Y;",
        "use ::a::X;\nuse a::Y;",
    ] {
        assert!(findings(RULE, source).is_empty(), "{source}");
    }
}

#[test]
fn test_preserves_aliases_globs_raw_names_visibility_and_duplicates() {
    for source in [
        "use ::a::r#type as _; use ::a::b::*; use ::a::b::C as D;",
        "pub(crate) use a::X; pub(crate) use a::Y;",
        "use a::b; use a::b::C;",
        "use a::b as B; use a::b::C;",
        "use a::X as _; use a::X as _; use a::Y;",
        "use a as A; use a as B; use a::X;",
        "use a::X; use a::X; use a::Y;",
        "use core::fmt; use alloc::vec; use std::env; use external::X; use self::X; use super::Y;",
    ] {
        let result = fixed(source);
        fix::validate(source, &result).unwrap();
    }
}

#[test]
fn test_comments_attributes_and_local_exceptions_are_boundaries() {
    for source in [
        "// no-slop-allow: require-grouped-imports -- separate platform imports\nuse a::X;\nuse a::Y;",
        "use a::X; // exported name\nuse a::Y;",
        "use a::X;\n// About Y.\nuse a::Y;\nuse a::Z;",
        "use a::{/* keep */ X};\nuse a::Y;",
        "#[cfg(unix)] use a::X;\nuse a::Y;",
        "/// About X.\nuse a::X;\nuse a::Y;",
        "use a::X;\n#[allow(unused_imports)] use a::Y;\nuse a::Z;",
        "use a::X;\nlet_invalid_macro! { use a::Y; use a::Z; }",
    ] {
        assert!(findings(RULE, source).is_empty(), "{source}");
    }
    assert_eq!(
        findings(RULE, "//! Module docs.\nuse a::X;\nuse a::Y;").len(),
        1
    );
}

#[test]
fn test_checks_each_lexical_scope_and_leaves_macro_bodies_opaque() {
    let source = "use a::X; use a::Y;\nmod nested { use b::X; use b::Y; }\n#[cfg(test)] mod tests { use super::X; use super::Y; }\nfn f() { use c::X; use c::Y; work(); use c::Z; }\nmacro_rules! imports { () => { use d::X; use d::Y; }; }";
    assert_eq!(findings(RULE, source).len(), 4);
    fixed(source);
}

#[test]
fn test_import_validator_rejects_changes_beyond_grouping() {
    let original = "use a::X as Y; use a::Z;\nfn f() { use b::X; work(); }";
    fix::validate(
        original,
        "use a::{Z, X as Y};\nfn f() { use b::X; work(); }",
    )
    .unwrap();
    for candidate in [
        "use a::{X as Q,Z}; fn f() { use b::X; work(); }",
        "use a::{X as Y,Q}; fn f() { use b::X; work(); }",
        "pub use a::{X as Y,Z}; fn f() { use b::X; work(); }",
        "use ::a::{X as Y,Z}; fn f() { use b::X; work(); }",
        "#[cfg(unix)] use a::{X as Y,Z}; fn f() { use b::X; work(); }",
        "use a::{X as Y,Z}; use b::X; fn f() { work(); }",
        "use a::{X as Y,Z}; fn f() { use b::X; other(); }",
    ] {
        assert!(fix::validate(original, candidate).is_err(), "{candidate}");
    }
    assert!(fix::validate("use a::{X,X};", "use a::X;").is_err());
    assert!(fix::validate("use a::{};", "use b::{};").is_err());
    assert!(fix::validate("use a::X; /// docs\nuse a::Y;", "use a::{X,Y};").is_err());
}

#[test]
fn test_header_unicode_crlf_and_comment_cleanup_interaction() {
    let source =
        "\u{feff}#!/usr/bin/env rust-script\r\nuse café::X;\r\n// Main logic\r\nuse café::Y;\r\n";
    assert_eq!(
        fixed(source),
        "\u{feff}#!/usr/bin/env rust-script\r\nuse café::{X, Y};\r\n"
    );
    let source = "fn f() {\r\n    use a::X;\r\n    use a::Y;\r\n}";
    assert_eq!(fixed(source), "fn f() {\r\n    use a::{X, Y};\r\n}");
}

#[test]
fn test_category_order_and_blank_lines_in_indented_blocks() {
    for source in [
        "use crate::X;\nuse external::Y;\nuse std::env;",
        "use std::env;\nuse external::Y;\n\nuse crate::X;",
        "fn f() {\n    use std::env;\n    use crate::X;\n}",
    ] {
        assert_eq!(findings(RULE, source).len(), 1, "{source}");
        fixed(source);
    }
    assert!(findings(RULE, "fn f() {\n    use std::env;\n\n    use crate::X;\n}").is_empty());
}

#[test]
fn test_levels_and_enclosing_exceptions_control_grouping_fixes() {
    let source = "use a::X;\nuse a::Y;";
    let config = Config::parse(&serde_json::json!({"no-slop":{"rules":{RULE:"off"}}})).unwrap();
    assert_eq!(
        fix::plan(Path::new("src/lib.rs"), source, false, &config).unwrap(),
        source
    );
    let config = Config::parse(&serde_json::json!({"no-slop":{"rules":{RULE:"warn"}}})).unwrap();
    assert_eq!(
        fix::plan(Path::new("src/lib.rs"), source, false, &config).unwrap(),
        "use a::{X, Y};"
    );
    let source = "// no-slop-allow: require-grouped-imports -- mirror generated protocol modules\nmod protocol { use a::X; use a::Y; }";
    assert_eq!(
        fix::plan(Path::new("src/lib.rs"), source, false, &Config::default()).unwrap(),
        source
    );
}

#[test]
fn test_plain_imports_do_not_become_type_only_self_imports() {
    let source = "use a::b; use a::b::X;";
    assert_eq!(fixed(source), "use a::{b, b::X};");
    assert!(fix::validate(source, "use a::b::{self, X};").is_err());
    assert!(fix::validate("use a::b::self;", "use a::b;").is_err());
    assert_eq!(
        fixed("use a::b::{self}; use a::b::X;"),
        "use a::b::{self, X};"
    );
}
