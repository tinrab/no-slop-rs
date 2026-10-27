//! Fix planning and file replacement, independent of CLI output.

use std::{fs, path::Path};

use tempfile;

use crate::no_slop::{config::Config, diagnostic::TextEdit, fix};

fn fixed(source: &str) -> String {
    fix::plan(Path::new("src/lib.rs"), source, false, &Config::default()).unwrap()
}

#[test]
fn test_spacing_preserves_comments_attributes_and_compact_groups() {
    for (before, after) in [
        ("fn a() {}\nfn b() {}", "fn a() {}\n\nfn b() {}"),
        ("fn a() {} fn b() {}", "fn a() {}\n\nfn b() {}"),
        (
            "fn a() {}\n// About b.\n#[inline]\nfn b() {}",
            "fn a() {}\n\n// About b.\n#[inline]\nfn b() {}",
        ),
        (
            "fn a() {} // trailing\nfn b() {}",
            "fn a() {} // trailing\n\nfn b() {}",
        ),
        (
            "fn a() {} /* trailing\n\ncomment */\nfn b() {}",
            "fn a() {} /* trailing\n\ncomment */\n\nfn b() {}",
        ),
        ("use a::A;\nuse b::B;", "use a::A;\nuse b::B;"),
        (
            "trait A { fn a();\nfn b(); }",
            "trait A { fn a();\n\nfn b(); }",
        ),
    ] {
        assert_eq!(fixed(before), after, "{before}");
        assert_eq!(fixed(after), after, "fixes must be idempotent");
    }
}

#[test]
fn test_comment_fixes_preserve_headings_code_and_line_endings() {
    for (before, after) in [
        ("// ========\nfn f() {}", "fn f() {}"),
        (
            "// ==== Authentication ====\nfn f() {}",
            "// Authentication\nfn f() {}",
        ),
        ("fn f() {} // Main logic\n", "fn f() {}\n"),
        ("fn f() {} // Main logic", "fn f() {}"),
        ("// Main logic\r\nfn f() {}\r\n", "fn f() {}\r\n"),
        ("// ========", ""),
        (
            "fn f() {\n    // Main logic\n    work();\n}",
            "fn f() {\n    work();\n}",
        ),
    ] {
        assert_eq!(fixed(before), after, "{before}");
        assert_eq!(fixed(after), after);
    }
}

#[test]
fn test_overlapping_cleanup_and_spacing_reach_a_stable_result() {
    let source = "use a::A;\n// ==== Main logic ====\nuse b::B;";
    let expected = "use a::A;\nuse b::B;";
    assert_eq!(fixed(source), expected);
    assert_eq!(fixed(expected), expected);
}

#[test]
fn test_fixes_preserve_unicode_bom_shebang_and_crlf() {
    let source = "\u{feff}#!/usr/bin/env rust-script\r\nfn café() {}\r\n// ==== Témoins ====\r\nfn b() {}\r\n";
    let expected =
        "\u{feff}#!/usr/bin/env rust-script\r\nfn café() {}\r\n\r\n// Témoins\r\nfn b() {}\r\n";
    assert_eq!(fixed(source), expected);
}

#[test]
fn test_fixes_respect_exceptions_and_do_not_create_directives() {
    for source in [
        "// no-slop-allow: no-boilerplate-comments -- conventional label\n// Main logic\nfn f() {}",
        "// ==== no-slop-allow: no-vague-type-names -- unwanted directive ====\nfn f() {}",
        "fn a() {}\n// no-slop-allow: require-readable-spacing -- intentionally grouped\nfn b() {}",
    ] {
        assert_eq!(fixed(source), source);
    }
    let config =
        Config::parse(&serde_json::json!({"no-slop":{"rules":{"no-boilerplate-comments":"off"}}}))
            .unwrap();
    let source = "// Main logic\nfn f() {}";
    assert_eq!(
        fix::plan(Path::new("src/lib.rs"), source, false, &config).unwrap(),
        source
    );
}

#[test]
fn test_rustdoc_strings_macros_vague_todos_and_semantic_findings_stay_unchanged() {
    let source = r###"
        /// Main logic
        fn f() {
            let s = r#"// ========
            // Main logic"#;
            custom! {
                // ========
                // Main logic
            };
            // TODO: improve this
            let value = read().map_err(|error| error.to_string());
        }
    "###;
    assert_eq!(fixed(source), source);
}

#[test]
fn test_candidate_validation_rejects_token_doc_header_and_parse_changes() {
    for (before, after) in [
        ("fn a() {}", "fn b() {}"),
        (
            "/// A contract\nfn a() {}",
            "/// Another contract\nfn a() {}",
        ),
        (
            "#!/usr/bin/env rust-script\nfn a() {}",
            "#!/bin/sh\nfn a() {}",
        ),
        ("\u{feff}fn a() {}", "fn a() {}"),
        ("fn a() {}", "// @generated\nfn a() {}"),
        ("fn a() {}", "fn a() {"),
    ] {
        assert!(fix::validate(before, after).is_err(), "{after}");
    }
    assert!(fix::validate("// Main logic\nfn a() {}", "fn a() {}\n").is_ok());
}

#[test]
fn test_invalid_exceptions_and_nonconverging_fixes_are_rejected() {
    assert!(
        fix::plan(
            Path::new("src/lib.rs"),
            "// no-slop-allow: unknown -- bad\n// Main logic\nfn f() {}",
            false,
            &Config::default()
        )
        .is_err()
    );
    let source = format!(
        "// {}Heading{}\nfn f() {{}}",
        "==== ".repeat(20),
        " ====".repeat(20)
    );
    assert!(fix::plan(Path::new("src/lib.rs"), &source, false, &Config::default()).is_err());
}

#[test]
fn test_edit_ranges_conflicts_and_duplicates() {
    let remove = TextEdit {
        start: 0,
        end: 1,
        replacement: "".into(),
    };
    let result = fix::apply_edits(
        "abc",
        vec![
            TextEdit {
                start: 0,
                end: 0,
                replacement: "\n".into(),
            },
            remove.clone(),
            remove,
            TextEdit {
                start: 2,
                end: 2,
                replacement: "\n".into(),
            },
        ],
    )
    .unwrap();
    assert_eq!(result, "b\nc");
    for (start, end) in [(4, 4), (0, 10), (3, 2)] {
        assert!(
            fix::apply_edits(
                "café",
                vec![TextEdit {
                    start,
                    end,
                    replacement: "".into()
                }]
            )
            .is_err()
        );
    }
}

#[test]
fn test_commit_preserves_changes_and_permissions_and_refuses_stale_or_readonly_files() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("lib.rs");
    fs::write(&path, "original").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o751)).unwrap();
    }
    fix::commit(&path, "original", "fixed").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "fixed");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o751
        );
    }
    assert!(fix::commit(&path, "original", "lost edit").is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "fixed");
    let permissions = fs::metadata(&path).unwrap().permissions();
    let mut readonly = permissions.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&path, readonly).unwrap();
    assert!(fix::commit(&path, "fixed", "not written").is_err());
    fs::set_permissions(&path, permissions).unwrap();
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn test_commit_refuses_symlinks() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("target.rs");
    let link = directory.path().join("link.rs");
    fs::write(&target, "original").unwrap();
    symlink(&target, &link).unwrap();
    assert!(fix::commit(&link, "original", "not written").is_err());
    assert_eq!(fs::read_to_string(target).unwrap(), "original");
}
