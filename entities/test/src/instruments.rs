// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::{IsEntity2, Metadata, Params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// The smallest possible [IsEntity2].
#[derive(Debug, Default, IsEntity2, Metadata, Params, Serialize, Deserialize)]
#[entity2(SkipInner)]
#[entity2(
    Controllable,
    Controls,
    Displays,
    HandlesMidi,
    Serializable,
    SkipInner,
    Ticks,
    TransformsAudio
)]

pub struct TestInstrument {
    pub uid: Uid,
    pub sample_rate: SampleRate,
}
impl TestInstrument {
    pub fn new_with(uid: Uid, _: &TestInstrumentParams) -> Self {
        Self {
            uid,
            ..Default::default()
        }
    }
}
impl Configurable for TestInstrument {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }
}
impl Generates<StereoSample> for TestInstrument {
    fn value(&self) -> StereoSample {
        StereoSample::default()
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        values.fill(StereoSample::default())
    }
}

/// An [IsEntity](ensnare::traits::IsEntity) that counts how many
/// MIDI messages it has received.
#[derive(Debug, Default, IsEntity2, Metadata, Params, Serialize, Deserialize)]
#[entity2(
    Configurable,
    Controllable,
    Controls,
    Displays,
    Serializable,
    SkipInner,
    Ticks,
    TransformsAudio
)]
#[entity2("skip_inner", "controls")]
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
impl TestInstrumentCountsMidiMessages {
    pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
        &self.received_midi_message_count
    }
}
