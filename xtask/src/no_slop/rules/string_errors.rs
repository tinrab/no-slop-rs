//! Inspect explicit Result return contracts and type aliases, including nested Result types.
//! Imports, user-defined Result/String types, and aliases are not resolved.
//! Test-only contracts are exempt.
//! `Box<dyn Error>` is also accepted from `main` and trait implementations, whose signatures are fixed.

use std::mem;

use syn::{
    File, GenericArgument, ItemImpl, ItemType, PathArguments, ReturnType, Signature, Type,
    TypeParamBound, TypePath,
    spanned::Spanned as _,
    visit::{self, Visit},
};

use super::super::{context::Context, diagnostic::Diagnostic};

pub fn check(file: &File, context: &Context<'_>) -> Vec<Diagnostic> {
    struct Check<'a> {
        context: &'a Context<'a>,
        in_trait_impl: bool,
        findings: Vec<Diagnostic>,
    }

    impl Check<'_> {
        fn contract(&mut self, ty: &Type, allow_boxed: bool) {
            struct Errors<'a, 'ctx> {
                check: &'a mut Check<'ctx>,
                allow_boxed: bool,
            }

            impl<'ast> Visit<'ast> for Errors<'_, '_> {
                fn visit_type_path(&mut self, ty: &'ast TypePath) {
                    if let Some(error) = opaque_error(ty, self.allow_boxed) {
                        self.check.findings.push(self.check.context.report(
                            "no-string-errors", error.span(), "error contract loses the failure's meaning",
                            "Return a named error type or a suitable standard error; use anyhow for coordination and reporting.",
                        ));
                    }

                    visit::visit_type_path(self, ty);
                }
            }
            Errors {
                check: self,
                allow_boxed,
            }
            .visit_type(ty);
        }
    }

    impl<'ast> Visit<'ast> for Check<'_> {
        fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
            let outer = mem::replace(&mut self.in_trait_impl, item.trait_.is_some());
            visit::visit_item_impl(self, item);
            self.in_trait_impl = outer;
        }

        fn visit_signature(&mut self, signature: &'ast Signature) {
            if let (false, ReturnType::Type(_, ty)) = (
                self.context.is_test(signature.ident.span()),
                &signature.output,
            ) {
                self.contract(ty, self.in_trait_impl || signature.ident == "main");
            }

            visit::visit_signature(self, signature);
        }

        fn visit_item_type(&mut self, alias: &'ast ItemType) {
            if !self.context.is_test(alias.ident.span()) {
                self.contract(&alias.ty, false);
            }

            visit::visit_item_type(self, alias);
        }
    }
    let mut check = Check {
        context,
        in_trait_impl: false,
        findings: Vec::new(),
    };
    check.visit_file(file);
    check.findings
}

fn boxed_error(ty: &TypePath) -> bool {
    if !matches!(
        super::path_name(&ty.path).as_str(),
        "Box" | "std::boxed::Box" | "alloc::boxed::Box"
    ) {
        return false;
    }

    let Some(PathArguments::AngleBracketed(args)) =
        ty.path.segments.last().map(|segment| &segment.arguments)
    else {
        return false;
    };
    let Some(GenericArgument::Type(Type::TraitObject(object))) = args.args.first() else {
        return false;
    };
    object.bounds.iter().any(|bound| {
        matches!(bound, TypeParamBound::Trait(bound) if matches!(
            super::path_name(&bound.path).as_str(),
            "Error" | "std::error::Error" | "core::error::Error"
        ))
    })
}

fn opaque_error(ty: &TypePath, allow_boxed: bool) -> Option<&Type> {
    if !matches!(
        super::path_name(&ty.path).as_str(),
        "Result" | "std::result::Result" | "core::result::Result"
    ) {
        return None;
    }

    let segment = ty.path.segments.last()?;
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(error)) = args.args.iter().nth(1) else {
        return None;
    };
    let opaque = match error {
        Type::Path(path) => {
            matches!(
                super::path_name(&path.path).as_str(),
                "String" | "std::string::String" | "alloc::string::String"
            ) || (!allow_boxed && boxed_error(path))
        }
        Type::Reference(reference) => {
            matches!(&*reference.elem, Type::Path(path) if path.path.is_ident("str"))
        }
        _ => false,
    };
    opaque.then_some(error)
}
