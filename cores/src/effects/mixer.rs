// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::Control;

// TODO: I don't think Mixer needs to exist.
#[derive(Debug, Default, Control)]
pub struct Mixer {}
impl TransformsAudio for Mixer {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        // This is a simple pass-through because it's the job of the
        // infrastructure to provide a sum of all inputs as the input.
        // Eventually this might turn into a weighted mixer, or we might handle
        // that by putting `Gain`s in front.
        input_sample
    }
}
impl Mixer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mixer_mainline() {
        // This could be replaced with a test, elsewhere, showing that
        // Orchestrator's gather_audio() method can gather audio.
    }
}
