//! Report a logging macro directly followed by returning that failure from the same block.
//! The caller receives the error too, so one failure is reported more than once.
//! Only adjacent statements count; `map_err` closures and opaque macro bodies are not inspected.

use syn::{
    Block, Expr, File, Macro, Stmt,
    spanned::Spanned as _,
    visit::{self, Visit},
};

use super::{
    super::{context::Context, diagnostic::Diagnostic},
    error_support::unwrapped,
    path_name,
};

fn logging_macro(stmt: &Stmt) -> Option<&Macro> {
    let mac = match stmt {
        Stmt::Macro(stmt) => &stmt.mac,
        Stmt::Expr(Expr::Macro(expr), _) => &expr.mac,
        _ => return None,
    };
    matches!(
        path_name(&mac.path).as_str(),
        "error"
            | "warn"
            | "log::error"
            | "log::warn"
            | "tracing::error"
            | "tracing::warn"
            | "eprintln"
            | "std::eprintln"
    )
    .then_some(mac)
}

fn bail_macro(mac: &Macro) -> bool {
    matches!(
        path_name(&mac.path).as_str(),
        "bail" | "anyhow::bail" | "eyre::bail" | "color_eyre::eyre::bail"
    )
}

fn err_call(expr: &Expr) -> bool {
    let Expr::Call(call) = unwrapped(expr) else {
        return false;
    };
    let Expr::Path(path) = unwrapped(&call.func) else {
        return false;
    };
    path.qself.is_none()
        && matches!(
            path_name(&path.path).as_str(),
            "Err" | "Result::Err" | "std::result::Result::Err" | "core::result::Result::Err"
        )
}

fn returns_failure(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Macro(stmt) => bail_macro(&stmt.mac),
        Stmt::Expr(Expr::Macro(expr), _) => bail_macro(&expr.mac),
        Stmt::Expr(Expr::Return(expr), _) => expr.expr.as_deref().is_some_and(err_call),
        // With a semicolon, the value would be discarded rather than returned.
        Stmt::Expr(expr, None) => err_call(expr),
        _ => false,
    }
}

pub fn check(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    struct Check<'a> {
        context: &'a Context<'a>,
        findings: Vec<Diagnostic>,
    }

    impl<'ast> Visit<'ast> for Check<'_> {
        fn visit_block(&mut self, block: &'ast Block) {
            for pair in block.stmts.windows(2) {
                let Some(mac) = logging_macro(&pair[0]) else {
                    continue;
                };
                if returns_failure(&pair[1]) && !self.context.is_test(mac.path.span()) {
                    self.findings.push(self.context.report(
                        "no-log-and-return",
                        mac.path.span(),
                        "failure is logged and then returned, so it is reported more than once",
                        "Return the error with context and log it once where it is handled; explain a required boundary log with a local exception.",
                    ));
                }
            }

            visit::visit_block(self, block);
        }
    }
    let mut check = Check {
        context,
        findings: Vec::new(),
    };
    check.visit_file(file);
    check.findings
}
