// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::traits::Displays;
use eframe::egui::Slider;
use ensnare_core::{
    prelude::*,
    stuff::toys::{ToyControllerParams, ToyEffectParams, ToyInstrumentParams, ToySynthParams},
};
use ensnare_egui_widgets::{dca, drag_normal, envelope, oscillator, waveform};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerControls, InnerEffect, InnerHandlesMidi,
    InnerInstrument, InnerSerializable, IsController, IsEffect, IsInstrument, Metadata,
};

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControls,
    InnerControllable,
    InnerHandlesMidi,
    InnerSerializable,
    IsController,
    Metadata,
)]
pub struct ToyController {
    uid: Uid,
    inner: ensnare_core::stuff::toys::ToyController,
}
impl Displays for ToyController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut channel = self.inner.midi_channel_out.0;
        let slider_response = ui.add(Slider::new(&mut channel, 0..=15).text("MIDI out"));
        if slider_response.changed() {
            self.inner.midi_channel_out = MidiChannel(channel);
        }
        ui.end_row();
        slider_response | ui.checkbox(&mut self.inner.is_enabled, "Enabled")
    }
}
impl ToyController {
    pub fn new_with(uid: Uid, params: &ToyControllerParams, midi_channel_out: MidiChannel) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::toys::ToyController::new_with(&params, midi_channel_out),
        }
    }
}

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
    inner: ensnare_core::stuff::toys::ToyInstrument,
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
            inner: ensnare_core::stuff::toys::ToyInstrument::new_with(&params),
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
    inner: ensnare_core::stuff::toys::ToySynth,
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
            inner: ensnare_core::stuff::toys::ToySynth::new_with(params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct ToyEffect {
    uid: Uid,

    inner: ensnare_core::stuff::toys::ToyEffect,
}
impl Displays for ToyEffect {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(drag_normal(&mut self.inner.magnitude, "Magnitude: "))
    }
}
impl ToyEffect {
    pub fn new_with(uid: Uid, params: &ToyEffectParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::toys::ToyEffect::new_with(params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsController,
    Metadata,
)]
pub struct ToyControllerAlwaysSendsMidiMessage {
    uid: Uid,

    inner: ensnare_core::stuff::toys::ToyControllerAlwaysSendsMidiMessage,
}
impl Displays for ToyControllerAlwaysSendsMidiMessage {}
impl ToyControllerAlwaysSendsMidiMessage {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::toys::ToyControllerAlwaysSendsMidiMessage::default(),
        }
    }
}
