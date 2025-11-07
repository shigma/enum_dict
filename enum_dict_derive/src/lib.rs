#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use crate::case::RenameRule;

mod case;

struct Argument {
    ident: syn::Ident,
    expr: Option<(syn::Token![=], syn::Expr)>,
}

impl Parse for Argument {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let expr = if input.peek(syn::Token![=]) {
            let eq_token: syn::Token![=] = input.parse()?;
            let expr: syn::Expr = input.parse()?;
            Some((eq_token, expr))
        } else {
            None
        };
        Ok(Argument { ident: name, expr })
    }
}

#[proc_macro_derive(DictKey, attributes(enum_dict))]
pub fn derive_dict_key(input: TokenStream) -> TokenStream {
    derive_dict_key_inner(input.into()).into()
}

pub(crate) fn derive_dict_key_inner(input: TokenStream2) -> TokenStream2 {
    let input: syn::DeriveInput = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };
    let syn::Data::Enum(data) = input.data else {
        return syn::Error::new(input.span(), "DictKey can only be derived for enums").to_compile_error();
    };

    let mut rename_all = RenameRule::None;
    let mut errors = TokenStream2::new();
    for attr in input.attrs {
        if !attr.path().is_ident("enum_dict") {
            continue;
        }
        let syn::Meta::List(meta_list) = attr.meta else {
            errors.extend(syn::Error::new(attr.span(), "expected #[enum_dict(...)]").to_compile_error());
            continue;
        };
        let args = match Punctuated::<Argument, syn::Token![,]>::parse_terminated.parse2(meta_list.tokens) {
            Ok(args) => args,
            Err(err) => {
                errors.extend(err.to_compile_error());
                continue;
            }
        };
        for arg in args {
            if arg.ident == "rename_all" {
                let Some((
                    _,
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }),
                )) = arg.expr
                else {
                    errors
                        .extend(syn::Error::new(arg.ident.span(), "expected rename_all = \"...\"").to_compile_error());
                    continue;
                };
                match RenameRule::from_str(&lit_str.value()) {
                    Ok(rule) => rename_all = rule,
                    Err(err) => errors.extend(syn::Error::new(lit_str.span(), err.to_string()).to_compile_error()),
                };
            } else {
                errors.extend(
                    syn::Error::new(arg.ident.span(), "unknown attribute for enum_dict derive").to_compile_error(),
                );
            }
        }
    }

    let mut ident_names = TokenStream2::new();
    let mut match_arms = TokenStream2::new();
    for variant in data.variants {
        let syn::Fields::Unit = &variant.fields else {
            errors.extend(
                syn::Error::new(variant.span(), "DictKey can only be derived for unit variants").to_compile_error(),
            );
            continue;
        };

        let ident = &variant.ident;
        let mut name = rename_all.apply(&ident.to_string());
        for attr in variant.attrs {
            if !attr.path().is_ident("enum_dict") {
                continue;
            }
            let syn::Meta::List(meta_list) = attr.meta else {
                errors.extend(syn::Error::new(attr.span(), "expected #[enum_dict(...)]").to_compile_error());
                continue;
            };
            let args = match Punctuated::<Argument, syn::Token![,]>::parse_terminated.parse2(meta_list.tokens) {
                Ok(args) => args,
                Err(err) => {
                    errors.extend(err.to_compile_error());
                    continue;
                }
            };
            for arg in args {
                if arg.ident == "rename" {
                    let Some((
                        _,
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }),
                    )) = arg.expr
                    else {
                        errors
                            .extend(syn::Error::new(arg.ident.span(), "expected rename = \"...\"").to_compile_error());
                        continue;
                    };
                    name = lit_str.value();
                } else {
                    errors.extend(
                        syn::Error::new(arg.ident.span(), "unknown attribute for enum_dict derive").to_compile_error(),
                    );
                }
            }
        }

        match_arms.extend(quote! { #name => Ok(Self::#ident), });
        ident_names.extend(quote! { #name, });
    }

    if !errors.is_empty() {
        return errors;
    }

    let ident = &input.ident;
    quote! {
        #[automatically_derived]
        impl ::enum_dict::DictKey for #ident {
            const VARIANTS: &'static [&'static str] = &[#ident_names];
            fn variant_index(self) -> usize {
                self as usize
            }
        }

        #[automatically_derived]
        impl ::std::str::FromStr for #ident {
            type Err = ();
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                match s {
                    #match_arms
                    _ => std::result::Result::Err(()),
                }
            }
        }
    }
}
