//! Diagnostics use one-based lines and columns and UTF-8 byte offsets.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, Serializer};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Off,
    Warn,
    #[default]
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub byte: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub rule: &'static str,
    pub level: Level,
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    pub start: Position,
    pub end: Position,
    pub message: String,
    pub help: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Failure {
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    pub message: String,
    pub position: Option<Position>,
}

fn serialize_path<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&path.display())
}

#[derive(Default, Debug, Serialize)]
pub struct Summary {
    pub scanned: usize,
    /// Excluded files, generated files, symlinks, and pruned directories.
    pub skipped: usize,
    pub suppressed: usize,
}

#[derive(Default, Debug, Serialize)]
pub struct Report {
    pub diagnostics: Vec<Diagnostic>,
    pub failures: Vec<Failure>,
    pub summary: Summary,
}

impl Report {
    pub fn exit_status(&self) -> u8 {
        if !self.failures.is_empty() {
            2
        } else if self.diagnostics.iter().any(|d| d.level == Level::Error) {
            1
        } else {
            0
        }
    }

    pub(crate) fn sort(&mut self) {
        self.diagnostics
            .sort_by(|a, b| (&a.path, a.start.byte, a.rule).cmp(&(&b.path, b.start.byte, b.rule)));
        self.failures
            .sort_by(|a, b| (&a.path, &a.message).cmp(&(&b.path, &b.message)));
    }
}
