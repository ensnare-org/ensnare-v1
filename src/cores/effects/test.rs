// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// An effect that negates the input.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestEffectNegatesInput {}
impl TransformsAudio for TestEffectNegatesInput {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        -input_sample
    }
}
