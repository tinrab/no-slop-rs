//! Author-only tests and helpers. This directory is not part of the skill bundle.

mod comments;
mod error_discard;
mod lint_suppressions;
mod readable_spacing;
mod scanner;
mod string_errors;
mod stringified_errors;
mod test_prefix;
mod vague_names;

use std::path::Path;

use syn::parse_file;

use crate::no_slop::config::Config;
use crate::no_slop::context::Context;
use crate::no_slop::diagnostic::{Diagnostic, Report};
use crate::no_slop::rules::RULES;
use crate::no_slop::{check_file, parse_source};

fn findings(id: &str, source: &str) -> Vec<Diagnostic> {
    let file = parse_file(source).unwrap();
    let context = Context::new(Path::new("src/lib.rs"), source, &file, false);
    let rule = RULES.iter().find(|rule| rule.id == id).unwrap();
    (rule.check)(&file, &context)
}

fn scan_source(path: &Path, source: &str, test_file: bool, config: &Config, report: &mut Report) {
    if let Some(file) = parse_source(path, source, report) {
        let context = Context::new(path, source, &file, test_file);
        check_file(&context, &file, config, report);
    }
}
