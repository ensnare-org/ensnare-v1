// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;

/// An effect that negates the input.
#[derive(Debug, Default)]
pub struct TestEffectNegatesInput {}
impl TransformsAudio for TestEffectNegatesInput {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        -input_sample
    }
}
