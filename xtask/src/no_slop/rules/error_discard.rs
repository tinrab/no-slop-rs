//! `.ok()` uses the standard Result convention; custom receivers are not inferred.
//! An intentional conversion to absence needs a local exception with its policy.
//! Arbitrary ignored expressions are outside this syntactic rule.

use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Stmt};

use super::super::context::Context;
use super::super::diagnostic::Diagnostic;

pub(crate) fn check(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    struct Check<'a> {
        context: &'a Context<'a>,
        findings: Vec<Diagnostic>,
    }

    impl<'ast> Visit<'ast> for Check<'_> {
        fn visit_stmt(&mut self, stmt: &'ast Stmt) {
            // Clippy's unused_result_ok handles bare discarded conversions.
            if let Stmt::Expr(expr, Some(_)) = stmt {
                let mut expr = expr;
                loop {
                    expr = match expr {
                        Expr::Paren(paren) => &paren.expr,
                        Expr::Group(group) => &group.expr,
                        _ => break,
                    };
                }
                if let Expr::MethodCall(call) = expr {
                    if call.method == "ok" && call.args.is_empty() {
                        // Still inspect the receiver: it can contain other error loss.
                        self.visit_expr(&call.receiver);
                        return;
                    }
                }
            }
            visit::visit_stmt(self, stmt);
        }

        fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
            let direct = call.method == "ok" && call.args.is_empty();
            let filter = matches!(call.method.to_string().as_str(), "filter_map" | "flat_map") &&
                call.args.iter().any(|arg| matches!(arg, syn::Expr::Path(path) if matches!(
                    super::path_name(&path.path).as_str(), "Result::ok" | "std::result::Result::ok" | "core::result::Result::ok"
                )));
            if direct || filter {
                self.findings.push(self.context.report(
                    "no-silent-error-discard", call.method.span(), "conversion to absence discards error information",
                    "Preserve the Result, handle the error, or explain intentional loss with a local exception.",
                ));
            }
            visit::visit_expr_method_call(self, call);
        }
    }
    let mut check = Check {
        context,
        findings: Vec::new(),
    };
    check.visit_file(file);
    check.findings
}
