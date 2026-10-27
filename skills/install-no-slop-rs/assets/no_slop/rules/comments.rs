//! Narrow checks on ordinary line comments, excluding opaque macro bodies.
//! Dictionaries describe exact noise patterns, not a judgment of prose quality.

use syn::File;

use super::super::context::Context;
use super::super::diagnostic::Diagnostic;

fn check(
    context: &Context<'_>,
    id: &'static str,
    matches: fn(&str) -> bool,
    help: &'static str,
) -> Vec<Diagnostic> {
    context
        .comments
        .iter()
        .filter(|comment| {
            comment.ordinary_line
                && !context
                    .opaque_macros
                    .iter()
                    .any(|range| range.contains(&comment.range.start))
        })
        .filter_map(|comment| {
            let body = comment.text.trim_start_matches('/').trim();
            if body.starts_with("no-slop-allow") || !matches(body) {
                return None;
            }
            Some(context.report_range(
                id,
                comment.range.start..comment.range.start + comment.text.trim_end().len(),
                "comment adds no concrete explanation",
                help,
            ))
        })
        .collect()
}

pub(crate) fn check_decorative(_file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    fn decorative(body: &str) -> bool {
        ['=', '-', '*'].into_iter().any(|delimiter| {
            let leading = body.chars().take_while(|&c| c == delimiter).count();
            if leading == body.len() {
                return leading >= 8;
            }
            let trailing = body.chars().rev().take_while(|&c| c == delimiter).count();
            leading >= 4 && trailing >= 4 && !body[leading..body.len() - trailing].trim().is_empty()
        })
    }
    let mut findings = check(
        context,
        "no-decorative-comments",
        decorative,
        "Use a plain explanation instead of a separator or framed heading.",
    );
    findings.retain(|finding| {
        context.source[..finding.start.byte]
            .rsplit('\n')
            .next()
            .is_some_and(|prefix| prefix.chars().all(char::is_whitespace))
    });
    findings
}

fn normalized(body: &str) -> String {
    body.trim_end_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub(crate) fn check_boilerplate(_file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    fn boilerplate(body: &str) -> bool {
        matches!(
            normalized(body).as_str(),
            "main logic"
                | "core logic"
                | "business logic"
                | "helper function"
                | "entry point"
                | "error handling"
                | "end if"
                | "end loop"
                | "end function"
        )
    }
    check(
        context,
        "no-boilerplate-comments",
        boilerplate,
        "Remove the label or explain a concrete decision, constraint, or invariant.",
    )
}

pub(crate) fn check_todos(_file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    fn vague(body: &str) -> bool {
        let body = normalized(body);
        let task = ["todo", "fixme"].into_iter().find_map(|marker| {
            let suffix = body.strip_prefix(marker)?;
            if !suffix.is_empty() && !suffix.starts_with(':') && !suffix.starts_with(' ') {
                return None;
            }
            let suffix = suffix.trim_start();
            Some(suffix.strip_prefix(':').unwrap_or(suffix).trim())
        });
        task.is_some_and(|task| {
            matches!(
                task,
                "" | "improve this"
                    | "fix this"
                    | "implement this"
                    | "add more validation"
                    | "optimize later"
            )
        })
    }
    check(
        context,
        "no-vague-todos",
        vague,
        "Describe the specific remaining change; an issue link is optional.",
    )
}
