// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! This crate provides macros that make Entity development easier.
//!
//! PRO TIP: use `cargo expand --lib entities` to see what's being generated

use entity::parse_and_generate_entity;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::Ident;

mod control;
mod entity;
mod inner;
mod metadata;

/// The `Metadata` macro derives the boilerplate necessary for the `HasMetadata`
/// trait. If a device needs to interoperate with Orchestrator, then it needs to
/// have a unique ID. Deriving with this macro makes that happen.
#[proc_macro_derive(Metadata)]
pub fn metadata_derive(input: TokenStream) -> TokenStream {
    metadata::impl_metadata(input)
}

/// Derives helper methods to access Entity traits.
///
/// Available struct-level attributes:
///
/// - `"skip_inner"`: Do not delegate trait methods to a field named "inner".
/// - `(various)`: implements an empty trait. For example, `HandlesMidi`
///   implements an empty [HandlesMidi] trait.
#[proc_macro_derive(IsEntity, attributes(entity))]
pub fn entity_derive(input: TokenStream) -> TokenStream {
    parse_and_generate_entity(input)
}

/// field types that don't recurse further for #[derive(Control)] and
/// #[derive(Params)] purposes.
fn make_primitives() -> HashSet<Ident> {
    vec![
        "BipolarNormal",
        "ControlValue",
        "FrequencyHz",
        "MusicalTime",
        "Normal",
        "ParameterType",
        "Ratio",
        "SampleRate",
        "Seconds",
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

// See https://github.com/bkchr/proc-macro-crate/issues/14, ModProg's solution
fn main_crate_name() -> proc_macro2::TokenStream {
    const MAIN_CRATE_NAME: &str = "ensnare";
    let name = match (
        proc_macro_crate::crate_name(MAIN_CRATE_NAME),
        std::env::var("CARGO_CRATE_NAME").as_deref(),
    ) {
        (Ok(proc_macro_crate::FoundCrate::Itself), Ok(MAIN_CRATE_NAME)) => quote!(crate),
        (Ok(proc_macro_crate::FoundCrate::Name(name)), _) => {
            let ident = Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        _ => {
            let n = format_ident!("{}", MAIN_CRATE_NAME);
            quote!(::#n)
        }
    };
    name
}
