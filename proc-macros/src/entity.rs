// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::main_crate_name;
use proc_macro::TokenStream;
use quote::quote;
use rustc_hash::FxHashSet;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use syn::{parse_macro_input, Attribute, DeriveInput};

pub(crate) const ENTITY_ATTRIBUTE_NAME: &str = "entity";

// TODO: see
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=03943d1dfbf41bd63878bfccb1c64670
// for an intriguing bit of code. Came from
// https://users.rust-lang.org/t/is-implementing-a-derive-macro-for-converting-nested-structs-to-flat-structs-possible/65839/3

#[derive(Debug, EnumString, Display, Eq, PartialEq, Hash, EnumIter)]
#[strum(serialize_all = "PascalCase")]
enum Attributes {
    Configurable,
    Controllable,
    Controls,
    Displays,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    TransformsAudio,
    SkipInner,
}

// TODO: see
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=03943d1dfbf41bd63878bfccb1c64670
// for an intriguing bit of code. Came from
// https://users.rust-lang.org/t/is-implementing-a-derive-macro-for-converting-nested-structs-to-flat-structs-possible/65839/3

pub(crate) fn parse_and_generate_entity(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let parsed_attrs = parse_attrs(&input.attrs);
        let mut skip_inner = false;
        let top_level_trait_names = parsed_attrs.iter().fold(Vec::default(), |mut v, a| {
            match a {
                Attributes::Configurable => {
                    v.push(quote! {#crate_name::traits::Configurable});
                }
                Attributes::Controllable => {
                    v.push(quote! {#crate_name::automation::Controllable});
                }
                Attributes::Controls => {
                    v.push(quote! {#crate_name::automation::Controls});
                }
                Attributes::Displays => {
                    v.push(quote! {#crate_name::traits::Displays});
                }
                Attributes::GeneratesStereoSample => {
                    v.push(quote! {#crate_name::traits::Generates<StereoSample>});
                }
                Attributes::HandlesMidi => {
                    v.push(quote! {#crate_name::traits::HandlesMidi});
                }
                Attributes::Serializable => {
                    v.push(quote! {#crate_name::traits::Serializable});
                }
                Attributes::TransformsAudio => {
                    v.push(quote! {#crate_name::traits::TransformsAudio});
                }
                Attributes::SkipInner => {
                    skip_inner = true;
                }
            }
            v
        });

        let quote = quote! {
            #[automatically_derived]
            #[typetag::serde]
            impl #generics #crate_name::traits::Entity for #struct_name #ty_generics {}
            #( impl #generics #top_level_trait_names for #struct_name #ty_generics {} )*
        };
        quote
    })
}

fn parse_attrs(attrs: &[Attribute]) -> FxHashSet<Attributes> {
    let mut strs = Vec::default();

    attrs
        .iter()
        .filter(|attr| attr.path.is_ident(ENTITY_ATTRIBUTE_NAME))
        .for_each(|attr| {
            if let Ok(meta) = attr.parse_meta() {
                match meta {
                    syn::Meta::List(meta_list) => {
                        if meta_list.path.is_ident(ENTITY_ATTRIBUTE_NAME) {
                            strs.extend(parse_meta_list_attrs(&meta_list));
                        }
                    }
                    _ => {}
                }
            }
        });

    let mut parsed_attributes = FxHashSet::default();
    strs.iter().for_each(|s| {
        if let Ok(e) = Attributes::from_str(s) {
            parsed_attributes.insert(e);
        } else {
            let attribute_value_names = Attributes::iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            panic!(
                "Unrecognized attribute value: \"{s}\". Valid values are {}",
                attribute_value_names
            );
        }
    });
    parsed_attributes
}

fn parse_meta_list_attrs(meta_list: &syn::MetaList) -> Vec<String> {
    let mut values = Vec::default();
    for item in meta_list.nested.iter() {
        match item {
            syn::NestedMeta::Meta(meta) => match meta {
                syn::Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        values.push(ident.to_string());
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    values
}
