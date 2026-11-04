//! Lowercase openings and unpunctuated endings at the error sites recognized by the shared collector.
//! Punctuation is only reported after prose, so token-like messages such as `"."` are left alone.

use std::{iter, ops::Range};

use syn::{File, LitStr, parse_str};

use super::{
    super::{context::Context, diagnostic::Diagnostic},
    error_support::{self, Message},
};

const TRAILING: [char; 5] = ['.', '!', ':', ';', ','];

fn capitalized(message: &str, formatted: bool) -> bool {
    let mut chars = message.chars().peekable();
    while let Some(ch) = chars.next() {
        if formatted && ch == '{' {
            if chars.next_if_eq(&'{').is_some() {
                continue;
            }
            // The opening is determined by runtime output, not this field's name.
            return false;
        }

        if ch.is_lowercase() {
            return false;
        }
        if ch.to_lowercase().ne(iter::once(ch)) {
            return true;
        }
    }
    false
}

/// The byte length of trailing punctuation that follows prose.
fn trailing_punctuation(message: &str) -> usize {
    let kept = message.trim_end_matches(TRAILING);
    let abbreviation = kept
        .rsplit(char::is_whitespace)
        .next()
        .is_some_and(|word| matches!(word, "etc" | "e.g" | "i.e" | "vs" | "cf"));
    let prose = kept.ends_with('}')
        || (kept.chars().any(char::is_alphabetic) && !kept.ends_with(char::is_whitespace));
    if prose && !abbreviation {
        message.len() - kept.len()
    } else {
        0
    }
}

/// Locate the punctuation in a literal's source; escapes that spell it are left unedited.
fn punctuation_range(literal: &str, length: usize) -> Option<Range<usize>> {
    let close = literal.rfind('"')?;
    let start = close.checked_sub(length)?;
    let written = literal.get(start..close)?;
    (written.trim_end_matches(TRAILING).is_empty() && literal[..start].contains('"'))
        .then_some(start..close)
}

/// Remove what the punctuation fix removes from a string literal token, for fix validation.
pub fn without_trailing_punctuation(token: &str) -> Option<String> {
    if !token.ends_with(['"', '#']) {
        return None;
    }

    let literal: LitStr = parse_str(token).ok()?;
    let range = punctuation_range(token, trailing_punctuation(&literal.value()))?;
    Some(format!("{}{}", &token[..range.start], &token[range.end..]))
}

fn punctuated(context: &Context<'_>, message: &Message, length: usize) -> Diagnostic {
    let span = message.literal.span().byte_range();
    let finding = context.report_range(
        "require-lowercase-error-messages",
        span.clone(),
        "error message should not end with punctuation",
        "Remove the trailing punctuation, or explain a required ending with a local exception.",
    );
    let edit = context
        .source
        .get(span.clone())
        .and_then(|literal| punctuation_range(literal, length));
    match edit {
        Some(range) => finding.with_fix(span.start + range.start, span.start + range.end, ""),
        None => finding,
    }
}

pub fn check(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    for message in error_support::messages(file) {
        let value = message.literal.value();
        if capitalized(&value, message.formatted) {
            findings.push(context.report(
                "require-lowercase-error-messages", message.literal.span(),
                "error message should begin with lowercase prose",
                "Use a lowercase opening; preserve acronyms and identifiers later in the message, or explain a required opening with a local exception.",
            ));
        }

        let length = trailing_punctuation(&value);
        if length > 0 {
            findings.push(punctuated(context, &message, length));
        }
    }
    findings
}
