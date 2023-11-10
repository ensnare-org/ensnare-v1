// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! This crate provides macros that make Entity development easier.

use entity::{parse_and_generate_entity, EntityType};
use proc_macro::TokenStream;
use proc_macro_crate::crate_name;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::Ident;

mod control;
mod entity;
mod inner;
mod metadata;
mod params;

/// The `Metadata` macro derives the boilerplate necessary for the `HasMetadata`
/// trait. If a device needs to interoperate with Orchestrator, then it needs to
/// have a unique ID. Deriving with this macro makes that happen.
#[proc_macro_derive(Metadata)]
pub fn metadata_derive(input: TokenStream) -> TokenStream {
    metadata::impl_metadata(input)
}

/// The [Params] macro generates helper structs that are useful for handing
/// around bundles of arguments. If you have a struct Foo, then this macro
/// generates a FooParams struct containing all the fields annotated #[params].
/// It automatically converts fields whose types are #[derive(Params)] to Params
/// structs as well. So a `#[derive(Params)] struct Foo` with `#[params] bar:
/// Bar` will generate `struct FooParams` with `bar: BarParams`. It tries to
/// handle primitives automatically. If you have a type that doesn't need its
/// own XxxParams struct (e.g., a NewType(u32)), then you should annotate with
/// #[params(leaf=true)], and it will be treated as a primitive.
#[proc_macro_derive(Params, attributes(params))]
pub fn params_derive(input: TokenStream) -> TokenStream {
    params::impl_params_derive(input, &make_primitives())
}

/// Derives helper methods to access Entity traits associated with controllers.
#[proc_macro_derive(IsController)]
pub fn controller_derive(input: TokenStream) -> TokenStream {
    parse_and_generate_entity(input, EntityType::Controller)
}

/// Derives helper methods to access Entity traits associated with effects.
#[proc_macro_derive(IsEffect)]
pub fn effect_derive(input: TokenStream) -> TokenStream {
    parse_and_generate_entity(input, EntityType::Effect)
}

/// Derives helper methods to access Entity traits associated with instruments.
#[proc_macro_derive(IsInstrument)]
pub fn instrument_derive(input: TokenStream) -> TokenStream {
    parse_and_generate_entity(input, EntityType::Instrument)
}

/// Derives helper methods to access Entity traits associated with entities that
/// are both controllers and effects.
#[proc_macro_derive(IsControllerEffect)]
pub fn controller_effect_derive(input: TokenStream) -> TokenStream {
    parse_and_generate_entity(input, EntityType::ControllerEffect)
}

/// Derives helper methods to access Entity traits associated with entities that
/// are both controllers and instruments.
#[proc_macro_derive(IsControllerInstrument)]
pub fn controller_instrument_derive(input: TokenStream) -> TokenStream {
    parse_and_generate_entity(input, EntityType::ControllerInstrument)
}

/// field types that don't recurse further for #[derive(Control)] purposes.
fn make_primitives() -> HashSet<Ident> {
    vec![
        "BipolarNormal",
        "ControlValue",
        "FrequencyHz",
        "MusicalTime",
        "Normal",
        "ParameterType",
        "Ratio",
        "String",
        "Tempo",
        "VoiceCount",
        "Waveform",
        "bool",
        "char",
        "f32",
        "f64",
        "i128",
        "i16",
        "i32",
        "i64",
        "i8",
        "u128",
        "u16",
        "u32",
        "u64",
        "u8",
        "usize",
    ]
    .into_iter()
    .fold(HashSet::default(), |mut hs, e| {
        hs.insert(format_ident!("{}", e));
        hs
    })
}

/// The [Control] macro derives the code that allows automation (one entity's
/// output driving another entity's control). Annotate a field with
/// #[control(leaf=true)] if it is neither a primitive nor #[derive(Control)].
#[proc_macro_derive(Control, attributes(control))]
pub fn derive_control(input: TokenStream) -> TokenStream {
    control::impl_derive_control(input, &make_primitives())
}

/// Derives code that delegates the implementation of the [Configurable] trait
/// to an inner struct.
#[proc_macro_derive(InnerConfigurable)]
pub fn derive_inner_configurable(input: TokenStream) -> TokenStream {
    inner::impl_inner_configurable_derive(input)
}

/// Derives code that delegates the implementation of the [Controllable] trait
/// to an inner struct.
#[proc_macro_derive(InnerControllable)]
pub fn derive_inner_controllable(input: TokenStream) -> TokenStream {
    inner::impl_derive_inner_controllable(input)
}

/// Derives the code that delegates the implementation of the [Controls] trait
/// to an inner struct.
#[proc_macro_derive(InnerControls)]
pub fn derive_inner_controls(input: TokenStream) -> TokenStream {
    inner::impl_derive_inner_controls(input)
}

/// Derives the code that delegates the implementation of the [HandlesMidi]
/// trait to an inner struct.
#[proc_macro_derive(InnerHandlesMidi)]
pub fn derive_inner_handles_midi(input: TokenStream) -> TokenStream {
    inner::impl_derive_inner_handles_midi(input)
}

/// Derives the code that delegates the implementation of traits unique to
/// [IsEffect] to an inner struct.
#[proc_macro_derive(InnerEffect)]
pub fn derive_inner_effect(input: TokenStream) -> TokenStream {
    inner::impl_derive_inner_effect(input)
}

/// Derives the code that delegates the implementation of traits unique to
/// [IsInstrument] to an inner struct.
#[proc_macro_derive(InnerInstrument)]
pub fn derive_inner_instrument(input: TokenStream) -> TokenStream {
    inner::impl_derive_inner_instrument(input)
}

/// Derives code that delegates the implementation of the [Serializable] trait
/// to an inner struct.
#[proc_macro_derive(InnerSerializable)]
pub fn derive_inner_serializable(input: TokenStream) -> TokenStream {
    inner::impl_inner_serializable_derive(input)
}

/// Derives code that delegates the implementation of the [TransformsAudio] trait
/// to an inner struct.
#[proc_macro_derive(InnerTransformsAudio)]
pub fn derive_inner_transforms_audio(input: TokenStream) -> TokenStream {
    inner::impl_inner_transforms_audio_derive(input)
}

// Some of the code generated in these macros uses the ensnare-core crate, but
// that crate also uses this proc-macro lib. So we need to correct the reference
// to sometimes be just `crate`.
fn core_crate_name() -> String {
    const CRATE_NAME: &'static str = "ensnare-core"; // if you named it with dashes -- my-crate
    const CRATE_NAME_FOR_USE: &'static str = "ensnare_core"; // substitute underscores for dashes -- my_crate

    if let Ok(found_crate) = crate_name(CRATE_NAME) {
        match found_crate {
            proc_macro_crate::FoundCrate::Itself => {
                // We aren't importing the crate by name, so we must be it.
                quote!(crate).to_string()
            }
            proc_macro_crate::FoundCrate::Name(_) => {
                // We're importing the crate by name, which means we aren't the
                // crate.
                let ident = format_ident!("{}", CRATE_NAME_FOR_USE);
                quote!(#ident).to_string()
            }
        }
    } else {
        panic!("forgot to import {}", CRATE_NAME);
    }
}

// Some of the code generated in these macros uses the ensnare-entity crate, but
// that crate also uses this proc-macro lib. So we need to correct the reference
// to sometimes be just `crate`.
fn entity_crate_name() -> String {
    const CRATE_NAME: &'static str = "ensnare-entity"; // if you named it with dashes -- my-crate
    const CRATE_NAME_FOR_USE: &'static str = "ensnare_entity"; // substitute underscores for dashes -- my_crate

    if let Ok(found_crate) = crate_name(CRATE_NAME) {
        match found_crate {
            proc_macro_crate::FoundCrate::Itself => {
                // We aren't importing the crate by name, so we must be it.
                quote!(crate).to_string()
            }
            proc_macro_crate::FoundCrate::Name(_) => {
                // We're importing the crate by name, which means we aren't the
                // crate.
                let ident = format_ident!("{}", CRATE_NAME_FOR_USE);
                quote!(#ident).to_string()
            }
        }
    } else {
        panic!("forgot to import {}", CRATE_NAME);
    }
}
