//! Vertical spacing between the statements of a block.

use std::path::Path;

use crate::no_slop::{config::Config, fix};

use super::findings;

const RUN: &str = "no-dense-statement-runs";

const BLOCK: &str = "require-blank-after-block";

fn statements(count: usize) -> String {
    "step();\n".repeat(count)
}

fn fixed(source: &str) -> String {
    let config = Config::default();
    fix::plan(Path::new("src/lib.rs"), source, false, &config).unwrap()
}

#[test]
fn test_reports_a_run_once_at_its_first_statement_from_the_threshold() {
    assert!(findings(RUN, &format!("fn f() {{\n{}}}", statements(5))).is_empty());
    for count in [6, 30] {
        let source = format!("fn f() {{\nfirst();\n{}}}", statements(count - 1));
        let diagnostics = findings(RUN, &source);
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert_eq!(diagnostics[0].start.line, 2);
        assert!(diagnostics[0].message.starts_with(&format!("{count} ")));
        assert!(diagnostics[0].fix.is_none());
    }
}

#[test]
fn test_blank_lines_end_a_run_wherever_they_sit_in_the_gap() {
    let run = statements(4);
    for gap in ["\n", "\n// note\n", "// note\n\n", "\r\n"] {
        let source = format!("fn f() {{\n{run}{gap}{run}}}");
        assert!(findings(RUN, &source).is_empty(), "{source}");
    }
    let two_runs = format!("fn f() {{\n{}\n{}}}", statements(8), statements(9));
    assert_eq!(findings(RUN, &two_runs).len(), 2);
}

#[test]
fn test_comments_alone_do_not_end_a_run() {
    let run = statements(4);
    for gap in ["// note\n", "/* first\n\nsecond */\n"] {
        let source = format!("fn f() {{\n{run}{gap}{run}}}");
        assert_eq!(findings(RUN, &source).len(), 1, "{source}");
    }
}

#[test]
fn test_counts_siblings_in_each_block_rather_than_nested_statements() {
    let nested = format!("fn f() {{\nif ready {{\n{}}}\nlast();\n}}", statements(5));
    assert!(findings(RUN, &nested).is_empty());
    for source in [
        format!("fn f() {{\nif ready {{\n{}}}\n}}", statements(8)),
        format!("fn f() {{\nlet g = || {{\n{}}};\n}}", statements(8)),
        format!("impl T {{\nfn f() {{\n{}}}\n}}", statements(8)),
    ] {
        assert_eq!(findings(RUN, &source).len(), 1, "{source}");
    }
}

#[test]
fn test_items_end_a_run_and_never_count_toward_one() {
    let imports = "use std::io;\n".repeat(8);
    assert!(findings(RUN, &format!("fn f() {{\n{imports}}}")).is_empty());
    let split = format!(
        "fn f() {{\n{}use std::io;\n{}}}",
        statements(4),
        statements(4)
    );
    assert!(findings(RUN, &split).is_empty());
}

#[test]
fn test_spacing_rules_exempt_test_scopes_and_macro_bodies() {
    let run = statements(8);
    for source in [
        format!("#[test]\nfn test_f() {{\n{run}}}"),
        format!("#[cfg(test)]\nmod tests {{\nfn f() {{\n{run}}}\n}}"),
        format!("fn f() {{\nrepeat!({{\n{run}}});\n}}"),
    ] {
        assert!(findings(RUN, &source).is_empty(), "{source}");
    }
    for source in [
        "#[test]\nfn test_f() {\nif a {\nb();\n}\nc();\n}",
        "#[cfg(test)]\nmod tests {\nfn f() {\nif a {\nb();\n}\nc();\n}\n}",
        "fn f() {\nrepeat!({\nif a {\nb();\n}\nc();\n});\n}",
    ] {
        assert!(findings(BLOCK, source).is_empty(), "{source}");
    }
}

#[test]
fn test_reports_a_statement_directly_after_each_multi_line_block_kind() {
    for block in [
        "if a {\nb();\n}",
        "if a {\nb();\n} else {\nc();\n}",
        "if let Some(a) = b {\nc();\n}",
        "match a {\n_ => {}\n}",
        "for a in b {\nc();\n}",
        "while a {\nb();\n}",
        "loop {\nbreak;\n}",
        "'outer: loop {\nbreak 'outer;\n}",
        "{\nb();\n}",
        "unsafe {\nb();\n}",
        "if a {\nb();\n};",
    ] {
        for next in ["let d = 1;", "d();", "if e {\nf();\n}"] {
            let source = format!("fn f() {{\n{block}\n{next}\nlast()\n}}");
            let diagnostics = findings(BLOCK, &source);
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert!(diagnostics[0].fix.is_some(), "{source}");
        }
    }
}

#[test]
fn test_accepts_spaced_one_line_and_value_producing_blocks() {
    for body in [
        "if a {\nb();\n}\n\nc();\nlast()",
        "if a { b(); }\nc();\nlast()",
        "let d = if a {\n1\n} else {\n2\n};\nc();\nlast()",
        "let Some(d) = a else {\nreturn;\n};\nc();\nlast()",
        "a.iter().for_each(|b| {\nc(b);\n});\nd();\nlast()",
        "call(\na,\n);\nd();\nlast()",
    ] {
        let source = format!("fn f() {{\n{body}\n}}");
        assert!(findings(BLOCK, &source).is_empty(), "{source}");
    }
}

#[test]
fn test_exempts_only_a_final_return_expression() {
    for ending in [
        "last()",
        "return last();",
        "return last()",
        "last!()",
        "last! {}",
        "if c {\nd\n} else {\ne\n}",
    ] {
        let source = format!("fn f() {{\nif a {{\nb();\n}}\n{ending}\n}}");
        assert!(findings(BLOCK, &source).is_empty(), "{source}");
    }
    for source in [
        "fn f() {\nif a {\nb();\n}\nlast();\n}",
        "fn f() {\nif a {\nb();\n}\nlast!();\n}",
        "fn f() {\nif a {\nb();\n}\nif c {\nreturn;\n}\nlast()\n}",
    ] {
        assert_eq!(findings(BLOCK, source).len(), 1, "{source}");
    }
}

#[test]
fn test_block_fix_preserves_comments_indentation_and_line_endings() {
    for (source, expected) in [
        (
            "fn f() {\n    if a {\n        b();\n    }\n    c();\n    d()\n}\n",
            "fn f() {\n    if a {\n        b();\n    }\n\n    c();\n    d()\n}\n",
        ),
        (
            "fn f() {\n    if a {\n    } // trailing\n    c();\n    d()\n}\n",
            "fn f() {\n    if a {\n    } // trailing\n\n    c();\n    d()\n}\n",
        ),
        (
            "fn f() {\n    if a {\n    }\n    // leading\n    #[cfg(x)]\n    c();\n    d()\n}\n",
            "fn f() {\n    if a {\n    }\n\n    // leading\n    #[cfg(x)]\n    c();\n    d()\n}\n",
        ),
        (
            "fn f() {\r\n    if a {\r\n    }\r\n    c();\r\n    d()\r\n}\r\n",
            "fn f() {\r\n    if a {\r\n    }\r\n\r\n    c();\r\n    d()\r\n}\r\n",
        ),
        (
            "fn f() {\n    for a in b {\n        if a {\n        }\n        c();\n    }\n    d();\n    e()\n}\n",
            "fn f() {\n    for a in b {\n        if a {\n        }\n\n        c();\n    }\n\n    d();\n    e()\n}\n",
        ),
    ] {
        assert_eq!(fixed(source), expected);
    }
}

#[test]
fn test_a_block_fix_can_resolve_a_dense_run_and_exceptions_are_kept() {
    let source = format!("fn f() {{\nif a {{\n}}\n{}}}\n", statements(5));
    assert_eq!(findings(RUN, &source).len(), 1);
    assert!(findings(RUN, &fixed(&source)).is_empty());

    let excepted = "fn f() {\nif a {\n}\n// no-slop-allow: require-blank-after-block -- paired with the check\nc();\nd()\n}\n";
    assert_eq!(fixed(excepted), excepted);
}

#[test]
fn test_block_fix_finishes_files_with_more_blocks_than_fix_passes() {
    let block = "    if a {\n    }\n    b();\n";
    let spaced = "    if a {\n    }\n\n    b();\n";
    let source = format!("fn f() {{\n{}}}\n", block.repeat(40));
    let expected = format!("fn f() {{\n{}}}\n", spaced.repeat(40));
    assert_eq!(fixed(&source), expected);
}
