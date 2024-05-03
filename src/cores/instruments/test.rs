// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Produces a constant audio signal. Used for ensuring that a known signal
/// value gets all the way through the pipeline.
#[derive(Clone, Builder, Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default)]
pub struct TestAudioSourceCore {
    // This should be a Normal, but we use this audio source for testing
    // edge conditions. Thus we need to let it go out of range.
    #[control]
    level: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    c: Configurables,
}
impl Generates<StereoSample> for TestAudioSourceCore {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        values.fill(StereoSample::from(self.level));
        self.level != 0.0
    }
}
impl Configurable for TestAudioSourceCore {
    delegate! {
        to self.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl TestAudioSourceCore {
    pub const TOO_LOUD: SampleType = 1.1;
    pub const LOUD: SampleType = 1.0;
    pub const MEDIUM: SampleType = 0.5;
    pub const SILENT: SampleType = 0.0;
    pub const QUIET: SampleType = -1.0;
    pub const TOO_QUIET: SampleType = -1.1;

    pub fn level(&self) -> f64 {
        self.level
    }

    pub fn set_level(&mut self, level: ParameterType) {
        self.level = level;
    }
}

#[derive(Debug, Default, Control)]
pub struct TestControllerAlwaysSendsMidiMessageCore {
    midi_note: u8,
    is_performing: bool,
}
impl HandlesMidi for TestControllerAlwaysSendsMidiMessageCore {}
impl Controls for TestControllerAlwaysSendsMidiMessageCore {
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
impl Configurable for TestControllerAlwaysSendsMidiMessageCore {}
impl Serializable for TestControllerAlwaysSendsMidiMessageCore {}
