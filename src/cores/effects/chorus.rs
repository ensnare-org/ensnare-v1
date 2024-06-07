// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derivative::Derivative;
use derive_builder::Builder;
use ensnare::cores::{DelayLine, Delays};
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Schroeder reverb. Uses four parallel recirculating delay lines feeding into
/// a series of two all-pass delay lines.
#[derive(Debug, Builder, Derivative, Control, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct ChorusCore {
    /// The number of voices in the chorus.
    #[control]
    #[derivative(Default(value = "4"))]
    voices: usize,

    /// The number of seconds to delay.
    #[control]
    #[derivative(Default(value = "1.0.into()"))]
    delay: Seconds,

    /// How soon the chorus fades. 1.0 = never
    #[control]
    #[derivative(Default(value = "1.0.into()"))]
    decay: Normal,

    #[serde(skip)]
    #[builder(setter(skip))]
    delay_line: DelayLine,
}
impl ChorusCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<ChorusCore, ChorusCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}

impl Serializable for ChorusCore {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.delay_line = DelayLine::new_with(self.delay, 0.5.into());
    }
}
impl TransformsAudio for ChorusCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let index_offset: f64 = (self.delay / self.voices).into();
        let mut sum = self.delay_line.pop_output(input_sample);
        for i in 1..self.voices as isize {
            sum += self
                .delay_line
                .peek_indexed_output(i * index_offset as isize);
        }
        sum
    }
}
impl Configurable for ChorusCore {
    delegate! {
        to self.delay_line {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl ChorusCore {
    pub fn voices(&self) -> usize {
        self.voices
    }

    pub fn set_voices(&mut self, voices: usize) {
        self.voices = voices;
    }

    pub fn delay(&self) -> Seconds {
        self.delay
    }

    pub fn set_delay(&mut self, delay: Seconds) {
        self.delay = delay;
        self.delay_line.set_delay(delay);
    }

    pub fn set_decay(&mut self, decay: Normal) {
        self.decay = decay;
        self.delay_line.set_decay(decay);
    }
}
