// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{self, Ui};
use ensnare_core::{
    generators::{Oscillator, OscillatorParams, Waveform},
    midi::prelude::*,
    modulators::{Dca, DcaParams},
    prelude::*,
    traits::prelude::*,
};
use ensnare_proc_macros::{Control, IsController, IsEffect, IsInstrument, Params, Uid};
use serde::{Deserialize, Serialize};
use std::{
    ops::Range,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
pub struct ToyInstrumentEphemerals {
    sample: StereoSample,
    pub is_playing: bool,
    pub received_midi_message_count: Arc<Mutex<usize>>,
    pub debug_messages: Vec<MidiMessage>,
}

/// An [IsInstrument](ensnare::traits::IsInstrument) that uses a default
/// [Oscillator] to produce sound. Its "envelope" is just a boolean that responds
/// to MIDI NoteOn/NoteOff.
#[derive(Debug, Default, Control, IsInstrument, Params, Uid, Serialize, Deserialize)]
pub struct ToyInstrument {
    uid: Uid,

    oscillator: Oscillator,

    #[control]
    #[params]
    dca: Dca,

    #[serde(skip)]
    e: ToyInstrumentEphemerals,
}
impl Generates<StereoSample> for ToyInstrument {
    fn value(&self) -> StereoSample {
        self.e.sample
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        for value in values {
            self.tick(1);
            *value = self.value();
        }
    }
}
impl Configurable for ToyInstrument {
    fn sample_rate(&self) -> SampleRate {
        self.oscillator.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.oscillator.update_sample_rate(sample_rate);
    }
}
impl Ticks for ToyInstrument {
    fn tick(&mut self, tick_count: usize) {
        self.oscillator.tick(tick_count);
        self.e.sample = if self.e.is_playing {
            self.dca
                .transform_audio_to_stereo(Sample::from(self.oscillator.value()))
        } else {
            StereoSample::SILENCE
        };
    }
}
impl HandlesMidi for ToyInstrument {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        self.e.debug_messages.push(message);
        if let Ok(mut received_count) = self.e.received_midi_message_count.lock() {
            *received_count += 1;
        }

        match message {
            MidiMessage::NoteOn { key, vel: _ } => {
                self.e.is_playing = true;
                self.oscillator.set_frequency(key.into());
            }
            MidiMessage::NoteOff { key: _, vel: _ } => {
                self.e.is_playing = false;
            }
            _ => {}
        }
    }
}
impl Serializable for ToyInstrument {}
impl ToyInstrument {
    pub fn new_with(params: &ToyInstrumentParams) -> Self {
        Self {
            uid: Default::default(),
            oscillator: Oscillator::new_with(&OscillatorParams::default_with_waveform(
                Waveform::Sine,
            )),
            dca: Dca::new_with(&params.dca),
            e: Default::default(),
        }
    }

    // If this instrument is being used in an integration test, then
    // received_midi_message_count provides insight into whether messages are
    // arriving.
    pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
        &self.e.received_midi_message_count
    }
}
impl Displays for ToyInstrument {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label(self.name())
    }
}

/// An [IsEffect](ensnare::traits::IsEffect) that negates the input signal.
#[derive(Debug, Default, Control, IsEffect, Params, Uid, Serialize, Deserialize)]
pub struct ToyEffect {
    uid: Uid,

    #[serde(skip)]
    sample_rate: SampleRate,
}
impl TransformsAudio for ToyEffect {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        -input_sample
    }
}
impl Configurable for ToyEffect {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}
impl Serializable for ToyEffect {}

impl Displays for ToyEffect {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.name())
    }
}

enum TestControllerAction {
    Nothing,
    NoteOn,
    NoteOff,
}

/// An [IsController](groove_core::traits::IsController) that emits a MIDI
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
    time_range: Range<MusicalTime>,

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
    fn ui(&mut self, ui: &mut Ui) -> eframe::egui::Response {
        ui.label(self.name())
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

#[cfg(test)]
pub mod tests {
    use crate::ToyInstrument;
    use ensnare_core::traits::prelude::*;

    // TODO: restore tests that test basic trait behavior, then figure out how
    // to run everyone implementing those traits through that behavior. For now,
    // this one just tests that a generic instrument doesn't panic when accessed
    // for non-consecutive time slices.
    #[test]
    fn sources_audio_random_access() {
        let mut instrument = ToyInstrument::default();
        let mut rng = oorandom::Rand32::new(0);

        for _ in 0..100 {
            instrument.tick(rng.rand_range(1..10) as usize);
            let _ = instrument.value();
        }
    }
}
