//! Each rule owns its traversal. Add a function and register it here to add a rule.

mod comments;
mod error_discard;
mod lint_suppressions;
mod readable_spacing;
mod string_errors;
mod stringified_errors;
mod test_prefix;
mod vague_names;

use syn::{File, Path};

use super::context::Context;
use super::diagnostic::Diagnostic;

pub(crate) struct Rule {
    pub id: &'static str,
    pub description: &'static str,
    pub check: fn(&File, &Context<'_>) -> Vec<Diagnostic>,
}

pub(crate) const RULES: &[Rule] = &[
    Rule {
        id: "require-test-prefix",
        description: "Name test-attributed functions with the test_ prefix; ordinary helpers are exempt.",
        check: test_prefix::check,
    },
    Rule {
        id: "require-readable-spacing",
        description: "Separate sibling declarations with blank lines while preserving compact imports, modules, and constants.",
        check: readable_spacing::check,
    },
    Rule {
        id: "no-string-errors",
        description: "Use meaningful error types instead of Result errors of String or &str.",
        check: string_errors::check,
    },
    Rule {
        id: "no-silent-error-discard",
        description: "Explain intentional loss of errors through .ok() or Result::ok filters.",
        check: error_discard::check,
    },
    Rule {
        id: "no-blanket-lint-suppression",
        description: "Avoid suppressing warnings or clippy::all as a group.",
        check: lint_suppressions::check_blanket,
    },
    Rule {
        id: "no-vague-type-names",
        description: "Name responsibilities rather than Manager, Helper, Processor, or Data types.",
        check: vague_names::check,
    },
    Rule {
        id: "no-decorative-comments",
        description: "Replace decorative comment separators and framed headings with ordinary explanations.",
        check: comments::check_decorative,
    },
    Rule {
        id: "no-boilerplate-comments",
        description: "Remove exact boilerplate labels that add no explanation.",
        check: comments::check_boilerplate,
    },
    Rule {
        id: "no-vague-todos",
        description: "Replace empty TODO/FIXME comments and generic placeholders with concrete tasks.",
        check: comments::check_todos,
    },
    Rule {
        id: "no-stringified-error-mapping",
        description: "Preserve the original error instead of directly mapping it to its display string.",
        check: stringified_errors::check,
    },
];

pub(crate) fn is_known(id: &str) -> bool {
    RULES.iter().any(|rule| rule.id == id)
}

pub(crate) fn path_name(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
