// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_cores::toys::{ToyInstrumentParams, ToySynthParams};

use ensnare_egui::modulators::dca;
use ensnare_egui_widgets::{envelope, oscillator, waveform};
use ensnare_entity::traits::Displays;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsInstrument, Metadata,
};

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerInstrument,
    InnerHandlesMidi,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
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
    pub fn new_with(uid: Uid, params: &ToyInstrumentParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::toys::ToyInstrument::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
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
impl ToySynth {
    pub fn new_with(uid: Uid, params: &ToySynthParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::toys::ToySynth::new_with(params),
        }
    }
}
