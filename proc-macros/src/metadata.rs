// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::main_crate_name;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn impl_metadata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let kebab_case_name = format!("{}", name.to_string().to_case(Case::Kebab));
    let generics = input.generics;
    let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    let crate_name = main_crate_name();
    TokenStream::from(quote! {
        #[automatically_derived]
        impl #generics #crate_name::traits::HasMetadata for #name #ty_generics {
            fn uid(&self) -> #crate_name::types::Uid {
                self.uid
            }

            fn set_uid(&mut self, uid: #crate_name::types::Uid) {
                self.uid = uid;
            }

            fn name(&self) -> &'static str {
                Self::ENTITY_NAME
            }

            fn key(&self) -> &'static str {
                Self::ENTITY_KEY
            }
        }

        #[automatically_derived]
        impl #name #ty_generics {
            pub const ENTITY_NAME: &'static str = stringify!(#name);
            pub const ENTITY_KEY: &'static str = #kebab_case_name;
        }
    })
}
