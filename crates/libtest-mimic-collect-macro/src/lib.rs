use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, GenericArgument, ItemFn, LitStr, PathArguments, ReturnType, Type, TypePath, TypeTuple, parse_macro_input, spanned::Spanned
};

/// This macro automatically adds tests marked with #[test] to the test collection.
/// Tests then can be run with libtest_mimic_collect::TestCollection::run().
#[proc_macro_attribute]
pub fn test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        sig, block, ..
    } = parse_macro_input!(input as ItemFn);

    let ident = &sig.ident;
    let test_name = ident.to_string();
    let test_name_str = LitStr::new(&test_name, Span::call_site());
    let ctor_name = format!("__{}_add_test", test_name);
    let ctor_ident = Ident::new(&ctor_name, Span::call_site());

    let ret_type_unit = quote! { Result<(), ::libtest_mimic_collect::libtest_mimic::Failed> };
    let ret_type_completion = quote! { Result<::libtest_mimic_collect::libtest_mimic::Completion, ::libtest_mimic_collect::libtest_mimic::Failed> };

    let trial = match &sig.output {
        // Case 1: Return type is () - call ident and return Ok(())
        ReturnType::Default => {
            quote! {
                ::libtest_mimic_collect::libtest_mimic::Trial::test(#test_name_str, || -> #ret_type_unit {
                    #ident();
                    Ok(())
                })
            }
        }
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(TypePath { path, qself: None }) => {
                let segment = path
                    .segments
                    .last()
                    .expect("Should have at least one segment");
                if segment.ident == "Result" {
                    // Case 2 & 3: Return type is Result<T, E>
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
                } else {
                    quote! {
                        ::libtest_mimic_collect::libtest_mimic::Trial::test(#test_name_str, || -> #ret_type_unit {
                            ::libtest_mimic_collect::TestCollection::convert_result(#ident())
                        })
                    }
                }
            }
            _ => {
                quote! {
                    ::libtest_mimic_collect::libtest_mimic::Trial::test(#test_name_str, || -> #ret_type_unit {
                        ::libtest_mimic_collect::TestCollection::convert_result(#ident())
                    });
                }
            }
        },
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
