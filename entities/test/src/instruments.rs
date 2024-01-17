// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::{InnerInstrument, IsEntity, Metadata};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// The smallest possible [IsEntity].
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(SkipInner)]
#[entity(
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
    pub fn new_with(uid: Uid) -> Self {
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
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controllable,
    Controls,
    Displays,
    Serializable,
    SkipInner,
    Ticks,
    TransformsAudio
)]
#[entity("skip_inner", "controls")]
pub struct TestInstrumentCountsMidiMessages {
    uid: Uid,
    #[serde(skip)]
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

#[derive(Debug, Default, InnerInstrument, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controllable,
    Controls,
    Displays,
    Serializable,
    SkipInner,
    TransformsAudio
)]
#[entity(SkipInner, HandlesMidi)]
pub struct TestAudioSource {
    uid: Uid,
    inner: ensnare_cores::TestAudioSource,
}
impl TestAudioSource {
    pub const TOO_LOUD: SampleType = 1.1;
    pub const LOUD: SampleType = 1.0;
    pub const MEDIUM: SampleType = 0.5;
    pub const SILENT: SampleType = 0.0;
    pub const QUIET: SampleType = -1.0;
    pub const TOO_QUIET: SampleType = -1.1;

    pub fn new_with(uid: Uid, level: ParameterType) -> Self {
        Self {
            uid,
            inner: ensnare_cores::TestAudioSource::new_with(level),
        }
    }
}
