// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_cores::toys::ToyInstrumentParams;
use ensnare_cores_egui::modulators::dca;
use ensnare_egui_widgets::{envelope, oscillator, waveform};
use ensnare_entity::traits::Displays;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsEntity2, Metadata,
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
    IsEntity2,
    Metadata,
    Serialize,
)]
#[entity2(Controls, TransformsAudio)]
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
            inner: ensnare_cores::toys::ToyInstrument::new_with(&ToyInstrumentParams::default()),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controls, GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct ToySynth {
    uid: Uid,
    inner: ensnare_cores::toys::ToySynth,
}
impl ToySynth {
    fn ui_waveform(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(waveform(&mut self.inner.waveform));
        if response.changed() {
            self.inner
                .inner
                .voices_mut()
                .for_each(|v| v.oscillator.set_waveform(self.inner.waveform));
        }
        response
    }

    fn ui_envelope(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui
            .scope(|ui| {
                let response = ui.add(envelope(&mut self.inner.envelope));
                response
            })
            .inner;
        if response.changed() {
            self.inner.inner.voices_mut().for_each(|v| {
                v.envelope.set_attack(self.inner.envelope.attack());
                v.envelope.set_decay(self.inner.envelope.decay());
                v.envelope.set_sustain(self.inner.envelope.sustain());
                v.envelope.set_release(self.inner.envelope.release());
            });
        }
        response
    }
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            ..Default::default()
        }
    }
}
impl Displays for ToySynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            let waveform_response = self.ui_waveform(ui);
            let envelope_response = self.ui_envelope(ui);
            waveform_response | envelope_response
        })
        .inner
    }
}
