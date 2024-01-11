// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::{Control, Params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Control, Params, Serialize, Deserialize)]
pub struct Gain {
    #[control]
    #[params]
    ceiling: Normal,
}
impl Serializable for Gain {}
impl Configurable for Gain {}
impl TransformsAudio for Gain {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        Sample(input_sample.0 * self.ceiling.0)
    }
}
impl Gain {
    pub fn new_with(params: &GainParams) -> Self {
        Self {
            ceiling: params.ceiling,
        }
    }

    pub fn ceiling(&self) -> Normal {
        self.ceiling
    }

    pub fn set_ceiling(&mut self, ceiling: Normal) {
        self.ceiling = ceiling;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestAudioSource;

    #[test]
    fn gain_mainline() {
        let mut gain = Gain::new_with(&GainParams {
            ceiling: Normal::new(0.5),
        });
        assert_eq!(
            gain.transform_audio(TestAudioSource::new_with(TestAudioSource::LOUD).value()),
            StereoSample::from(0.5)
        );
    }
}
