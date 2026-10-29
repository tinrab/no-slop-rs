use std::path::Path;

use syn::parse_file;

use crate::no_slop::{context::Context, diagnostic::Diagnostic, rules::RULES};

fn findings(path: &str, crate_root: bool, source: &str) -> Vec<Diagnostic> {
    let file = parse_file(source).unwrap();
    let mut context = Context::new(Path::new(path), source, &file, false);
    context.crate_root = crate_root;
    let rule = RULES
        .iter()
        .find(|rule| rule.id == "require-mod-rs-layout")
        .unwrap();
    (rule.check)(&file, &context)
}

#[test]
fn test_rejects_sibling_file_with_out_of_line_children_once() {
    let source = "mod first;\nmod second;";
    let result = findings("src/parser.rs", false, source);
    assert_eq!(result.len(), 1);
    assert_eq!(&source[result[0].start.byte..result[0].end.byte], "first");
}

#[test]
fn test_accepts_mod_files_crate_roots_and_inline_or_path_children() {
    for (path, crate_root, source) in [
        ("src/parser/mod.rs", false, "mod child;"),
        ("src/lib.rs", true, "mod child;"),
        ("src/bin/tool.rs", true, "mod child;"),
        ("src/parser.rs", false, "mod child {}"),
        ("src/parser.rs", false, "#[path = \"other.rs\"] mod child;"),
        ("src/parser.rs", false, "fn f() {}"),
    ] {
        assert!(
            findings(path, crate_root, source).is_empty(),
            "{path}: {source}"
        );
    }
}
