use std::borrow::Borrow;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments,
    Attribute, GenericArgument, ItemFn, LitStr, Meta, MetaNameValue, PathArguments, ReturnType,
    Token, Type, TypePath, TypeTuple,
};

/// This macro automatically adds tests marked with #[test] to the test collection.
/// Tests then can be run with libtest_mimic_collect::TestCollection::run().
#[proc_macro_attribute]
pub fn test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        sig, block, attrs, ..
    } = parse_macro_input!(input as ItemFn);

    let ident = &sig.ident;
    let test_name = ident.to_string();
    let test_name_str = LitStr::new(&test_name, Span::call_site());
    let ctor_name = format!("__{}_add_test", test_name);
    let ctor_ident = Ident::new(&ctor_name, Span::call_site());

    let ret_type_unit = quote! { Result<(), ::libtest_mimic_collect::libtest_mimic::Failed> };
    let ret_type_completion = quote! { Result<::libtest_mimic_collect::libtest_mimic::Completion, ::libtest_mimic_collect::libtest_mimic::Failed> };

    let trial = match &sig.output {
        ReturnType::Default => {
            quote! {
                ::libtest_mimic_collect::libtest_mimic::Trial::test(#test_name_str, || -> #ret_type_unit {
                    #ident();
                    Ok(())
                })
            }
        }
        ReturnType::Type(_, ty) => {
            let result_segment = if let Type::Path(TypePath { path, qself: None }) = ty.as_ref() {
                path.segments
                    .last()
                    .filter(|segment| segment.ident == "Result")
            } else {
                None
            };

            match result_segment {
                Some(segment) => {
                    let is_unit_result = match &segment.arguments {
                        PathArguments::None => false,
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            args,
                            ..
                        }) => {
                            matches!(
                                args.first(),
                                Some(GenericArgument::Type(Type::Tuple(TypeTuple { elems, .. }))) if elems.is_empty()
                            )
                        }
                        PathArguments::Parenthesized(args) => {
                            return syn::Error::new(args.span(), "unexpected return type")
                                .to_compile_error()
                                .into();
                        }
                    };

                    if is_unit_result {
                        quote! {
                            ::libtest_mimic_collect::libtest_mimic::Trial::test(#test_name_str, || -> #ret_type_unit {
                                Ok(#ident()?.into())
                            })
                        }
                    } else {
                        quote! {
                            ::libtest_mimic_collect::libtest_mimic::Trial::ignorable_test(#test_name_str, || -> #ret_type_completion {
                                Ok(#ident()?.into())
                            })
                        }
                    }
                }
                None => {
                    quote! {
                        ::libtest_mimic_collect::libtest_mimic::Trial::test(#test_name_str, || -> #ret_type_unit {
                            ::libtest_mimic_collect::TestCollection::convert_result(#ident())
                        })
                    }
                }
            }
        }
    };

    // If there was an #[ignore] (or #[cfg_attr(...)] which evaluates to #[ignore], then map it to
    // a Trial::with_ignored_flag() that expresses the same constraint.
    let trial = match ignore_attrs(&attrs) {
        Ok(Some(is_ignored)) => quote! { #trial.with_ignored_flag(#is_ignored) },
        Ok(None) => trial,
        Err(err) => return err.to_compile_error().into(),
    };

    (quote! {
        #sig #block

        #[::libtest_mimic_collect::ctor]
        fn #ctor_ident() {
            ::libtest_mimic_collect::TestCollection::add_test(#trial);
        }
    })
    .into()
}

/// Builds the expression passed to `Trial::with_ignored_flag` from the attributes on a test
/// function, or `None` if the test has no `#[ignore]`-triggering attributes.
///
/// The predicate of a `#[cfg_attr(..., ignore)]` cannot be evaluated inside a proc-macro, so it is
/// mapped to a runtime `cfg!(any(all(...), ...))` check instead. `#[ignore]` is unconditional but
/// is essentially treated as `#[cfg_attr(all(), ignore)]` to make things simpler.
fn ignore_attrs(attrs: &[Attribute]) -> syn::Result<Option<TokenStream2>> {
    let chains = attrs
        .iter()
        .map(|attr| ignore_attr_chains(&attr.meta))
        // collate errors
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        // drop Ok(None)s
        .flatten()
        .collect::<Vec<_>>();

    // TODO: Can we unify this with the same chain-collect-filter logic in ignore_attr_chains?

    if chains.is_empty() {
        // No chains found, emit nothing.
        return Ok(None);
    }
    Ok(Some(quote! { ::core::cfg!(any( #(#chains),* )) }))
}

/// Recursively walks an attribute's [`Meta`], returning the equivalent set of [`cfg!`][]
/// predicates that are necessary for this `Meta` to apply an `#[ignore]` attribute.
///
/// [`cfg!`]: core::cfg
fn ignore_attr_chains(meta: impl Borrow<Meta>) -> syn::Result<Option<TokenStream2>> {
    match meta.borrow() {
        // #[ignore] or #[ignore = "reason"] (libtest-mimic cannot handle reason strings).
        Meta::Path(path) | Meta::NameValue(MetaNameValue { path, .. })
            if path.is_ident("ignore") =>
        {
            // cfg!(all()) is always true -- equivalent to #[cfg_attr()].
            Ok(Some(quote! { all() }))
        }
        // #[cfg_attr(..., ..., ignore, ...)]
        Meta::List(list) if list.path.is_ident("cfg_attr") => {
            // Split out the cfg_attr predicate and attributes.
            let (predicate, attrs) = {
                let mut cfg_attr = list
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                    .into_iter();
                (
                    // cfg_attr requires a predicate (though this error is never hit because the
                    // compiler rejects such programs before proc-macros get executed).
                    cfg_attr.next().ok_or_else(|| {
                        syn::Error::new_spanned(list, "cfg_attr is missing a predicate")
                    })?,
                    // Rest of the meta iterator.
                    cfg_attr,
                )
            };

            let chains = attrs
                .map(ignore_attr_chains)
                // collate errors
                .collect::<syn::Result<Vec<_>>>()?
                .into_iter()
                // drop Ok(None)s
                .flatten()
                .collect::<Vec<_>>();

            if chains.is_empty() {
                // No chains found, prune this branch.
                return Ok(None);
            }
            Ok(Some(quote! { all( #predicate, any( #(#chains),* ) ) }))
        }
        _ => Ok(None),
    }
}
