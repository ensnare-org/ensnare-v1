// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{CollapsingHeader, Slider, Widget};
use ensnare_core::generators::Envelope;
use ensnare_egui_widgets::envelope;

/// Wraps a [FmSynthWidget] as a [Widget](eframe::egui::Widget).
pub fn fm_synth<'a>(inner: &'a mut ensnare_cores::FmSynth) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| FmSynthWidget::new(inner).ui(ui)
}

#[derive(Debug)]
pub struct FmSynthWidget<'a> {
    inner: &'a mut ensnare_cores::FmSynth,
}
impl<'a> FmSynthWidget<'a> {
    fn new(inner: &'a mut ensnare_cores::FmSynth) -> Self {
        Self { inner }
    }
}
impl<'a> eframe::egui::Widget for FmSynthWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
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
