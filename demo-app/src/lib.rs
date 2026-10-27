//! Compiling examples of every no-slop rule violation.
//! `cargo xtask no-slop -p demo-app` should report one finding per rule.

// Triggers `require-readable-spacing`: these sibling declarations deliberately
// have no blank line between them. rustfmt preserves that gap.
pub fn first_declaration() -> u8 {
    1
}
pub fn second_declaration() -> u8 {
    2
}

// Triggers `no-string-errors`: String erases a structured error contract.
pub fn opaque_failure() -> Result<(), String> {
    Err("something went wrong".to_owned())
}

// Triggers `no-silent-error-discard`: the returned Option drops the parse error.
pub fn discarded_parse_error(input: &str) -> Option<usize> {
    input.parse().ok()
}

// Triggers `no-blanket-lint-suppression`: a reason does not make suppressing the
// entire warnings group narrow enough.
#[allow(warnings, reason = "demonstrates a blanket suppression")]
pub fn blanket_suppression() {}

// Triggers `no-vague-type-names`: Manager does not name a concrete responsibility.
pub struct UserManager;

// Triggers `no-decorative-comments`: the next comment is a decorative separator.
// ========
pub fn decorated_section() {}

// Triggers `no-boilerplate-comments`: the next comment is an empty category label.
// Main logic
pub fn boilerplate_label() {}

// Triggers `no-vague-todos`: the next comment promises no specific remaining task.
// TODO: improve this
pub fn vague_task() {}

// Triggers `no-stringified-error-mapping`: map_err returns only the error's display
// string. Boxing it avoids a second finding for a string error return contract.
pub fn stringified_failure(input: &str) -> Result<usize, Box<dyn std::error::Error>> {
    input
        .parse::<usize>()
        .map_err(|error| error.to_string())
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::first_declaration;

    // Triggers `require-test-prefix`: this test function does not begin with test_.
    #[test]
    fn returns_first_value() {
        assert_eq!(first_declaration(), 1);
    }
}
