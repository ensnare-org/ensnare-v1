// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::cores::ToyControllerCore;
use eframe::egui::Slider;
use ensnare::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerControls, InnerHandlesMidi, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControls,
    InnerControllable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
pub struct ToyController {
    uid: Uid,
    #[serde(skip)]
    inner: ToyControllerCore,
}
impl Generates<StereoSample> for ToyController {}
impl Ticks for ToyController {}
impl TransformsAudio for ToyController {}
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
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: ToyControllerCore::new_with(MidiChannel::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    // use crate::cores::controllers::sequencers::tests::{validate_sequences_midi_trait, validate_sequences_notes_trait};

    // #[test]
    // fn toy_passes_sequences_trait_validation() {
    //     let mut s = ToySequencer::default();

    //     validate_sequences_midi_trait(&mut s);
    //     validate_sequences_notes_trait(&mut s);
    // }
}
