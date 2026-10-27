//! Author-only tests and helpers. This directory is not part of the skill bundle.

mod comments;
mod error_discard;
mod error_messages;
mod fallible_constructors;
mod fixes;
mod grouped_imports;
mod lint_suppressions;
mod readable_spacing;
mod scanner;
mod string_errors;
mod stringified_errors;
mod test_prefix;
mod vague_names;

use std::path::Path;

use syn::parse_file;

use crate::no_slop::{
    check_file,
    config::Config,
    context::Context,
    diagnostic::{Diagnostic, Report},
    parse_source,
    rules::RULES,
};

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
