// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GainCore {
    #[control]
    ceiling: Normal,

    #[serde(skip)]
    c: Configurables,
}
impl Serializable for GainCore {}
impl Configurable for GainCore {
    delegate! {
        to self.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl TransformsAudio for GainCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        Sample(input_sample.0 * self.ceiling.0)
    }
}
impl GainCore {
    pub fn new_with(ceiling: Normal) -> Self {
        Self {
            ceiling,
            ..Default::default()
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
    use crate::cores::instruments::TestAudioSourceCore;

    #[test]
    fn gain_mainline() {
        let mut gain = GainCore::new_with(Normal::new(0.5));
        assert_eq!(
            gain.transform_audio(TestAudioSourceCore::new_with(TestAudioSourceCore::LOUD).value()),
            StereoSample::from(0.5)
        );
    }
}
