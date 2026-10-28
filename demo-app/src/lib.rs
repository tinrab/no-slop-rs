//! Compiling examples of every no-slop rule violation.
//! `cargo xtask no-slop -p demo-app` should report one finding per rule.
//! Triggers `require-grouped-imports`: these std imports share a written root.

use std::env;
use std::{
    error::Error as StdError,
    fs::read_to_string,
    io::{self, Error},
    path::{Path, PathBuf},
};

use anyhow::Context as _;

/// Return the current executable's path.
///
/// # Errors
///
/// Returns an error when the operating system cannot determine the executable path.
pub fn executable_path() -> io::Result<PathBuf> {
    env::current_exe()
}

// Triggers `require-lowercase-error-messages`: capitalize the opening deliberately.
#[must_use]
pub fn capitalized_error() -> Error {
    Error::other("Failed to load configuration")
}

// Triggers `no-vague-error-messages`: the text identifies no operation or failure.
#[must_use]
pub fn vague_failure() -> anyhow::Error {
    anyhow::anyhow!("operation failed")
}

// Triggers `no-duplicate-error-context`: these adjacent contexts repeat a message.
/// Read configuration while demonstrating duplicate context.
///
/// # Errors
///
/// Returns an error when the configuration file cannot be read as UTF-8 text.
pub fn repeated_context(path: &Path) -> anyhow::Result<String> {
    read_to_string(path)
        .context("reading configuration")
        .context("reading configuration")
}

pub struct Connection;

impl Connection {
    // Triggers `require-create-for-fallible-constructors`: this should be create.
    #[must_use]
    pub const fn new() -> Option<Self> {
        Some(Self)
    }
}

// Triggers `require-readable-spacing`: these sibling declarations deliberately
// have no blank line between them. rustfmt preserves that gap.
#[must_use]
pub const fn first_declaration() -> u8 {
    1
}
#[must_use]
pub const fn second_declaration() -> u8 {
    2
}

// Triggers `no-string-errors`: String erases a structured error contract.
/// Demonstrate an opaque error contract.
///
/// # Errors
///
/// Always returns a string error instead of a structured failure.
pub fn opaque_failure() -> Result<(), String> {
    Err("could not load configuration".to_owned())
}

// Triggers `no-blanket-lint-suppression`: a reason does not make suppressing the
// entire warnings group narrow enough.
#[allow(warnings, reason = "demonstrates a blanket suppression")]
pub const fn blanket_suppression() {}

// Triggers `no-vague-type-names`: Manager does not name a concrete responsibility.
pub struct UserManager;

// Triggers `no-decorative-comments`: the next comment is a decorative separator.
// ========
pub const fn decorated_section() {}

// Triggers `no-boilerplate-comments`: the next comment is an empty category label.
// Main logic
pub const fn boilerplate_label() {}

// Triggers `no-vague-todos`: the next comment promises no specific remaining task.
// TODO: improve this
pub const fn vague_task() {}

// Triggers `no-stringified-error-mapping`: map_err returns only the error's display
// string. Boxing it avoids a second finding for a string error return contract.
/// Parse an integer while demonstrating loss of the original error.
///
/// # Errors
///
/// Returns a boxed display string when the input is not a valid unsigned integer.
pub fn stringified_failure(input: &str) -> Result<usize, Box<dyn StdError>> {
    input
        .parse::<usize>()
        .map_err(|error| error.to_string())
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::first_declaration;

    #[derive(Debug, thiserror::Error)]
    enum ReadFailure {
        // Triggers `require-error-sources`: this field should declare #[source].
        #[error("could not read configuration")]
        Read(io::Error),
    }

    // Triggers `no-vague-test-names`: the prefix alone describes no behavior.
    #[test]
    fn test_basic() {
        let failure = ReadFailure::Read(io::Error::other("could not read configuration"));
        assert_eq!(failure.to_string(), "could not read configuration");
    }

    // Triggers `require-test-prefix`: this test function does not begin with test_.
    #[test]
    fn returns_first_value() {
        assert_eq!(first_declaration(), 1);
    }
}
