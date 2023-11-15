// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{core_crate_name, entity_crate_name};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use syn::{parse_macro_input, Attribute, DeriveInput};

pub(crate) const ENTITY_ATTRIBUTE_NAME: &str = "entity";

#[derive(Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
enum Attributes {
    Controller,
    Effect,
    Instrument,
    Timeline,
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

        let mut displays_in_timeline = false;
        let mut is_controller = false;
        let mut is_effect = false;
        let mut is_instrument = false;
        let parsed_attrs = parse_attrs(&input.attrs);
        parsed_attrs.iter().for_each(|attr| match attr {
            Attributes::Controller => is_controller = true,
            Attributes::Effect => is_effect = true,
            Attributes::Instrument => is_instrument = true,
            Attributes::Timeline => displays_in_timeline = true,
        });

        let mut top_level_trait_names = Vec::default();
        if is_controller {
            top_level_trait_names.push(quote! {#entity_crate::traits::IsController});
        }
        if is_effect {
            top_level_trait_names.push(quote! {#entity_crate::traits::IsEffect});
        }
        if is_instrument {
            top_level_trait_names.push(quote! {#entity_crate::traits::IsInstrument});
        }

        let is_controller_items = if is_controller {
            quote! {
                fn as_controller(&self) -> Option<&dyn #entity_crate::traits::IsController> {
                    Some(self)
                }
                fn as_controller_mut(&mut self) -> Option<&mut dyn #entity_crate::traits::IsController> {
                    Some(self)
                }
            }
        } else {
            quote! {}
        };
        let is_effect_items = if is_effect {
            quote! {
                fn as_effect(&self) -> Option<&dyn #entity_crate::traits::IsEffect> {
                    Some(self)
                }
                fn as_effect_mut(&mut self) -> Option<&mut dyn #entity_crate::traits::IsEffect> {
                    Some(self)
                }
            }
        } else {
            quote! {}
        };
        let is_instrument_items = if is_instrument {
            quote! {
                fn as_instrument(&self) -> Option<&dyn #entity_crate::traits::IsInstrument> {
                    Some(self)
                }
                fn as_instrument_mut(&mut self) -> Option<&mut dyn #entity_crate::traits::IsInstrument> {
                    Some(self)
                }
            }
        } else {
            quote! {}
        };
        let displays_in_timeline_items = if displays_in_timeline {
            quote! {
                fn as_displays_in_timeline_mut(&self) -> Option<&dyn #entity_crate::traits::DisplaysInTimeline> {
                    Some(self)
                }
            }
        } else {
            quote! {}
        };

        let handles_midi_items = if is_controller || is_instrument {
            quote! {
                fn as_handles_midi(&self) -> Option<&dyn ensnare_core::traits::HandlesMidi> {
                    Some(self)
                }
                fn as_handles_midi_mut(&mut self) -> Option<&mut dyn ensnare_core::traits::HandlesMidi> {
                    Some(self)
                }
            }
        } else {
            quote! {}
        };
        let controllable_items = if is_effect || is_instrument {
            quote! {
                fn as_controllable(&self) -> Option<&dyn #core_crate::traits::Controllable> {
                    Some(self)
                }
                fn as_controllable_mut(&mut self) -> Option<&mut dyn #core_crate::traits::Controllable> {
                    Some(self)
                }
            }
        } else {
            quote! {}
        };

        let quote = quote! {
            #[automatically_derived]
            #( impl #generics #top_level_trait_names for #struct_name #ty_generics {} )*

            #[automatically_derived]
            impl #generics #entity_crate::traits::Entity for #struct_name #ty_generics {
                #is_controller_items
                #is_effect_items
                #is_instrument_items
                #displays_in_timeline_items
                #handles_midi_items
                #controllable_items
            }
        };
        quote
    })
}

fn parse_attrs(attrs: &[Attribute]) -> Vec<Attributes> {
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

    let mut parsed_attributes = Vec::default();
    strs.iter().for_each(|s| {
        if let Ok(e) = Attributes::from_str(s) {
            parsed_attributes.push(e);
        }
    });
    parsed_attributes
}

fn parse_meta_list_attrs(meta_list: &syn::MetaList) -> Vec<String> {
    let mut values = Vec::default();
    for item in meta_list.nested.iter() {
        match item {
            syn::NestedMeta::Lit(literal) => match literal {
                syn::Lit::Str(litstr) => values.push(litstr.value()),
                _ => {}
            },
            _ => {}
        }
    }
    values
}
