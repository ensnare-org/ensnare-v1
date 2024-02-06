// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::modulators::DcaWidget;
use crate::{
    egui::unfiled::{EnvelopeWidget, OscillatorWidget},
    prelude::*,
};
use eframe::egui::{CollapsingHeader, Slider, Widget};

#[derive(Debug)]
pub struct FmSynthWidget<'a> {
    uid: Uid,
    inner: &'a mut crate::cores::instruments::FmSynth,
}
impl<'a> FmSynthWidget<'a> {
    fn new(inner: &'a mut crate::cores::instruments::FmSynth, uid: Uid) -> Self {
        Self { uid, inner }
    }

    pub fn widget(
        inner: &'a mut crate::cores::instruments::FmSynth,
        controllable_uid: Uid,
    ) -> impl eframe::egui::Widget + '_ {
        move |ui: &mut eframe::egui::Ui| FmSynthWidget::new(inner, controllable_uid).ui(ui)
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

        let carrier_response = CollapsingHeader::new("Carrier")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                let carrier_response = ui.add(OscillatorWidget::widget(&mut self.inner.carrier));
                if carrier_response.changed() {
                    self.inner.notify_change_carrier();
                }
                let carrier_envelope_response =
                    ui.add(EnvelopeWidget::widget(&mut self.inner.carrier_envelope));
                if carrier_envelope_response.changed() {
                    self.inner.notify_change_carrier_envelope();
                }
                carrier_response | carrier_envelope_response
            })
            .body_response;

        let modulator_response = CollapsingHeader::new("Modulator")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                let modulator_response =
                    ui.add(OscillatorWidget::widget(&mut self.inner.modulator));
                if modulator_response.changed() {
                    self.inner.notify_change_modulator();
                }
                let modulator_envelope_response =
                    ui.add(EnvelopeWidget::widget(&mut self.inner.modulator_envelope));
                if modulator_envelope_response.changed() {
                    self.inner.notify_change_modulator_envelope();
                }
                modulator_response | modulator_envelope_response
            })
            .body_response;

        let dca_response = CollapsingHeader::new("DCA")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                let response = ui.add(DcaWidget::widget(&mut self.inner.dca, self.uid));
                if response.changed() {
                    self.inner.notify_change_dca();
                }
                response
            })
            .body_response;

        let mut response = depth_response | ratio_response | beta_response;
        if let Some(carrier) = carrier_response {
            response |= carrier;
        }
        if let Some(modulator) = modulator_response {
            response |= modulator;
        }
        if let Some(modulator) = dca_response {
            response |= modulator;
        }
        response
    }
}
