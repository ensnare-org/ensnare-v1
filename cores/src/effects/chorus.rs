// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::delay::{DelayLine, Delays};
use ensnare_core::{prelude::*, time::Seconds};
use ensnare_proc_macros::Control;

/// Schroeder reverb. Uses four parallel recirculating delay lines feeding into
/// a series of two all-pass delay lines.
#[derive(Debug, Default, Control)]
pub struct Chorus {
    /// The number of voices in the chorus.
    #[control]
    voices: usize,

    /// The number of seconds to delay.
    #[control]
    delay: Seconds,

    delay_line: DelayLine,
}
impl Serializable for Chorus {}
impl TransformsAudio for Chorus {
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
impl Configurable for Chorus {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.delay_line.update_sample_rate(sample_rate);
    }
}
impl Chorus {
    pub fn new_with(voices: usize, delay: Seconds) -> Self {
        // TODO: the delay_seconds param feels like a hack
        Self {
            voices,
            delay,
            delay_line: DelayLine::new_with(delay, 1.0),
        }
    }

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
    }
}

#[cfg(test)]
mod tests {
    //TODO
}
