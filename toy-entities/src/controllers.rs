// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::Slider;
use ensnare_core::{prelude::*, toys::ToyControllerParams};
use ensnare_entity::traits::Displays;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerControls, InnerHandlesMidi, InnerSerializable,
    IsController, Metadata,
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
    inner: ensnare_core::toys::ToyController,
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
            inner: ensnare_core::toys::ToyController::new_with(&params, midi_channel_out),
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

    inner: ensnare_core::toys::ToyControllerAlwaysSendsMidiMessage,
}
impl Displays for ToyControllerAlwaysSendsMidiMessage {}
impl ToyControllerAlwaysSendsMidiMessage {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_core::toys::ToyControllerAlwaysSendsMidiMessage::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    // use ensnare_cores::controllers::sequencers::tests::{validate_sequences_midi_trait, validate_sequences_notes_trait};

    // #[test]
    // fn toy_passes_sequences_trait_validation() {
    //     let mut s = ToySequencer::default();

    //     validate_sequences_midi_trait(&mut s);
    //     validate_sequences_notes_trait(&mut s);
    // }
}
