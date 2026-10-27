//! Recognize literal messages at conventional error APIs without name resolution.
//! Only selected macro arguments are parsed; arbitrary macro bodies stay opaque.

use proc_macro2::TokenStream;
use syn::{
    Attribute, Expr, ExprCall, ExprMethodCall, File, ItemStruct, Lit, LitStr, Macro, Stmt, Token,
    Variant,
    parse::{ParseStream, Parser as _},
    visit::{self, Visit},
};

use super::{
    super::{context::Context, diagnostic::Diagnostic},
    path_name,
};

fn unwrapped(mut expr: &Expr) -> &Expr {
    loop {
        expr = match expr {
            Expr::Paren(paren) => &paren.expr,
            Expr::Group(group) => &group.expr,
            Expr::Block(block) if block.block.stmts.len() == 1 => {
                let Stmt::Expr(tail, None) = &block.block.stmts[0] else {
                    return expr;
                };
                tail
            }
            _ => return expr,
        };
    }
}

/// Parse just the message expression, leaving formatting arguments opaque.
fn argument(tokens: TokenStream, index: usize) -> Option<Expr> {
    // no-slop-allow: no-silent-error-discard -- unsupported macro arguments are outside this syntax rule
    (|input: ParseStream<'_>| {
        for _ in 0..index {
            input.parse::<Expr>()?;
            input.parse::<Token![,]>()?;
        }
        let message = input.parse::<Expr>()?;
        if !input.is_empty() {
            input.parse::<Token![,]>()?;
            input.parse::<TokenStream>()?;
        }
        Ok(message)
    })
    .parse2(tokens)
    .ok()
}

fn format_macro(mac: &Macro) -> bool {
    matches!(
        path_name(&mac.path).as_str(),
        "format" | "std::format" | "alloc::format"
    )
}

fn error_macro(mac: &Macro) -> Option<usize> {
    let name = path_name(&mac.path);
    let name = name
        .strip_prefix("anyhow::")
        .or_else(|| name.strip_prefix("eyre::"))
        .or_else(|| name.strip_prefix("color_eyre::eyre::"))
        .unwrap_or(&name);
    match name {
        "anyhow" | "eyre" | "bail" | "format_err" => Some(0),
        "ensure" => Some(1),
        _ => None,
    }
}

fn capitalized(message: &str, formatted: bool) -> bool {
    let mut chars = message.chars().peekable();
    while let Some(ch) = chars.next() {
        if formatted && ch == '{' {
            if chars.next_if_eq(&'{').is_some() {
                continue;
            }
            // The opening is determined by runtime output, not this field's name.
            return false;
        }
        if ch.is_lowercase() {
            return false;
        }
        if ch.to_lowercase().ne(std::iter::once(ch)) {
            return true;
        }
    }
    false
}

pub(crate) fn check(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    struct Check<'a> {
        context: &'a Context<'a>,
        findings: Vec<Diagnostic>,
    }

    impl Check<'_> {
        fn literal(&mut self, literal: &LitStr, formatted: bool) {
            if capitalized(&literal.value(), formatted) {
                self.findings.push(self.context.report(
                    "require-lowercase-error-messages", literal.span(),
                    "error message should begin with lowercase prose",
                    "Use a lowercase opening; preserve acronyms and identifiers later in the message, or explain a required opening with a local exception.",
                ));
            }
        }

        fn message(&mut self, expr: &Expr, formatted: bool) {
            match unwrapped(expr) {
                Expr::Lit(literal) => {
                    if let Lit::Str(literal) = &literal.lit {
                        self.literal(literal, formatted);
                    }
                }
                Expr::Macro(expr) if format_macro(&expr.mac) => {
                    if let Some(message) = argument(expr.mac.tokens.clone(), 0) {
                        self.message(&message, true);
                    }
                }
                Expr::MethodCall(call)
                    if call.args.is_empty()
                        && matches!(
                            call.method.to_string().as_str(),
                            "to_owned" | "to_string" | "into"
                        ) =>
                {
                    self.message(&call.receiver, formatted);
                }
                _ => {}
            }
        }

        fn lazy_message(&mut self, expr: &Expr) {
            if let Expr::Closure(closure) = unwrapped(expr) {
                if closure.inputs.is_empty() && closure.asyncness.is_none() {
                    self.message(&closure.body, false);
                }
            }
        }

        fn attributes(&mut self, attributes: &[Attribute]) {
            for attr in attributes
                .iter()
                .filter(|attr| attr.path().is_ident("error"))
            {
                if let syn::Meta::List(list) = &attr.meta {
                    if let Some(message) = argument(list.tokens.clone(), 0) {
                        // Transparent forms and custom expressions are not messages.
                        if let Expr::Lit(literal) = unwrapped(&message) {
                            if let Lit::Str(literal) = &literal.lit {
                                self.literal(literal, true);
                            }
                        }
                    }
                }
            }
        }
    }

    impl<'ast> Visit<'ast> for Check<'_> {
        fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
            self.attributes(&item.attrs);
            visit::visit_item_struct(self, item);
        }

        fn visit_variant(&mut self, variant: &'ast Variant) {
            self.attributes(&variant.attrs);
            visit::visit_variant(self, variant);
        }

        fn visit_macro(&mut self, mac: &'ast Macro) {
            if let Some(index) = error_macro(mac) {
                if let Some(message) = argument(mac.tokens.clone(), index) {
                    self.message(&message, true);
                }
            }
        }

        fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
            if call.args.len() == 1 {
                match call.method.to_string().as_str() {
                    "context" | "wrap_err" | "ok_or" => self.message(&call.args[0], false),
                    "with_context" | "wrap_err_with" | "ok_or_else" => {
                        self.lazy_message(&call.args[0])
                    }
                    _ => {}
                }
            }
            visit::visit_expr_method_call(self, call);
        }

        fn visit_expr_call(&mut self, call: &'ast ExprCall) {
            if let Expr::Path(path) = unwrapped(&call.func) {
                if path.qself.is_none() {
                    let name = path_name(&path.path);
                    let message = match name.as_str() {
                        "Err"
                        | "Result::Err"
                        | "std::result::Result::Err"
                        | "core::result::Result::Err"
                            if call.args.len() == 1 =>
                        {
                            call.args.first()
                        }
                        "Error::other" | "io::Error::other" | "std::io::Error::other"
                            if call.args.len() == 1 =>
                        {
                            call.args.first()
                        }
                        "Error::new" | "io::Error::new" | "std::io::Error::new"
                            if call.args.len() == 2 =>
                        {
                            call.args.get(1)
                        }
                        _ => None,
                    };
                    if let Some(message) = message {
                        self.message(message, false);
                    }
                }
            }
            visit::visit_expr_call(self, call);
        }
    }
    let mut check = Check {
        context,
        findings: Vec::new(),
    };
    check.visit_file(file);
    check.findings
}
