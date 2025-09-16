#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Fields, LitStr, parse_macro_input};

#[proc_macro_derive(DictKey)]
pub fn derive_dict_key(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let ident = &input.ident;

    let mut ident_names = vec![];

    match &input.data {
        Data::Enum(data) => {
            for variant in &data.variants {
                let ident = &variant.ident;
                let name = LitStr::new(&ident.to_string(), ident.span());
                ident_names.push(quote! { #name });

                match &variant.fields {
                    Fields::Unit => {}
                    _ => {
                        return Error::new(variant.span(), "DictKey can only be derived for unit variants")
                            .to_compile_error()
                            .into();
                    }
                }
            }
        }
        _ => {
            return Error::new(input.span(), "DictKey can only be derived for enums")
                .to_compile_error()
                .into();
        }
    }

    quote! {
        impl DictKey for #ident {
            const FIELDS: &'static [&'static str] = &[#(#ident_names),*];

            fn into_usize(self) -> usize {
                self as usize
            }
        }
    }
    .into()
}

#[proc_macro_derive(FromStr)]
pub fn derive_from_str(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let ident = &input.ident;

    let mut match_arms = vec![];

    match &input.data {
        Data::Enum(data) => {
            for variant in &data.variants {
                let ident = &variant.ident;
                let name = LitStr::new(&ident.to_string(), ident.span());
                match_arms.push(quote! { #name => Ok(Self::#ident), });

                match &variant.fields {
                    Fields::Unit => {}
                    _ => {
                        return Error::new(variant.span(), "FromStr can only be derived for unit variants")
                            .to_compile_error()
                            .into();
                    }
                }
            }
        }
        _ => {
            return Error::new(input.span(), "FromStr can only be derived for enums")
                .to_compile_error()
                .into();
        }
    }

    quote! {
        impl FromStr for #ident {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#match_arms)*
                    _ => Err(()),
                }
            }
        }
    }
    .into()
}
