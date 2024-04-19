// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default)]
pub struct GainCore {
    #[control]
    ceiling: Normal,

    #[serde(skip)]
    #[builder(setter(skip))]
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
    use crate::cores::instruments::{TestAudioSourceCore, TestAudioSourceCoreBuilder};

    #[test]
    fn gain_mainline() {
        let mut gain = GainCoreBuilder::default()
            .ceiling(0.5.into())
            .build()
            .unwrap();
        let mut buffer = [StereoSample::default(); 1];
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::LOUD)
            .build()
            .unwrap()
            .generate(&mut buffer);
        assert_eq!(gain.transform_audio(buffer[0]), StereoSample::from(0.5));
    }
}
