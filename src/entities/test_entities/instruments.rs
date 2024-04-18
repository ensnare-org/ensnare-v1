// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{cores::instruments::TestAudioSourceCore, prelude::*, traits::InternalBuffer};
use delegate::delegate;
use ensnare_proc_macros::{Control, InnerControllable, InnerInstrument, IsEntity, Metadata};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// The smallest possible [IsEntity].
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[entity(
    Controllable,
    Controls,
    Displays,
    HandlesMidi,
    Serializable,
    SkipInner,
    TransformsAudio
)]

pub struct TestInstrument {
    pub uid: Uid,
    pub sample_rate: SampleRate,

    #[serde(skip)]
    g: InternalBuffer<StereoSample>,
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
    delegate! {
        to self.g {
            fn buffer_size(&self) -> usize;
            fn set_buffer_size(&mut self, size: usize);
            fn buffer(&self) -> &[StereoSample];
            fn buffer_mut(&mut self) -> &mut [StereoSample];
        }
    }

    fn generate(&mut self) {
        // TODO: revisit whether we need to keep filling. At this moment I don't
        // know whether the buffer can be written to by anyone else, so we're
        // playing it safe by assuming it has changed since the last fill.
        self.generates_buffer_mut().fill(StereoSample::default())
    }
}

/// An [IsEntity](ensnare::traits::IsEntity) that counts how many
/// MIDI messages it has received.
#[derive(Debug, Default, Control, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controls,
    Displays,
    Serializable,
    SkipInner,
    TransformsAudio
)]
#[serde(rename_all = "kebab-case")]
pub struct TestInstrumentCountsMidiMessages {
    uid: Uid,

    #[serde(skip)]
    pub received_midi_message_count: Arc<Mutex<usize>>,

    #[serde(skip)]
    g: InternalBuffer<StereoSample>,
}
impl Generates<StereoSample> for TestInstrumentCountsMidiMessages {
    delegate! {
        to self.g {
            fn buffer_size(&self) -> usize;
            fn set_buffer_size(&mut self, size: usize);
            fn buffer(&self) -> &[StereoSample];
            fn buffer_mut(&mut self) -> &mut [StereoSample];
        }
    }

    fn generate(&mut self) {
        self.g.buffer_mut().fill(StereoSample::default())
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

#[derive(
    Debug, Default, InnerControllable, InnerInstrument, IsEntity, Metadata, Serialize, Deserialize,
)]
#[entity(
    Configurable,
    Controls,
    Displays,
    HandlesMidi,
    Serializable,
    SkipInner,
    TransformsAudio
)]
#[serde(rename_all = "kebab-case")]
pub struct TestAudioSource {
    uid: Uid,
    inner: TestAudioSourceCore,
}
impl TestAudioSource {
    pub const TOO_LOUD: SampleType = 1.1;
    pub const LOUD: SampleType = 1.0;
    pub const MEDIUM: SampleType = 0.5;
    pub const SILENT: SampleType = 0.0;
    pub const QUIET: SampleType = -1.0;
    pub const TOO_QUIET: SampleType = -1.1;

    pub fn new_with(uid: Uid, inner: TestAudioSourceCore) -> Self {
        Self { uid, inner }
    }
}
