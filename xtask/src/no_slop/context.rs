//! File-local information shared by independent rule functions.

use std::cmp::Reverse;
use std::ops::Range;
use std::path::Path;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Expr, File, ImplItem, ImplItemFn, Item, ItemFn, ItemMod, Macro, Meta, Stmt, Token,
    TraitItem,
};

use super::diagnostic::{Diagnostic, Level, Position};
use super::source::{self, Comment};

pub(crate) struct Context<'a> {
    pub path: &'a Path,
    pub source: &'a str,
    pub comments: Vec<Comment<'a>>,
    test_file: bool,
    test_ranges: Vec<Range<usize>>,
    pub targets: Vec<Range<usize>>,
    pub opaque_macros: Vec<Range<usize>>,
}

impl<'a> Context<'a> {
    pub fn new(path: &'a Path, source: &'a str, file: &File, test_file: bool) -> Self {
        let mut index = Index::default();
        index.visit_file(file);
        index
            .targets
            .sort_by_key(|range| (range.start, Reverse(range.end)));
        index.targets.dedup();
        Self {
            path,
            source,
            comments: source::comments(source),
            test_file: test_file || test_attributes(&file.attrs),
            test_ranges: index.test_ranges,
            targets: index.targets,
            opaque_macros: index.opaque_macros,
        }
    }

    pub fn is_test(&self, span: Span) -> bool {
        let offset = span.byte_range().start;
        self.test_file || self.test_ranges.iter().any(|range| range.contains(&offset))
    }

    pub fn position(&self, byte: usize) -> Position {
        let prefix = &self.source[..byte];
        Position {
            line: prefix.bytes().filter(|&b| b == b'\n').count() + 1,
            column: prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1,
            byte,
        }
    }

    pub fn report(
        &self,
        rule: &'static str,
        span: Span,
        message: impl Into<String>,
        help: &'static str,
    ) -> Diagnostic {
        self.report_range(rule, span.byte_range(), message, help)
    }

    pub fn report_range(
        &self,
        rule: &'static str,
        range: Range<usize>,
        message: impl Into<String>,
        help: &'static str,
    ) -> Diagnostic {
        Diagnostic {
            rule,
            level: Level::Error,
            path: self.path.to_owned(),
            start: self.position(range.start),
            end: self.position(range.end),
            message: message.into(),
            help,
        }
    }
}

/// Conservative recognition: the condition must require `test`, not merely permit it.
fn requires_test(meta: &Meta) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident("test"),
        Meta::List(list) => {
            let Ok(parts) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                return false;
            };
            if list.path.is_ident("all") {
                parts.iter().any(requires_test)
            } else if list.path.is_ident("any") {
                !parts.is_empty() && parts.iter().all(requires_test)
            } else {
                false
            }
        }
        Meta::NameValue(_) => false,
    }
}

pub(crate) fn test_attributes(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("cfg") {
            attr.parse_args::<Meta>()
                .is_ok_and(|meta| requires_test(&meta))
        } else {
            attr.path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        }
    })
}

#[derive(Default)]
struct Index {
    test_ranges: Vec<Range<usize>>,
    targets: Vec<Range<usize>>,
    opaque_macros: Vec<Range<usize>>,
}

impl Index {
    fn record_test(&mut self, attrs: &[Attribute], span: Span) {
        if test_attributes(attrs) {
            self.test_ranges.push(span.byte_range());
        }
    }
}

impl<'ast> Visit<'ast> for Index {
    fn visit_macro(&mut self, mac: &'ast Macro) {
        self.opaque_macros.push(mac.span().byte_range());
        visit::visit_macro(self, mac);
    }

    fn visit_item(&mut self, item: &'ast Item) {
        self.targets.push(item.span().byte_range());
        visit::visit_item(self, item);
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        self.targets.push(stmt.span().byte_range());
        visit::visit_stmt(self, stmt);
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        self.targets.push(item.span().byte_range());
        visit::visit_impl_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        self.targets.push(item.span().byte_range());
        visit::visit_trait_item(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        self.record_test(&item.attrs, item.span());
        visit::visit_item_mod(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.record_test(&item.attrs, item.span());
        visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        self.record_test(&item.attrs, item.span());
        visit::visit_impl_item_fn(self, item);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        // Block expressions may themselves be conditionally test-only.
        if let Expr::Block(block) = expr {
            self.record_test(&block.attrs, block.span());
        }
        visit::visit_expr(self, expr);
    }
}
