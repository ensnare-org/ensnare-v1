// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{core_crate_name, entity_crate_name};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::{collections::HashSet, str::FromStr};
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
    Ticks,
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
        let core_crate = format_ident!("{}", core_crate_name());
        let entity_crate = format_ident!("{}", entity_crate_name());

        let parsed_attrs = parse_attrs(&input.attrs);
        let mut skip_inner = false;
        let top_level_trait_names = parsed_attrs.iter().fold(Vec::default(), |mut v, a| {
            match a {
                Attributes::Configurable => {
                    v.push(quote! {#core_crate::traits::Configurable});
                }
                Attributes::Controllable => {
                    v.push(quote! {#core_crate::traits::Controllable});
                }
                Attributes::Controls => {
                    v.push(quote! {#core_crate::traits::Controls});
                }
                Attributes::Displays => {
                    v.push(quote! {#entity_crate::traits::Displays});
                }
                Attributes::GeneratesStereoSample => {
                    v.push(quote! {#core_crate::traits::Generates<StereoSample>});
                }
                Attributes::HandlesMidi => {
                    v.push(quote! {#core_crate::traits::HandlesMidi});
                }
                Attributes::Serializable => {
                    v.push(quote! {#core_crate::traits::Serializable});
                }
                Attributes::Ticks => {
                    v.push(quote! {#core_crate::traits::Ticks});
                }
                Attributes::TransformsAudio => {
                    v.push(quote! {#core_crate::traits::TransformsAudio});
                }
                Attributes::SkipInner => {
                    skip_inner = true;
                }
            }
            v
        });
        let as_handles_midi_mut_impl = if skip_inner {
            quote! { self }
        } else {
            quote! { &mut self.inner }
        };

        let quote = quote! {
            #[automatically_derived]
            #[typetag::serde]
            impl #generics #entity_crate::traits::Entity for #struct_name #ty_generics {
                fn as_handles_midi_mut(&mut self) -> Option<&mut dyn HandlesMidi> {
                    Some(#as_handles_midi_mut_impl)
                }
            }
            #[typetag::serde]
            impl #generics #entity_crate::traits::EntityBounds for #struct_name #ty_generics {}
            #( impl #generics #top_level_trait_names for #struct_name #ty_generics {} )*
        };
        quote
    })
}

fn parse_attrs(attrs: &[Attribute]) -> HashSet<Attributes> {
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

    let mut parsed_attributes = HashSet::default();
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
