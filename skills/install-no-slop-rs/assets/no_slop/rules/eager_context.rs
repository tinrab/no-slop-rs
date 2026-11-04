//! Report `format!` passed directly to `context` or `wrap_err`, which allocates even on success.
//! Receiver types are not resolved, so visible error values, which have no lazy method, are exempt.
//! The fix runs only when deferring the arguments cannot skip a side effect or capture `?`.

use syn::{
    Expr, ExprClosure, ExprMethodCall, File, Lit, Macro, Pat, Token,
    punctuated::Punctuated,
    spanned::Spanned as _,
    visit::{self, Visit},
};

use super::{
    super::{context::Context, diagnostic::Diagnostic},
    error_support::{format_macro, unwrapped},
    path_name,
};

fn lazy_method(eager: &str) -> Option<&'static str> {
    match eager {
        "context" => Some("with_context"),
        "wrap_err" => Some("wrap_err_with"),
        _ => None,
    }
}

fn binding(pattern: &Pat) -> Option<String> {
    match pattern {
        Pat::Ident(pattern) => Some(pattern.ident.to_string()),
        Pat::Type(pattern) => binding(&pattern.pat),
        _ => None,
    }
}

/// Values that are visibly errors rather than `Result` or `Option`.
fn error_value(receiver: &Expr, closure_bindings: &[String]) -> bool {
    match unwrapped(receiver) {
        Expr::Path(path) => path.path.get_ident().is_some_and(|ident| {
            let name = ident.to_string();
            matches!(
                name.as_str(),
                "self" | "e" | "err" | "error" | "source" | "cause" | "report"
            ) || closure_bindings.contains(&name)
        }),
        Expr::Macro(expr) => matches!(
            path_name(&expr.mac.path).as_str(),
            "anyhow"
                | "anyhow::anyhow"
                | "eyre"
                | "eyre::eyre"
                | "format_err"
                | "anyhow::format_err"
        ),
        Expr::Call(call) => match unwrapped(&call.func) {
            Expr::Path(path) => {
                let segments = &path.path.segments;
                segments.len() >= 2
                    && matches!(
                        segments[segments.len() - 2].ident.to_string().as_str(),
                        "Error" | "Report"
                    )
            }
            _ => false,
        },
        Expr::MethodCall(call) if lazy_method(&call.method.to_string()).is_some() => {
            error_value(&call.receiver, closure_bindings)
        }
        _ => false,
    }
}

fn deferrable(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(_) | Expr::Path(_) => true,
        Expr::Field(expr) => deferrable(&expr.base),
        Expr::Reference(expr) if expr.mutability.is_none() => deferrable(&expr.expr),
        Expr::Paren(expr) => deferrable(&expr.expr),
        Expr::Group(expr) => deferrable(&expr.expr),
        Expr::Assign(expr) => matches!(&*expr.left, Expr::Path(_)) && deferrable(&expr.right),
        Expr::MethodCall(call) => {
            call.args.is_empty()
                && matches!(
                    call.method.to_string().as_str(),
                    "as_path"
                        | "as_str"
                        | "clone"
                        | "display"
                        | "is_empty"
                        | "len"
                        | "to_owned"
                        | "to_string"
                        | "to_string_lossy"
                )
                && deferrable(&call.receiver)
        }
        _ => false,
    }
}

fn deferrable_arguments(mac: &Macro) -> bool {
    mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
        .is_ok_and(|arguments| {
            matches!(
                arguments.first(),
                Some(Expr::Lit(literal)) if matches!(literal.lit, Lit::Str(_))
            ) && arguments.iter().all(deferrable)
        })
}

pub fn check(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    struct Check<'a> {
        context: &'a Context<'a>,
        closure_bindings: Vec<String>,
        findings: Vec<Diagnostic>,
    }

    impl Check<'_> {
        fn call(&mut self, call: &ExprMethodCall) {
            let method = call.method.to_string();
            let Some(lazy) = lazy_method(&method) else {
                return;
            };
            if call.args.len() != 1
                || call.turbofish.is_some()
                || error_value(&call.receiver, &self.closure_bindings)
            {
                return;
            }

            let argument = &call.args[0];
            let Expr::Macro(expr) = unwrapped(argument) else {
                return;
            };
            if !format_macro(&expr.mac) {
                return;
            }

            let finding = self.context.report(
                "no-eager-error-context",
                call.method.span(),
                format!("`{method}` formats its message even when the operation succeeds"),
                "Use the lazy form, such as with_context(|| format!(...)), so the message is built only on failure.",
            );
            // A bare receiver is the likeliest place for a same-named custom method.
            let edit = matches!(argument, Expr::Macro(_))
                && matches!(
                    unwrapped(&call.receiver),
                    Expr::Call(_) | Expr::MethodCall(_) | Expr::Try(_) | Expr::Await(_)
                )
                && deferrable_arguments(&expr.mac);
            self.findings.push(if edit {
                let start = call.method.span().byte_range().start;
                let end = argument.span().byte_range().start;
                let between = &self.context.source[call.method.span().byte_range().end..end];
                finding.with_fix(start, end, format!("{lazy}{between}|| "))
            } else {
                finding
            });
        }
    }

    impl<'ast> Visit<'ast> for Check<'_> {
        fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
            self.call(call);
            visit::visit_expr_method_call(self, call);
        }

        fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
            let outer = self.closure_bindings.len();
            self.closure_bindings
                .extend(closure.inputs.iter().filter_map(binding));
            visit::visit_expr_closure(self, closure);
            self.closure_bindings.truncate(outer);
        }
    }
    let mut check = Check {
        context,
        closure_bindings: Vec::new(),
        findings: Vec::new(),
    };
    check.visit_file(file);
    check.findings
}
