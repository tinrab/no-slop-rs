use super::findings;

#[test]
fn test_reports_adjacent_functions_types_aliases_and_impls() {
    for source in [
        "fn first() {}\nfn second() {}",
        "struct First;\nstruct Second;",
        "type First = u8;\ntype Second = u16;",
        "struct First;\nimpl First {}",
        "enum First {} enum Second {}",
    ] {
        assert_eq!(
            findings("require-readable-spacing", source).len(),
            1,
            "{source}"
        );
    }
}

#[test]
fn test_checks_module_associated_foreign_and_block_items() {
    for source in [
        "mod nested { fn first() {}\nfn second() {} }",
        "impl First { fn first() {}\nfn second() {} }",
        "trait First { fn first();\nfn second(); }",
        "unsafe extern \"C\" { fn first();\nfn second(); }",
        "fn outer() { struct First;\nstruct Second; }",
    ] {
        assert_eq!(
            findings("require-readable-spacing", source).len(),
            1,
            "{source}"
        );
    }
}

#[test]
fn test_preserves_import_module_and_single_line_constant_groups() {
    for source in [
        "use std::fs;\nuse std::io;",
        "use std::{\nfs,\nio,\n};\nuse std::path::Path;",
        "extern crate first;\nextern crate second;",
        "mod first;\n#[cfg(test)] mod second;",
        "const FIRST: u8 = 1;\nstatic SECOND: u8 = 2;",
        "impl First { const FIRST: u8 = 1;\nconst SECOND: u8 = 2; }",
        "trait First { const FIRST: u8;\nconst SECOND: u8; }",
        "unsafe extern \"C\" { static FIRST: u8;\nstatic SECOND: u8; }",
    ] {
        assert!(
            findings("require-readable-spacing", source).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn test_distinct_groups_and_multiline_constants_need_spacing() {
    for source in [
        "use std::io;\nmod second;",
        "mod first;\nconst SECOND: u8 = 2;",
        "const FIRST: [u8; 1] = [\n1,\n];\nconst SECOND: u8 = 2;",
        "const FIRST: u8 = 1;\nconst SECOND: [u8; 1] = [\n2,\n];",
    ] {
        assert_eq!(
            findings("require-readable-spacing", source).len(),
            1,
            "{source}"
        );
    }
}

#[test]
fn test_documentation_and_attributes_remain_attached_to_the_next_declaration() {
    let source = "fn first() {}\n/// Documents second.\n#[inline]\nfn second() {}";
    let result = findings("require-readable-spacing", source);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start.line, 2);
    for source in [
        "fn first() {}\n\n/// Documents second.\n#[inline]\nfn second() {}",
        "fn first() {}\n\n#[inline]\nfn second() {}",
        "const FIRST: u8 = 1;\n\n/// Documents second.\nconst SECOND: u8 = 2;",
    ] {
        assert!(
            findings("require-readable-spacing", source).is_empty(),
            "{source}"
        );
    }
    assert_eq!(
        findings(
            "require-readable-spacing",
            "const FIRST: u8 = 1;\n/// Documents second.\nconst SECOND: u8 = 2;"
        )
        .len(),
        1
    );
    assert_eq!(
        findings(
            "require-readable-spacing",
            "fn first() {}\n/// Documents second.\n\nfn second() {}"
        )
        .len(),
        1
    );
}

#[test]
fn test_requires_spacing_before_leading_comments_and_accepts_trailing_comments() {
    for source in [
        "fn first() {}\n// About second.\nfn second() {}",
        "const FIRST: u8 = 1;\n// About second.\nconst SECOND: u8 = 2;",
        "fn first() {}\n/* About second. */\n\nfn second() {}",
        "fn first() {} /* trailing\n\ncomment */\nfn second() {}",
    ] {
        assert_eq!(
            findings("require-readable-spacing", source).len(),
            1,
            "{source}"
        );
    }
    for source in [
        "fn first() {} // trailing\n\n// About second.\nfn second() {}",
        "const FIRST: u8 = 1; // trailing\nconst SECOND: u8 = 2;",
        "fn first() {} /* trailing\ncomment */\n\nfn second() {}",
    ] {
        assert!(
            findings("require-readable-spacing", source).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn test_accepts_whitespace_only_blank_lines_crlf_and_extra_spacing() {
    for gap in ["\n\n", "\r\n \t\r\n", "\n\n\n"] {
        let source = format!("fn first() {{}}{gap}fn second() {{}}");
        assert!(
            findings("require-readable-spacing", &source).is_empty(),
            "{gap:?}"
        );
    }
    assert_eq!(
        findings(
            "require-readable-spacing",
            "fn first() {}\r\nfn second() {}"
        )
        .len(),
        1
    );
}

#[test]
fn test_ignores_statement_spacing_fields_variants_strings_and_macro_bodies() {
    for source in [
        "fn f() { let first = 1;\nlet second = 2;\nif first == second {}\nreturn; }",
        "struct First { first: u8,\nsecond: u8 }",
        "enum First { One,\nTwo }",
        "macro_rules! items { () => { fn first() {} fn second() {} }; }",
        "fn f() { custom! { fn first() {} fn second() {} } }",
        "fn f() { let s = \"fn first() {}\\nfn second() {}\"; }",
        "fn f() { struct First; let value = 1; struct Second; }",
    ] {
        assert!(
            findings("require-readable-spacing", source).is_empty(),
            "{source}"
        );
    }
}
