//! Report thiserror messages that print the whole field `source()` already returns.
//! Message wording is never judged, and derived details such as `source.kind()` are accepted.
//! Handwritten `Display`, enum-level messages, and conditional attributes are not inspected.

use std::collections::HashMap;

use proc_macro2::{Spacing, TokenStream, TokenTree};
use syn::{
    Attribute, Fields, File, ItemEnum, ItemStruct, LitStr, Meta,
    ext::IdentExt as _,
    parse2,
    visit::{self, Visit},
};

use super::{
    super::{context::Context, diagnostic::Diagnostic},
    error_support::{conditional, derived, is_source_field},
};

enum Placeholder {
    Next,
    Index(usize),
    Name(String),
}

fn placeholders(format: &str) -> Vec<Placeholder> {
    let mut found = Vec::new();
    let mut chars = format.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' || chars.next_if_eq(&'{').is_some() {
            continue;
        }

        let inner: String = chars.by_ref().take_while(|&ch| ch != '}').collect();
        let name = inner.split(':').next().unwrap_or("").trim();
        found.push(if name.is_empty() {
            Placeholder::Next
        } else if let Ok(index) = name.parse() {
            Placeholder::Index(index)
        } else {
            Placeholder::Name(name.to_owned())
        });
    }
    found
}

/// The field an argument prints in full; thiserror binds tuple fields as `_0`.
fn field_reference(tokens: &[TokenTree]) -> Option<String> {
    match tokens {
        [TokenTree::Punct(dot), TokenTree::Ident(field)] if dot.as_char() == '.' => {
            Some(field.unraw().to_string())
        }
        [TokenTree::Punct(dot), TokenTree::Literal(index)] if dot.as_char() == '.' => {
            Some(index.to_string())
        }
        [TokenTree::Ident(field)] => {
            let field = field.unraw().to_string();
            match field.strip_prefix('_') {
                Some(index) if index.parse::<usize>().is_ok() => Some(index.to_owned()),
                _ => Some(field),
            }
        }
        _ => None,
    }
}

#[derive(Default)]
struct Arguments {
    positional: Vec<Option<String>>,
    named: HashMap<String, Option<String>>,
}

fn split_commas(tokens: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut parts = vec![Vec::new()];
    for token in tokens {
        match &token {
            TokenTree::Punct(punct) if punct.as_char() == ',' => parts.push(Vec::new()),
            _ => parts
                .last_mut()
                .expect("parts starts non-empty")
                .push(token),
        }
    }
    parts
}

fn message_and_arguments(attr: &Attribute) -> Option<(LitStr, Arguments)> {
    let Meta::List(list) = &attr.meta else {
        return None;
    };
    let mut parts = split_commas(list.tokens.clone()).into_iter();
    let literal = parse2(parts.next()?.into_iter().collect()).ok()?;
    let mut arguments = Arguments::default();

    for part in parts.filter(|part| !part.is_empty()) {
        match part.as_slice() {
            [TokenTree::Ident(name), TokenTree::Punct(equals), value @ ..]
                if equals.as_char() == '=' && equals.spacing() == Spacing::Alone =>
            {
                arguments
                    .named
                    .insert(name.unraw().to_string(), field_reference(value));
            }
            tokens => arguments.positional.push(field_reference(tokens)),
        }
    }
    Some((literal, arguments))
}

/// Mirror thiserror: `{0}` names a tuple field, and a named argument shadows a field.
fn printed_source(
    literal: &LitStr,
    arguments: &Arguments,
    fields: &Fields,
    sources: &[String],
) -> Option<String> {
    let tuple = matches!(fields, Fields::Unnamed(_));
    let mut next = 0;
    placeholders(&literal.value())
        .into_iter()
        .find_map(|placeholder| {
            let printed = match placeholder {
                Placeholder::Next => {
                    next += 1;
                    arguments.positional.get(next - 1).cloned().flatten()
                }
                Placeholder::Index(index) if tuple => Some(index.to_string()),
                Placeholder::Index(index) => arguments.positional.get(index).cloned().flatten(),
                Placeholder::Name(name) => {
                    arguments.named.get(&name).cloned().unwrap_or(Some(name))
                }
            }?;
            sources.contains(&printed).then_some(printed)
        })
}

pub fn check(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    struct Check<'a> {
        context: &'a Context<'a>,
        findings: Vec<Diagnostic>,
    }

    impl Check<'_> {
        fn wrapper(&mut self, attrs: &[Attribute], fields: &Fields) {
            if conditional(attrs) || fields.iter().any(|field| conditional(&field.attrs)) {
                return;
            }

            let sources: Vec<_> = fields
                .iter()
                .enumerate()
                .filter(|(_, field)| is_source_field(field))
                .map(|(index, field)| {
                    field
                        .ident
                        .as_ref()
                        .map_or_else(|| index.to_string(), |name| name.unraw().to_string())
                })
                .collect();
            for attr in attrs.iter().filter(|attr| attr.path().is_ident("error")) {
                let Some((literal, arguments)) = message_and_arguments(attr) else {
                    continue;
                };
                if let Some(field) = printed_source(&literal, &arguments, fields, &sources) {
                    self.findings.push(self.context.report(
                        "no-source-in-error-display",
                        literal.span(),
                        format!("error message prints source field `{field}`, which `source()` also returns"),
                        "Remove the placeholder so reporters print the cause once, use #[error(transparent)] when the wrapper adds nothing, or explain with a local exception that only this message reaches its reader.",
                    ));
                }
            }
        }
    }

    impl<'ast> Visit<'ast> for Check<'_> {
        fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
            if derived(&item.attrs) {
                self.wrapper(&item.attrs, &item.fields);
            }

            visit::visit_item_struct(self, item);
        }

        fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
            if derived(&item.attrs) {
                for variant in &item.variants {
                    self.wrapper(&variant.attrs, &variant.fields);
                }
            }

            visit::visit_item_enum(self, item);
        }
    }
    let mut check = Check {
        context,
        findings: Vec::new(),
    };
    check.visit_file(file);
    check.findings
}
