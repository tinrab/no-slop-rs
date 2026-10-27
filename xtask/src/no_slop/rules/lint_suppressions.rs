//! Suppressing warnings or clippy::all requires
//! a no-slop exception even when the Rust attribute already has a reason.
//! Inspect cfg_attr contents without evaluating their condition.

use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::visit::{self, Visit};
use syn::{Attribute, File, Meta, MetaList, Token};

use super::super::context::Context;
use super::super::diagnostic::Diagnostic;

pub(crate) fn check_blanket(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    check_attributes(file, context, |list, context| {
        let Ok(parts) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            return None;
        };
        let broad = parts.iter().any(|meta| {
            matches!(meta, Meta::Path(path) if matches!(
                super::path_name(path).as_str(), "warnings" | "clippy::all"
            ))
        });
        broad.then(|| {
            context.report(
                "no-blanket-lint-suppression",
                list.span(),
                "blanket lint suppression hides unrelated findings",
                "Keep exceptions narrow and explain each lint being suppressed.",
            )
        })
    })
}

fn check_attributes(
    file: &File,
    context: &Context<'_>,
    check: fn(&MetaList, &Context<'_>) -> Option<Diagnostic>,
) -> Vec<Diagnostic> {
    struct Check<'a> {
        context: &'a Context<'a>,
        check: fn(&MetaList, &Context<'_>) -> Option<Diagnostic>,
        findings: Vec<Diagnostic>,
    }

    impl Check<'_> {
        fn meta(&mut self, meta: &Meta) {
            let Meta::List(list) = meta else { return };
            if list.path.is_ident("allow") || list.path.is_ident("expect") {
                self.findings.extend((self.check)(list, self.context));
            } else if list.path.is_ident("cfg_attr") {
                if let Ok(parts) =
                    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                {
                    for nested in parts.iter().skip(1) {
                        self.meta(nested);
                    }
                }
            }
        }
    }

    impl<'ast> Visit<'ast> for Check<'_> {
        fn visit_attribute(&mut self, attribute: &'ast Attribute) {
            self.meta(&attribute.meta);
            visit::visit_attribute(self, attribute);
        }
    }
    let mut visitor = Check {
        context,
        check,
        findings: Vec::new(),
    };
    visitor.visit_file(file);
    visitor.findings
}
