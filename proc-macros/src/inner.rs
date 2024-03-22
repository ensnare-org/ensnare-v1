// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::main_crate_name;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn impl_inner_configurable_derive(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let quote = quote! {
            #[automatically_derived]
            impl #generics #crate_name::traits::Configurable for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn sample_rate(&self) -> #crate_name::types::SampleRate;
                        fn update_sample_rate(&mut self, sample_rate: #crate_name::types::SampleRate);
                        fn tempo(&self) -> #crate_name::types::Tempo;
                        fn update_tempo(&mut self, tempo: #crate_name::types::Tempo);
                        fn time_signature(&self) -> #crate_name::types::TimeSignature;
                        fn update_time_signature(&mut self, time_signature: #crate_name::types::TimeSignature);
                    }
                }
            }
        };
        quote
    })
}

pub(crate) fn impl_derive_inner_controllable(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let quote = quote! {
            #[automatically_derived]
            impl #generics #crate_name::automation::Controllable for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn control_index_count(&self) -> usize;
                        fn control_index_for_name(&self, name: &str) -> Option<#crate_name::automation::ControlIndex>;
                        fn control_name_for_index(&self, index: ControlIndex) -> Option<String>;
                        fn control_set_param_by_name(&mut self, name: &str, value: #crate_name::automation::ControlValue);
                        fn control_set_param_by_index(&mut self, index: #crate_name::automation::ControlIndex, value: #crate_name::automation::ControlValue);
                    }
                }
            }
        };
        quote
    })
}

pub(crate) fn impl_derive_inner_controls(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let quote = quote! {
            #[automatically_derived]
            impl #generics #crate_name::automation::Controls for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn time_range(&self) -> Option<TimeRange>;
                        fn update_time_range(&mut self, time_range: &TimeRange);
                        fn work(&mut self, control_events_fn: &mut ControlEventsFn);
                        fn is_finished(&self) -> bool;
                        fn play(&mut self);
                        fn stop(&mut self);
                        fn skip_to_start(&mut self);
                        fn is_performing(&self) -> bool;
                    }
                }
            }
        };
        quote
    })
}

pub(crate) fn impl_derive_inner_effect(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let quote = quote! {
            #[automatically_derived]
            impl #generics #crate_name::traits::TransformsAudio for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample;
                    }
                }
            }
        };
        quote
    })
}

pub(crate) fn impl_derive_inner_handles_midi(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let quote = quote! {
            #[automatically_derived]
            impl #generics #crate_name::traits::HandlesMidi for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn handle_midi_message(
                            &mut self,
                            channel: MidiChannel,
                            message: MidiMessage,
                            midi_messages_fn: &mut MidiMessagesFn,
                        );
                        fn midi_note_label_metadata(&self) -> Option<#crate_name::traits::MidiNoteLabelMetadata>;
                    }
                }
            }
        };
        quote
    })
}

pub(crate) fn impl_derive_inner_instrument(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let quote = quote! {
            #[automatically_derived]
            impl #generics #crate_name::traits::Generates<StereoSample> for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn value(&self) -> StereoSample;
                        fn generate(&mut self, values: &mut [StereoSample]);
                    }
                }
            }

            #[automatically_derived]
            impl #generics #crate_name::traits::Ticks for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn tick(&mut self, tick_count: usize);
                    }
                }
            }
        };
        quote
    })
}

pub(crate) fn impl_inner_serializable_derive(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let quote = quote! {
            #[automatically_derived]
            impl #generics #crate_name::traits::Serializable for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn before_ser(&mut self);
                        fn after_deser(&mut self);
                    }
                }
            }
        };
        quote
    })
}

pub(crate) fn impl_inner_transforms_audio_derive(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let crate_name = main_crate_name();

        let quote = quote! {
            #[automatically_derived]
            impl #generics #crate_name::traits::TransformsAudio for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn transform_audio(&mut self, input_sample: StereoSample) -> StereoSample;
                        fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample;
                        fn transform_batch(&mut self, samples: &mut [StereoSample]);
                    }
                }
            }
        };
        quote
    })
}
