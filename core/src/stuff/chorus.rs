// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::delay::{DelayLine, Delays};
use crate::prelude::*;
use ensnare_proc_macros::{Control, Params};

/// Schroeder reverb. Uses four parallel recirculating delay lines feeding into
/// a series of two all-pass delay lines.
#[derive(Debug, Default, Control, Params)]
pub struct Chorus {
    /// The number of voices in the chorus.
    #[control]
    #[params]
    voices: usize,

    /// The number of seconds to delay.
    #[control]
    #[params]
    delay_seconds: ParameterType,

    delay: DelayLine,
}
impl Serializable for Chorus {}
impl TransformsAudio for Chorus {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let index_offset = self.delay_seconds / self.voices as ParameterType;
        let mut sum = self.delay.pop_output(input_sample);
        for i in 1..self.voices as isize {
            sum += self.delay.peek_indexed_output(i * index_offset as isize);
        }
        sum
    }
}
impl Configurable for Chorus {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.delay.update_sample_rate(sample_rate);
    }
}
impl Chorus {
    #[allow(dead_code)]
    fn new() -> Self {
        Self::default()
    }

    pub fn new_with(params: &ChorusParams) -> Self {
        // TODO: the delay_seconds param feels like a hack
        Self {
            voices: params.voices(),
            delay_seconds: params.delay_seconds(),
            delay: DelayLine::new_with(params.delay_seconds(), 1.0),
        }
    }

    pub fn voices(&self) -> usize {
        self.voices
    }

    pub fn set_voices(&mut self, voices: usize) {
        self.voices = voices;
    }

    pub fn delay_seconds(&self) -> f64 {
        self.delay_seconds
    }

    pub fn set_delay_seconds(&mut self, delay_seconds: ParameterType) {
        self.delay_seconds = delay_seconds;
    }
}

#[cfg(test)]
mod tests {
    //TODO
}
