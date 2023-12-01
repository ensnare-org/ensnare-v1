// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::core_crate_name;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn impl_inner_configurable_derive(input: TokenStream) -> TokenStream {
    TokenStream::from({
        let input = parse_macro_input!(input as DeriveInput);
        let generics = &input.generics;
        let struct_name = &input.ident;
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
        let core_crate = format_ident!("{}", core_crate_name());

        let quote = quote! {
            #[automatically_derived]
            impl #generics #core_crate::traits::Configurable for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn sample_rate(&self) -> #core_crate::time::SampleRate;
                        fn update_sample_rate(&mut self, sample_rate: #core_crate::time::SampleRate);
                        fn tempo(&self) -> #core_crate::time::Tempo;
                        fn update_tempo(&mut self, tempo: #core_crate::time::Tempo);
                        fn time_signature(&self) -> #core_crate::time::TimeSignature;
                        fn update_time_signature(&mut self, time_signature: #core_crate::time::TimeSignature);
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
        let core_crate = format_ident!("{}", core_crate_name());

        let quote = quote! {
            #[automatically_derived]
            impl #generics #core_crate::traits::Controllable for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn control_index_count(&self) -> usize;
                        fn control_index_for_name(&self, name: &'static str) -> Option<#core_crate::control::ControlIndex>;
                        fn control_name_for_index(&self, index: ControlIndex) -> Option<String>;
                        fn control_set_param_by_name(&mut self, name: &'static str, value: #core_crate::control::ControlValue);
                        fn control_set_param_by_index(&mut self, index: #core_crate::control::ControlIndex, value: #core_crate::control::ControlValue);
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
        let core_crate = format_ident!("{}", core_crate_name());

        let quote = quote! {
            #[automatically_derived]
            impl #generics #core_crate::traits::Controls for #struct_name #ty_generics {
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
        let core_crate = format_ident!("{}", core_crate_name());

        let quote = quote! {
            #[automatically_derived]
            impl #generics #core_crate::traits::TransformsAudio for #struct_name #ty_generics {
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
        let core_crate = format_ident!("{}", core_crate_name());

        let quote = quote! {
            #[automatically_derived]
            impl #generics #core_crate::traits::HandlesMidi for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn handle_midi_message(
                            &mut self,
                            channel: MidiChannel,
                            message: MidiMessage,
                            midi_messages_fn: &mut MidiMessagesFn,
                        );
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
        let core_crate = format_ident!("{}", core_crate_name());

        let quote = quote! {
            #[automatically_derived]
            impl #generics #core_crate::traits::Generates<StereoSample> for #struct_name #ty_generics {
                delegate::delegate! {
                    to self.inner {
                        fn value(&self) -> StereoSample;
                        fn generate_batch_values(&mut self, values: &mut [StereoSample]);
                    }
                }
            }

            #[automatically_derived]
            impl #generics #core_crate::traits::Ticks for #struct_name #ty_generics {
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
        let core_crate = format_ident!("{}", core_crate_name());

        let quote = quote! {
            #[automatically_derived]
            impl #generics #core_crate::traits::Serializable for #struct_name #ty_generics {
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
        let core_crate = format_ident!("{}", core_crate_name());

        let quote = quote! {
            #[automatically_derived]
            impl #generics #core_crate::traits::TransformsAudio for #struct_name #ty_generics {
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
