//! Eagerly formatted context, its exemptions, and when deferring it is safe to apply.

use std::path::Path;

use crate::no_slop::{config::Config, fix};

use super::findings;

const RULE: &str = "no-eager-error-context";

fn fixed(source: &str) -> String {
    fix::plan(Path::new("src/lib.rs"), source, false, &Config::default()).unwrap()
}

#[test]
fn test_fixes_calls_with_deferrable_arguments() {
    for (before, after) in [
        (
            r#"fn f() { read().context(format!("reading {path}")); }"#,
            r#"fn f() { read().with_context(|| format!("reading {path}")); }"#,
        ),
        (
            r#"fn f() { read().wrap_err(format!("reading {}", path.display())); }"#,
            r#"fn f() { read().wrap_err_with(|| format!("reading {}", path.display())); }"#,
        ),
        (
            r#"fn f() { self.file.read()?.context(std::format!("{}", self.path)); }"#,
            r#"fn f() { self.file.read()?.with_context(|| std::format!("{}", self.path)); }"#,
        ),
        (
            r#"async fn f() { read().await.context(format!("{name}", name = &config.name)); }"#,
            r#"async fn f() { read().await.with_context(|| format!("{name}", name = &config.name)); }"#,
        ),
        (
            "fn f() { read().context(\n    // Names the file.\n    format!(\"reading {path}\"),\n); }",
            "fn f() { read().with_context(\n    // Names the file.\n    || format!(\"reading {path}\"),\n); }",
        ),
    ] {
        assert_eq!(findings(RULE, before).len(), 1, "{before}");
        assert_eq!(fixed(before), after, "{before}");
        assert!(findings(RULE, after).is_empty(), "{after}");
        assert_eq!(fixed(after), after, "fixes must be idempotent");
    }
}

#[test]
fn test_reports_without_an_edit_when_deferring_could_change_behavior() {
    for expression in [
        r#"read().context(format!("{}", next_id()))"#,
        r#"read().context(format!("{}", ids.next()))"#,
        r#"read().context(format!("{}", name()?))"#,
        r#"read().context(format!("{}", lookup().await))"#,
        r#"read().context(format!("{}", values[index]))"#,
        r#"read().context(format!("{}", name.replace("a", "b")))"#,
        r#"read().context(format!("{}", &mut counter))"#,
        r#"read().context((format!("reading {path}")))"#,
        r#"read().context({ format!("reading {path}") })"#,
        r#"result.context(format!("reading {path}"))"#,
        r#"self.result.context(format!("reading {path}"))"#,
    ] {
        let source = format!("fn f() {{ {expression}; }}");
        let diagnostics = findings(RULE, &source);
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert!(diagnostics[0].fix.is_none(), "{source}");
        assert_eq!(fixed(&source), source);
    }
}

#[test]
fn test_accepts_error_values_lazy_forms_and_other_messages() {
    for expression in [
        r#"read().map_err(|failure| failure.context(format!("reading {path}")))"#,
        r#"read().map_err(|failure: Failure| failure.context(format!("reading {path}")))"#,
        r#"error.context(format!("reading {path}"))"#,
        r#"err.wrap_err(format!("reading {path}"))"#,
        r#"self.context(format!("reading {path}"))"#,
        r#"anyhow!("missing").context(format!("reading {path}"))"#,
        r#"anyhow::anyhow!("missing").context("a").context(format!("reading {path}"))"#,
        r#"anyhow::Error::new(failure).context(format!("reading {path}"))"#,
        r#"Report::msg("missing").wrap_err(format!("reading {path}"))"#,
        r#"read().with_context(|| format!("reading {path}"))"#,
        r#"read().context("reading configuration")"#,
        r#"read().context(message)"#,
        r#"read().context(format_args!("reading {path}"))"#,
        r#"read().other(format!("reading {path}"))"#,
        r#"read().context(format!("a"), extra)"#,
        r#"anyhow::Context::context(read(), format!("reading {path}"))"#,
    ] {
        let source = format!("fn f() {{ {expression}; }}");
        assert!(findings(RULE, &source).is_empty(), "{source}");
    }
}
