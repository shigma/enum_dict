#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_derive(DictKey)]
pub fn derive_dict_key(input: TokenStream) -> TokenStream {
    derive_dict_key_inner(input.into()).into()
}

pub(crate) fn derive_dict_key_inner(input: TokenStream2) -> TokenStream2 {
    let input: syn::DeriveInput = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };
    let syn::Data::Enum(data) = &input.data else {
        return syn::Error::new(input.span(), "DictKey can only be derived for enums").to_compile_error();
    };

    let mut ident_names = TokenStream2::new();
    let mut errors = TokenStream2::new();
    for variant in &data.variants {
        let name = variant.ident.to_string();
        ident_names.extend(quote! { #name, });

        let syn::Fields::Unit = &variant.fields else {
            errors.extend(
                syn::Error::new(variant.span(), "DictKey can only be derived for unit variants").to_compile_error(),
            );
            continue;
        };
    }
    if !errors.is_empty() {
        return errors;
    }

    let ident = &input.ident;
    quote! {
        #[automatically_derived]
        impl DictKey for #ident {
            const VARIANTS: &'static [&'static str] = &[#ident_names];
            fn into_usize(self) -> usize {
                self as usize
            }
        }
    }
}

#[proc_macro_derive(FromStr)]
pub fn derive_from_str(input: TokenStream) -> TokenStream {
    derive_from_str_inner(input.into()).into()
}

pub(crate) fn derive_from_str_inner(input: TokenStream2) -> TokenStream2 {
    let input: syn::DeriveInput = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };
    let syn::Data::Enum(data) = &input.data else {
        return syn::Error::new(input.span(), "FromStr can only be derived for enums").to_compile_error();
    };

    let mut match_arms = TokenStream2::new();
    let mut errors = TokenStream2::new();
    for variant in &data.variants {
        let ident = &variant.ident;
        let name = variant.ident.to_string();
        match_arms.extend(quote! { #name => Ok(Self::#ident), });

        let syn::Fields::Unit = &variant.fields else {
            errors.extend(
                syn::Error::new(variant.span(), "FromStr can only be derived for unit variants").to_compile_error(),
            );
            continue;
        };
    }

    let ident = &input.ident;
    quote! {
        #[automatically_derived]
        impl FromStr for #ident {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #match_arms
                    _ => Err(()),
                }
            }
        }
    }
}
