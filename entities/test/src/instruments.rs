// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    midi::{MidiChannel, MidiMessage},
    time::SampleRate,
    traits::{
        Configurable, Controllable, Generates, HandlesMidi, MidiMessagesFn, Serializable, Ticks,
    },
    types::StereoSample,
    uid::Uid,
};
use ensnare_entity::traits::Displays;
use ensnare_proc_macros::{IsInstrument, Metadata};
use std::sync::{Arc, Mutex};

/// The smallest possible [IsInstrument].
#[derive(Debug, Default, IsInstrument, Metadata)]
pub struct TestInstrument {
    pub uid: Uid,
    pub sample_rate: SampleRate,
}
impl Displays for TestInstrument {}
impl HandlesMidi for TestInstrument {}
impl Controllable for TestInstrument {}
impl Configurable for TestInstrument {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }
}
impl Serializable for TestInstrument {}
impl Generates<StereoSample> for TestInstrument {
    fn value(&self) -> StereoSample {
        StereoSample::default()
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        values.fill(StereoSample::default())
    }
}
impl Ticks for TestInstrument {}

/// An [IsInstrument](ensnare::traits::IsInstrument) that counts how many
/// MIDI messages it has received.
#[derive(Debug, Default, IsInstrument, Metadata)]
pub struct TestInstrumentCountsMidiMessages {
    uid: Uid,
    pub received_midi_message_count: Arc<Mutex<usize>>,
}
impl Generates<StereoSample> for TestInstrumentCountsMidiMessages {
    fn value(&self) -> StereoSample {
        StereoSample::default()
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        values.fill(StereoSample::default())
    }
}
impl Configurable for TestInstrumentCountsMidiMessages {}
impl Controllable for TestInstrumentCountsMidiMessages {}
impl Ticks for TestInstrumentCountsMidiMessages {}
impl HandlesMidi for TestInstrumentCountsMidiMessages {
    fn handle_midi_message(
        &mut self,
        _action: MidiChannel,
        _: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        if let Ok(mut received_count) = self.received_midi_message_count.lock() {
            *received_count += 1;
        }
    }
}
impl Serializable for TestInstrumentCountsMidiMessages {}
impl TestInstrumentCountsMidiMessages {
    pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
        &self.received_midi_message_count
    }
}
impl Displays for TestInstrumentCountsMidiMessages {}
