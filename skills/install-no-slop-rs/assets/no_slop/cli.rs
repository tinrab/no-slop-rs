//! The command accepts OS paths without requiring UTF-8.

use std::ffi::OsString;
use std::io::{self, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context as _, Result, bail};
use serde::Serialize;
use serde_json;

use super::diagnostic::{Failure, Level, Report};
use super::rules::RULES;
use super::scan;

#[derive(Clone, Copy, Default)]
pub(crate) enum Format {
    #[default]
    Text,
    Json,
}

#[derive(Default)]
pub(crate) struct Options {
    pub manifest_path: Option<PathBuf>,
    pub packages: Vec<String>,
    pub format: Format,
    pub list_rules: bool,
    pub help: bool,
}

impl Options {
    pub fn parse(args: &[OsString]) -> Result<Self> {
        let mut options = Self::default();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--manifest-path") => {
                    options.manifest_path = Some(PathBuf::from(
                        args.next().context("--manifest-path needs a path")?,
                    ))
                }
                Some("-p" | "--package") => options.packages.push(
                    args.next()
                        .context("--package needs a name")?
                        .to_str()
                        .context("package names must be UTF-8")?
                        .to_owned(),
                ),
                Some("--format") => {
                    options.format = match args.next().and_then(|arg| arg.to_str()) {
                        Some("text") => Format::Text,
                        Some("json") => Format::Json,
                        _ => bail!("--format requires text or json"),
                    }
                }
                Some("--list-rules") => options.list_rules = true,
                Some("--help" | "-h") => options.help = true,
                _ => bail!("unknown no-slop argument: {}", arg.to_string_lossy()),
            }
        }
        Ok(options)
    }
}

/// Run the scanner from an existing xtask command dispatcher.
pub fn run(args: &[OsString]) -> ExitCode {
    let options = match Options::parse(args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error:#}");
            return ExitCode::from(2);
        }
    };
    if options.help {
        println!(
            "cargo xtask no-slop [-p NAME ...] [--manifest-path PATH] [--format text|json] [--list-rules]"
        );
        println!(
            "Scans all workspace members by default. Exit codes: 0 success, 1 lint errors, 2 incomplete scan."
        );
        return ExitCode::SUCCESS;
    }
    if options.list_rules {
        match options.format {
            Format::Text => {
                for rule in RULES {
                    println!("{}: {}", rule.id, rule.description);
                }
            }
            Format::Json => {
                let rules: Vec<_> = RULES.iter().map(|rule| serde_json::json!({ "id": rule.id, "description": rule.description })).collect();
                if let Err(error) = write_json(&rules) {
                    eprintln!("{error:#}");
                    return ExitCode::from(2);
                }
            }
        }
        return ExitCode::SUCCESS;
    }
    let report = match scan(&options) {
        Ok(report) => report,
        Err(error) => {
            let mut report = Report::default();
            report.failures.push(Failure {
                path: options
                    .manifest_path
                    .unwrap_or_else(|| PathBuf::from("Cargo.toml")),
                message: format!("{error:#}"),
                position: None,
            });
            report
        }
    };
    let status = report.exit_status();
    let output = match options.format {
        Format::Json => write_json(&report),
        Format::Text => write_text(&report),
    };
    if let Err(error) = output {
        eprintln!("writing no-slop output: {error:#}");
        return ExitCode::from(2);
    }
    ExitCode::from(status)
}

fn write_json(value: &impl Serialize) -> Result<()> {
    let mut output = io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, value)?;
    writeln!(output)?;
    Ok(())
}

fn write_text(report: &Report) -> Result<()> {
    let mut output = io::stdout().lock();
    for diagnostic in &report.diagnostics {
        let level = match diagnostic.level {
            Level::Error => "error",
            Level::Warn => "warning",
            Level::Off => "off",
        };
        writeln!(
            output,
            "{}:{}:{}: {level}[{}]: {}",
            diagnostic.path.display(),
            diagnostic.start.line,
            diagnostic.start.column,
            diagnostic.rule,
            diagnostic.message
        )?;
        writeln!(output, "  help: {}", diagnostic.help)?;
    }
    for failure in &report.failures {
        let location = failure
            .position
            .map(|p| format!(":{}:{}", p.line, p.column))
            .unwrap_or_default();
        writeln!(
            output,
            "{}{location}: scan error: {}",
            failure.path.display(),
            failure.message
        )?;
    }
    writeln!(
        output,
        "{} files scanned, {} excluded paths, {} suppressed findings, {} findings, {} scan errors",
        report.summary.scanned,
        report.summary.skipped,
        report.summary.suppressed,
        report.diagnostics.len(),
        report.failures.len()
    )?;
    Ok(())
}
