use std::path::Path;

use crate::no_slop::{config::Config, fix};

use super::findings;

fn fixed(source: &str) -> String {
    fix::plan(Path::new("src/lib.rs"), source, false, &Config::default()).unwrap()
}

#[test]
fn test_splits_after_sentence_punctuation() {
    assert_eq!(
        fixed("/// Returns the value. Fails if `x` is 0! Then [`Foo`] runs.\nfn f() {}"),
        "/// Returns the value.\n/// Fails if `x` is 0!\n/// Then [`Foo`] runs.\nfn f() {}"
    );
}

#[test]
fn test_rejoins_wrapped_sentences_and_keeps_indent_and_markers() {
    assert_eq!(
        fixed(
            "//! First part\n//! continues. Second.\n\nmod m {\n    /// One\n    /// sentence.\n    fn f() {}\n}"
        ),
        "//! First part continues.\n//! Second.\n\nmod m {\n    /// One sentence.\n    fn f() {}\n}"
    );
}

#[test]
fn test_reflows_list_items_with_continuation_indent() {
    assert_eq!(
        fixed("/// - First. Second\n///   part.\n/// 12. Third. Fourth.\nfn f() {}"),
        "/// - First.\n///   Second part.\n/// 12. Third.\n///     Fourth.\nfn f() {}"
    );
}

#[test]
fn test_combines_with_import_grouping_in_one_file() {
    assert_eq!(
        fixed("use std::env;\nuse std::fs;\n\n/// First. Second.\nfn f() {}"),
        "use std::{env, fs};\n\n/// First.\n/// Second.\nfn f() {}"
    );
}

#[test]
fn test_validation_still_rejects_changed_doc_words() {
    assert!(
        fix::validate(
            "/// First. Second.\nfn f() {}",
            "/// First.\n/// Other.\nfn f() {}"
        )
        .is_err()
    );
}

#[test]
fn test_preserves_crlf_line_endings() {
    assert_eq!(
        fixed("/// First. Second.\r\nfn f() {}\r\n"),
        "/// First.\r\n/// Second.\r\nfn f() {}\r\n"
    );
}

#[test]
fn test_reports_once_per_paragraph_with_matching_message() {
    let result = findings(
        "require-sentence-per-line",
        "/// First. Second.\n///\n/// Third\n/// part.\nfn f() {}",
    );
    let messages: Vec<_> = result.iter().map(|d| d.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "doc comment line holds more than one sentence",
            "sentence continues onto the next doc comment line"
        ]
    );
}

#[test]
fn test_keeps_abbreviations_code_links_and_numbers_inside_a_sentence() {
    for source in [
        "/// Uses e.g. Foo, `a. B`, [see. It](x), and version 13.4 of v1.\nfn f() {}",
        "/// Step 2. Then i.e. More and etc. Done.\nfn f() {}",
        "/// Named after J. Smith.\nfn f() {}",
        "/// ends lowercase. next starts lowercase.\nfn f() {}",
    ] {
        assert!(
            findings("require-sentence-per-line", source).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn test_leaves_layout_sensitive_markdown_and_other_comments_alone() {
    let source = r"
/// ```
/// let a = 1. Then b.
/// ```
///
/// | A. B. | C. D. |
/// # Heading. Two.
/// > Quote. Two.
/// <br> Html. Two.
/// [link]: https://example.com. Two.
///
///     indented code. Two.
///
/// Hard break.
/// Next line\
/// continues.
//// Not docs. Two.
// Ordinary. Two.
/* Block. Two. */
fn f() {}
";
    assert!(findings("require-sentence-per-line", source).is_empty());
}

#[test]
fn test_trailing_code_comment_does_not_join_paragraphs() {
    let source = "/// First\nfn f() {}\n/// second.\nfn g() {}";
    assert!(findings("require-sentence-per-line", source).is_empty());
}
