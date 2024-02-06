// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::core::prelude::*;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Produces a constant audio signal. Used for ensuring that a known signal
/// value gets all the way through the pipeline.
#[derive(Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestAudioSource {
    // This should be a Normal, but we use this audio source for testing
    // edge conditions. Thus we need to let it go out of range.
    #[control]
    level: ParameterType,

    #[serde(skip)]
    sample_rate: SampleRate,
}
impl Ticks for TestAudioSource {}
impl Generates<StereoSample> for TestAudioSource {
    fn value(&self) -> StereoSample {
        StereoSample::from(self.level)
    }

    fn generate(&mut self, values: &mut [StereoSample]) {
        values.fill(self.value());
    }
}
impl Configurable for TestAudioSource {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }
}
#[allow(dead_code)]
impl TestAudioSource {
    pub const TOO_LOUD: SampleType = 1.1;
    pub const LOUD: SampleType = 1.0;
    pub const MEDIUM: SampleType = 0.5;
    pub const SILENT: SampleType = 0.0;
    pub const QUIET: SampleType = -1.0;
    pub const TOO_QUIET: SampleType = -1.1;

    pub fn new_with(level: ParameterType) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    pub fn level(&self) -> f64 {
        self.level
    }

    pub fn set_level(&mut self, level: ParameterType) {
        self.level = level;
    }
}

#[derive(Debug, Default)]
pub struct TestControllerAlwaysSendsMidiMessage {
    midi_note: u8,
    is_performing: bool,
}
impl HandlesMidi for TestControllerAlwaysSendsMidiMessage {}
impl Controls for TestControllerAlwaysSendsMidiMessage {
    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.is_performing {
            control_events_fn(WorkEvent::Midi(
                MidiChannel::default(),
                MidiMessage::NoteOn {
                    key: u7::from(self.midi_note),
                    vel: u7::from(127),
                },
            ));
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
impl Configurable for TestControllerAlwaysSendsMidiMessage {}
impl Serializable for TestControllerAlwaysSendsMidiMessage {}
