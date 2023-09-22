// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{self, Ui};
use ensnare::{
    generators::{Oscillator, OscillatorParams, Waveform},
    midi::prelude::*,
    modulators::{Dca, DcaParams},
    prelude::*,
    traits::prelude::*,
};
use ensnare_proc_macros::{Control, IsEffect, IsInstrument, Params, Uid};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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
