// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::prelude::*;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// #[derive(Debug, Display)]
// pub enum ToyInstrumentAction {
//     LinkControl(Uid, Uid, ControlIndex),
// }
// impl IsAction for ToyInstrumentAction {}

#[derive(Debug, Default)]
pub struct ToyInstrumentEphemerals {
    sample: StereoSample,
    pub is_playing: bool,
    pub received_midi_message_count: Arc<Mutex<usize>>,
    pub debug_messages: Vec<MidiMessage>,
}

/// An [IsInstrument](ensnare::traits::IsInstrument) that uses a default
/// [Oscillator] to produce sound. Its "envelope" is just a boolean that
/// responds to MIDI NoteOn/NoteOff. Unlike [super::ToySynth], it is monophonic.
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
    fn value(&self) -> StereoSample {
        self.e.sample
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
impl Ticks for ToyInstrumentCore {
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
    pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
        &self.e.received_midi_message_count
    }

    pub fn notify_change_dca(&mut self) {}
}
