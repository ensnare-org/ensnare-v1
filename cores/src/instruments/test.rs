// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::{Control, Params};

/// Produces a constant audio signal. Used for ensuring that a known signal
/// value gets all the way through the pipeline.
#[derive(Debug, Default, Control, Params)]
pub struct TestAudioSource {
    // This should be a Normal, but we use this audio source for testing
    // edge conditions. Thus we need to let it go out of range.
    #[control]
    #[params]
    level: ParameterType,

    sample_rate: SampleRate,
}
impl Ticks for TestAudioSource {}
impl Generates<StereoSample> for TestAudioSource {
    fn value(&self) -> StereoSample {
        StereoSample::from(self.level)
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
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

    pub fn new_with(params: &TestAudioSourceParams) -> Self {
        Self {
            level: params.level(),
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
