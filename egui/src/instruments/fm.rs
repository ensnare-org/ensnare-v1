// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{CollapsingHeader, Slider};
use ensnare_core::{generators::Envelope, prelude::*, stuff::fm::FmSynthParams};
use ensnare_egui_widgets::envelope;
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerHandlesMidi, InnerInstrument, InnerSerializable, IsInstrument,
    Metadata,
};

#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
pub struct FmSynth {
    uid: Uid,
    inner: ensnare_core::stuff::fm::FmSynth,
}
impl Displays for FmSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut depth = self.inner.depth().to_percentage();
        let depth_response = ui.add(
            Slider::new(&mut depth, 0.0..=100.0)
                .text("Depth")
                .suffix(" %")
                .fixed_decimals(2),
        );
        if depth_response.changed() {
            self.inner.set_depth((depth / 100.0).into());
        }
        let mut ratio = self.inner.ratio().0;
        let ratio_response = ui.add(
            Slider::new(&mut ratio, 0.1..=32.0)
                .text("Ratio")
                .fixed_decimals(1),
        );
        if ratio_response.changed() {
            self.inner.set_ratio(ratio.into());
        }
        let mut beta = self.inner.beta();
        let beta_response = ui.add(
            Slider::new(&mut beta, 0.0..=100.0)
                .text("Beta")
                .fixed_decimals(1),
        );
        if beta_response.changed() {
            self.inner.set_beta(beta);
        }

        CollapsingHeader::new("Carrier")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui.add(envelope(&mut self.inner.carrier_envelope)).changed() {
                    self.inner.inner_synth.voices_mut().for_each(|v| {
                        v.set_carrier_envelope(Envelope::new_with(
                            &self.inner.carrier_envelope.to_params(),
                        ));
                    });
                }
            });

        CollapsingHeader::new("Modulator")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(envelope(&mut self.inner.modulator_envelope))
                    .changed()
                {
                    self.inner.inner_synth.voices_mut().for_each(|v| {
                        v.set_modulator_envelope(Envelope::new_with(
                            &self.inner.modulator_envelope.to_params(),
                        ));
                    });
                }
            });

        depth_response | ratio_response | beta_response
    }
}
impl FmSynth {
    pub fn new_with(uid: Uid, params: &FmSynthParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::fm::FmSynth::new_with(params),
        }
    }
}
