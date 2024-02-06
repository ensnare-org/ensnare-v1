// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::core::prelude::*;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Gain {
    #[control]
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
    pub fn new_with(ceiling: Normal) -> Self {
        Self { ceiling }
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
    use crate::cores::TestAudioSource;

    use super::*;

    #[test]
    fn gain_mainline() {
        let mut gain = Gain::new_with(Normal::new(0.5));
        assert_eq!(
            gain.transform_audio(TestAudioSource::new_with(TestAudioSource::LOUD).value()),
            StereoSample::from(0.5)
        );
    }
}
