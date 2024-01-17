// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::Control;

/// An effect that applies a negative gain.
#[derive(Debug, Default, Control)]
pub struct ToyEffect {
    /// The [ToyEffect] transformation is signal * -magnitude.
    #[control]
    pub magnitude: Normal,

    sample_rate: SampleRate,
    tempo: Tempo,
    time_signature: TimeSignature,
}
impl ToyEffect {
    pub fn new_with(magnitude: Normal) -> Self {
        Self {
            magnitude,
            ..Default::default()
        }
    }

    pub fn set_magnitude(&mut self, magnitude: Normal) {
        self.magnitude = magnitude;
    }

    pub fn magnitude(&self) -> Normal {
        self.magnitude
    }
}
impl TransformsAudio for ToyEffect {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        input_sample * self.magnitude * -1.0
    }
}
impl Configurable for ToyEffect {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }

    fn tempo(&self) -> Tempo {
        self.tempo
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo;
    }

    fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
    }
}
impl Serializable for ToyEffect {}
impl HandlesMidi for ToyEffect {}
