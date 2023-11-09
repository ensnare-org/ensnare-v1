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
                fn sample_rate(&self) -> SampleRate {
                    self.inner.sample_rate()
                }

                fn update_sample_rate(&mut self, sample_rate: SampleRate) {
                    self.inner.update_sample_rate(sample_rate)
                }

                fn tempo(&self) -> Tempo {
                    self.inner.tempo()
                }

                fn update_tempo(&mut self, tempo: Tempo) {
                    self.inner.update_tempo(tempo)
                }

                fn time_signature(&self) -> TimeSignature {
                    self.inner.time_signature()
                }

                fn update_time_signature(&mut self, time_signature: TimeSignature) {
                    self.inner.update_time_signature(time_signature)
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
                fn control_index_count(&self) -> usize {
                    self.inner.control_index_count()
                }

                fn control_index_for_name(&self, name: &'static str) -> Option<#core_crate::control::ControlIndex> {
                    self.inner.control_index_for_name(name)
                }

                fn control_name_for_index(&self, index: ControlIndex) -> Option<String> {
                    self.inner.control_name_for_index(index)
                }

                fn control_set_param_by_name(&mut self, name: &'static str, value: #core_crate::control::ControlValue) {
                    self.inner.control_set_param_by_name(name, value)
                }

                fn control_set_param_by_index(&mut self, index: #core_crate::control::ControlIndex, value: #core_crate::control::ControlValue) {
                    self.inner.control_set_param_by_index(index, value)
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
                fn update_time(&mut self, range: &ViewRange) {
                    self.inner.update_time(range)
                }

                fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
                    self.inner.work(control_events_fn)
                }

                fn is_finished(&self) -> bool {
                    self.inner.is_finished()
                }

                fn play(&mut self) {
                    self.inner.play()
                }

                fn stop(&mut self) {
                    self.inner.stop()
                }

                fn skip_to_start(&mut self) {
                    self.inner.skip_to_start()
                }

                fn is_performing(&self) -> bool {
                    self.inner.is_performing()
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
                fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
                    self.inner.transform_channel(channel, input_sample)
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
                fn value(&self) -> StereoSample {
                    self.inner.value()
                }

                fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
                    self.inner.generate_batch_values(values)
                }
            }

            #[automatically_derived]
            impl #generics #core_crate::traits::Ticks for #struct_name #ty_generics {
                fn tick(&mut self, tick_count: usize) {
                    self.inner.tick(tick_count)
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
                fn before_ser(&mut self) {
                    self.inner.before_ser()
                }

                fn after_deser(&mut self) {
                    self.inner.after_deser()
                }
            }
        };
        quote
    })
}
