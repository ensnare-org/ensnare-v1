// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_proc_macros::Control;
use ensnare_v1::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct ToyInstrumentEphemerals {
    pub is_playing: bool,
    pub received_midi_message_count: Arc<Mutex<usize>>,
    pub debug_messages: Vec<MidiMessage>,

    pub oscillator_buffer: Vec<BipolarNormal>,
    pub mono_buffer: Vec<Sample>,
}

/// An instrument that uses a default [Oscillator] to produce sound. Its
/// "envelope" is just a boolean that responds to MIDI NoteOn/NoteOff. Unlike
/// [super::ToySynthCore], it is monophonic.
#[derive(Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ToyInstrumentCore {
    pub oscillator: Oscillator,

    #[control]
    pub dca: Dca,

    #[serde(skip)]
    e: ToyInstrumentEphemerals,
}
impl Generates<StereoSample> for ToyInstrumentCore {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        self.set_buffer_sizes(values.len());
        if self.e.is_playing {
            self.oscillator.generate(&mut self.e.oscillator_buffer);
            self.e
                .oscillator_buffer
                .iter()
                .zip(self.e.mono_buffer.iter_mut())
                .for_each(|(s, d)| *d = (*s).into());
            self.dca
                .transform_batch_to_stereo(&self.e.mono_buffer, values);
            true
        } else {
            values.fill(StereoSample::SILENCE);
            false
        }
    }
}
impl Configurable for ToyInstrumentCore {
    fn sample_rate(&self) -> SampleRate {
        self.oscillator.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.oscillator.update_sample_rate(sample_rate);
    }
}
impl HandlesMidi for ToyInstrumentCore {
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
impl Serializable for ToyInstrumentCore {}
impl ToyInstrumentCore {
    pub fn new() -> Self {
        Self {
            oscillator: Oscillator::default(),
            dca: Dca::default(),
            e: Default::default(),
        }
    }

    // If this instrument is being used in an integration test, then
    // received_midi_message_count provides insight into whether messages are
    // arriving.
    #[allow(dead_code)]
    pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
        &self.e.received_midi_message_count
    }

    pub fn notify_change_dca(&mut self) {}

    fn set_buffer_sizes(&mut self, len: usize) {
        if len != self.e.oscillator_buffer.len() {
            self.e
                .oscillator_buffer
                .resize(len, BipolarNormal::default());
            self.e.mono_buffer.resize(len, Sample::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cores::tests::{check_instrument, GeneratesStereoSampleAndHandlesMidi};

    impl GeneratesStereoSampleAndHandlesMidi for ToyInstrumentCore {}

    #[test]
    fn toy_instrument_works() {
        let mut instrument = ToyInstrumentCore::default();
        check_instrument(&mut instrument);
    }
}
