use super::findings;

#[test]
fn test_reports_only_separator_and_framed_heading_shapes() {
    let source = "// ========\n// --------\n// ********\n// ==== Authentication ====\n// ---- Tasks ----\n// **** Setup ****\nfn f() {}";
    assert_eq!(findings("no-decorative-comments", source).len(), 6);
    assert!(findings("no-decorative-comments", "// =======\n// --- Heading ---\n// ====> flow\n// A -> B -> C\n// UPPER CASE\n// 🚀 Performance\nfn f() {} // ========").is_empty());
}

#[test]
fn test_boilerplate_matching_is_exact_but_normalizes_case_and_spacing() {
    let source = "// MAIN   LOGIC.\n// core logic\n// Business logic:\n// helper function\n// Entry point\n// error handling\n// end if\n// end loop\n// end function\nfn f() {}";
    assert_eq!(findings("no-boilerplate-comments", source).len(), 9);
    assert!(findings("no-boilerplate-comments", "// Error handling retries only transient failures.\n// Entry point required by the FFI harness.\n// End function validate_token\nfn f() {}").is_empty());
}

#[test]
fn test_vague_todos_have_a_small_exact_dictionary() {
    let source = "// TODO\n// FIXME:\n// TODO: Improve this.\n// TODO : improve this\n// fixme fix this\n// TODO: implement this\n// TODO: add more validation\n// TODO: optimize later!\nfn f() {}";
    assert_eq!(findings("no-vague-todos", source).len(), 8);
    assert!(findings("no-vague-todos", "// TODO: reject overflowing lengths before allocation.\n// TODO(#42): preserve the server's retry deadline.\n// FIXME: fix this by checking the checksum.\n// TODO_LIST is a protocol field.\nfn f() {}").is_empty());
}

#[test]
fn test_comment_rules_ignore_docs_literals_blocks_directives_and_macros() {
    let source = r###"
        //! main logic
        /// ========
        /// TODO: improve this
        fn f() {
            let raw = r#"// main logic
                // ========
                // TODO: improve this"#;
            let bytes = br#"// TODO"#;
            let c_string = cr#"// core logic"#;
            /* // TODO: improve this */
            /* main logic */
            /* ======== */
            // no-slop-allow: no-boilerplate-comments -- main logic
            custom! {
                // main logic
                // ========
                // TODO: improve this
            };
        }
        macro_rules! sample { () => {
            // main logic
            // ========
            // TODO: improve this
        }; }
    "###;
    for id in [
        "no-decorative-comments",
        "no-boilerplate-comments",
        "no-vague-todos",
    ] {
        assert!(findings(id, source).is_empty(), "{id}");
    }
}

#[test]
fn test_comment_findings_use_original_unicode_crlf_and_shebang_offsets() {
    let source =
        "\u{feff}#!/usr/bin/env rust-script // ========\r\n// café\r\nfn f() {} // MAIN LOGIC.\r\n";
    let diagnostics = findings("no-boilerplate-comments", source);
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        &source[diagnostic.start.byte..diagnostic.end.byte],
        "// MAIN LOGIC."
    );
    assert_eq!(diagnostic.start.line, 3);
    assert_eq!(diagnostic.start.column, 11);
    assert!(findings("no-decorative-comments", source).is_empty());
}
