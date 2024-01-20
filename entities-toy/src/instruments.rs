// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    generators::{Envelope, Oscillator},
    modulators::Dca,
    prelude::*,
};
use ensnare_cores_egui::modulators::dca;
use ensnare_egui_widgets::{envelope, oscillator};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct ToyInstrument {
    uid: Uid,
    inner: ensnare_cores::toys::ToyInstrument,
}
impl Displays for ToyInstrument {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(oscillator(&mut self.inner.oscillator)) | ui.add(dca(&mut self.inner.dca, self.uid))
    }
}
impl ToyInstrument {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_cores::toys::ToyInstrument::new(),
        }
    }
}

#[derive(
    Debug,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct ToySynth {
    uid: Uid,
    inner: ensnare_cores::toys::ToySynth,
}
impl ToySynth {
    fn ui_oscillator(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(oscillator(&mut self.inner.oscillator));
        if response.changed() {
            // make sure everyone knows
        }
        response
    }

    fn ui_envelope(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(envelope(&mut self.inner.envelope));
        if response.changed() {
            // make sure everyone knows
        }
        response
    }
    pub fn new_with(uid: Uid, oscillator: Oscillator, envelope: Envelope, dca: Dca) -> Self {
        Self {
            uid,
            inner: ensnare_cores::toys::ToySynth::new_with(oscillator, envelope, dca),
        }
    }
}
impl Displays for ToySynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            let oscillator_response = self.ui_oscillator(ui);
            let envelope_response = self.ui_envelope(ui);
            oscillator_response | envelope_response
        })
        .inner
    }
}
