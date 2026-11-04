//! Check vertical spacing between the statements of a block.
//! A change of purpose is not visible in syntax, so these rules only use structure: the length of a run, and whether a multi-line block just ended.
//! Test scopes are exempt, and macro bodies stay opaque.

use syn::{
    Block, Expr, File, Stmt, StmtMacro,
    spanned::Spanned,
    visit::{self, Visit},
};

use super::{
    super::{context::Context, diagnostic::Diagnostic},
    readable_spacing::{has_blank_line, separating_edit},
};

/// The shortest reported run; rare in well-spaced crates, and edit this to fit a project.
const DENSE_RUN: usize = 6;

type StatementCheck = fn(&[Stmt], &Context<'_>, &mut Vec<Diagnostic>);

struct Blocks<'a> {
    context: &'a Context<'a>,
    statement_check: StatementCheck,
    findings: Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for Blocks<'_> {
    fn visit_block(&mut self, block: &'ast Block) {
        let tested = block
            .stmts
            .first()
            .is_some_and(|statement| self.context.is_test(statement.span()));
        if !tested {
            (self.statement_check)(&block.stmts, self.context, &mut self.findings);
        }

        visit::visit_block(self, block);
    }
}

fn check_blocks(
    file: &File,
    context: &Context<'_>,
    statement_check: StatementCheck,
) -> Vec<Diagnostic> {
    let mut blocks = Blocks {
        context,
        statement_check,
        findings: Vec::new(),
    };
    blocks.visit_file(file);
    blocks.findings
}

/// Report each run once at its first statement, because only the author can tell where its purpose changes.
fn dense_runs(statements: &[Stmt], context: &Context<'_>, findings: &mut Vec<Diagnostic>) {
    let adjacent = |previous: &Stmt, current: &Stmt| {
        !matches!(previous, Stmt::Item(_))
            && !matches!(current, Stmt::Item(_))
            && !has_blank_line(
                context,
                previous.span().byte_range().end,
                current.span().byte_range().start,
            )
    };
    for run in statements.chunk_by(adjacent) {
        if run.len() >= DENSE_RUN {
            findings.push(context.report(
                "no-dense-statement-runs",
                run[0].span(),
                format!("{} statements run without a blank line", run.len()),
                "Insert blank lines where the purpose changes, or explain an intentionally uniform sequence with a local exception.",
            ));
        }
    }
}

fn is_multi_line_block(statement: &Stmt) -> bool {
    let Stmt::Expr(expression, _) = statement else {
        return false;
    };
    let span = statement.span();
    matches!(
        expression,
        Expr::If(_)
            | Expr::Match(_)
            | Expr::ForLoop(_)
            | Expr::While(_)
            | Expr::Loop(_)
            | Expr::Block(_)
            | Expr::Unsafe(_)
    ) && span.start().line != span.end().line
}

fn returns_value(statement: &Stmt) -> bool {
    matches!(
        statement,
        Stmt::Expr(_, None)
            | Stmt::Expr(Expr::Return(_), _)
            | Stmt::Macro(StmtMacro {
                semi_token: None,
                ..
            })
    )
}

fn blocks_without_blank(
    statements: &[Stmt],
    context: &Context<'_>,
    findings: &mut Vec<Diagnostic>,
) {
    for (index, pair) in statements.windows(2).enumerate() {
        let final_return = index + 2 == statements.len() && returns_value(&pair[1]);
        if final_return || !is_multi_line_block(&pair[0]) {
            continue;
        }

        let next = pair[1].span();
        let edit = separating_edit(
            context,
            pair[0].span().byte_range().end,
            next.byte_range().start,
        );
        if let Some((start, end, replacement)) = edit {
            findings.push(
                context
                    .report(
                        "require-blank-after-block",
                        next,
                        "missing blank line after multi-line block",
                        "Insert a blank line before this statement and its leading comments.",
                    )
                    .with_fix(start, end, replacement),
            );
        }
    }
}

pub fn check_runs(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    check_blocks(file, context, dense_runs)
}

pub fn check_block_endings(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    check_blocks(file, context, blocks_without_blank)
}
