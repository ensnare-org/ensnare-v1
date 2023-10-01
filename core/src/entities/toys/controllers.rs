// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{midi::prelude::*, prelude::*, traits::prelude::*};
use eframe::egui::Slider;
use ensnare_proc_macros::{Control, IsController, Params, Uid};
use serde::{Deserialize, Serialize};
use std::ops::Range;

enum TestControllerAction {
    Nothing,
    NoteOn,
    NoteOff,
}

/// An [IsController](ensnare_core::traits::IsController) that emits a MIDI
/// note-on event on each beat, and a note-off event on each half-beat.
#[derive(Debug, Default, Control, IsController, Params, Uid, Serialize, Deserialize)]
pub struct ToyController {
    uid: Uid,

    #[serde(skip)]
    midi_channel_out: MidiChannel,

    #[serde(skip)]
    is_enabled: bool,
    #[serde(skip)]
    is_playing: bool,
    #[serde(skip)]
    is_performing: bool,

    #[serde(skip)]
    time_range: std::ops::Range<MusicalTime>,

    #[serde(skip)]
    last_time_handled: MusicalTime,
}
impl Serializable for ToyController {}
impl Controls for ToyController {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.time_range = range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        match self.what_to_do() {
            TestControllerAction::Nothing => {}
            TestControllerAction::NoteOn => {
                // This is elegant, I hope. If the arpeggiator is
                // disabled during play, and we were playing a note,
                // then we still send the off note,
                if self.is_enabled && self.is_performing {
                    self.is_playing = true;
                    control_events_fn(
                        self.uid,
                        EntityEvent::Midi(self.midi_channel_out, new_note_on(60, 127)),
                    );
                }
            }
            TestControllerAction::NoteOff => {
                if self.is_playing {
                    control_events_fn(
                        self.uid,
                        EntityEvent::Midi(self.midi_channel_out, new_note_off(60, 0)),
                    );
                }
            }
        }
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn skip_to_start(&mut self) {}

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Configurable for ToyController {
    fn update_sample_rate(&mut self, _sample_rate: SampleRate) {}
}
impl HandlesMidi for ToyController {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        #[allow(unused_variables)]
        match message {
            MidiMessage::NoteOff { key, vel } => self.is_enabled = false,
            MidiMessage::NoteOn { key, vel } => self.is_enabled = true,
            _ => todo!(),
        }
    }
}
impl Displays for ToyController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("MIDI out: ");
        let mut channel = self.midi_channel_out.0;
        let slider_response = ui.add(Slider::new(&mut channel, 0..=16));
        if slider_response.changed() {
            self.midi_channel_out = MidiChannel(channel);
        }
        ui.end_row();
        slider_response | ui.checkbox(&mut self.is_enabled, "Enabled")
    }
}
impl ToyController {
    pub fn new_with(_params: &ToyControllerParams, midi_channel_out: MidiChannel) -> Self {
        Self {
            midi_channel_out,
            ..Default::default()
        }
    }

    fn what_to_do(&mut self) -> TestControllerAction {
        if !self.time_range.contains(&self.last_time_handled) {
            self.last_time_handled = self.time_range.start;
            if self.time_range.start.units() == 0 {
                if self.time_range.start.parts() == 0 {
                    return TestControllerAction::NoteOn;
                }
                if self.time_range.start.parts() == 8 {
                    return TestControllerAction::NoteOff;
                }
            }
        }
        TestControllerAction::Nothing
    }
}

#[derive(Debug, Default, Uid, IsController, Serialize, Deserialize)]
pub struct ToyControllerAlwaysSendsMidiMessage {
    uid: Uid,

    #[serde(skip)]
    midi_note: u8,

    #[serde(skip)]
    is_performing: bool,
}
impl Displays for ToyControllerAlwaysSendsMidiMessage {}
impl HandlesMidi for ToyControllerAlwaysSendsMidiMessage {}
impl Controls for ToyControllerAlwaysSendsMidiMessage {
    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.is_performing {
            control_events_fn(
                self.uid,
                EntityEvent::Midi(
                    MidiChannel(0),
                    MidiMessage::NoteOn {
                        key: u7::from(self.midi_note),
                        vel: u7::from(127),
                    },
                ),
            );
            self.midi_note += 1;
            if self.midi_note > 127 {
                self.midi_note = 1;
            }
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Configurable for ToyControllerAlwaysSendsMidiMessage {}
impl Serializable for ToyControllerAlwaysSendsMidiMessage {}
