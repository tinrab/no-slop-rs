//! Editable, syntax-based rules for common agent-generated Rust patterns.
//!
//! Integrate by dispatching `no-slop` to [`run`]. Application code is read only.
//! Rules inspect written syntax, including inactive cfg branches, without macro
//! expansion, inferred types, or cross-file alias resolution.

pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod diagnostic;
mod discovery;
pub(crate) mod rules;
mod source;
mod suppression;
mod test_scope;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use syn::{self, File};

use self::cli::Options;
use self::config::Config;
use self::rules::RULES;
pub use cli::run;
use context::Context;
use diagnostic::{Failure, Level, Report};

struct ParsedFile {
    path: PathBuf,
    source: String,
    syntax: File,
    is_test: bool,
    is_crate_root: bool,
}

pub(crate) fn scan(options: &Options) -> Result<Report> {
    let workspace = discovery::discover(options)?;
    let mut report = workspace.report;
    let mut parsed = Vec::new();
    for file in workspace.files {
        let path = file
            .path
            .strip_prefix(&workspace.root)
            .unwrap_or(&file.path);
        let source = match fs::read_to_string(&file.path) {
            Ok(source) => source,
            Err(error) => {
                report.failures.push(Failure {
                    path: path.to_owned(),
                    message: error.to_string(),
                    position: None,
                });
                continue;
            }
        };
        if let Some(syntax) = parse_source(path, &source, &mut report) {
            parsed.push(ParsedFile {
                path: file.path,
                source,
                syntax,
                is_test: file.is_test,
                is_crate_root: file.is_crate_root,
            });
        }
    }
    test_scope::propagate(&mut parsed);
    for file in parsed {
        let path = file
            .path
            .strip_prefix(&workspace.root)
            .unwrap_or(&file.path);
        let context = Context::new(path, &file.source, &file.syntax, file.is_test);
        check_file(&context, &file.syntax, &workspace.config, &mut report);
    }
    report.sort();
    Ok(report)
}

pub(crate) fn parse_source(path: &Path, source: &str, report: &mut Report) -> Option<File> {
    let prepared = source::prepare(source);
    let comments = source::comments(&prepared);
    if source::is_generated(&prepared, &comments) {
        report.summary.skipped += 1;
        return None;
    }
    let file = match syn::parse_file(&prepared) {
        Ok(file) => file,
        Err(error) => {
            // Even a failed parse provides an address into the original source.
            let empty = File {
                shebang: None,
                frontmatter: None,
                attrs: Vec::new(),
                items: Vec::new(),
            };
            let context = Context::new(path, source, &empty, false);
            report.failures.push(Failure {
                path: path.to_owned(),
                message: error.to_string(),
                position: Some(context.position(error.span().byte_range().start)),
            });
            return None;
        }
    };
    report.summary.scanned += 1;
    Some(file)
}

pub(crate) fn check_file(context: &Context<'_>, file: &File, config: &Config, report: &mut Report) {
    let comments = &context.comments;
    let mut diagnostics = Vec::new();
    for rule in RULES {
        let level = config.level(rule.id);
        if level == Level::Off {
            continue;
        }
        diagnostics.extend(
            (rule.check)(file, context)
                .into_iter()
                .map(|mut diagnostic| {
                    diagnostic.level = level;
                    diagnostic
                }),
        );
    }
    let (suppressed, failures) = suppression::apply(context, comments, &mut diagnostics);
    report.summary.suppressed += suppressed;
    report.diagnostics.extend(diagnostics);
    report.failures.extend(failures);
}
