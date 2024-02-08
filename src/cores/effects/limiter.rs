// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derivative::Derivative;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, Derivative, Control, Serialize, Deserialize)]
#[derivative(Default)]
#[builder(default)]
#[serde(rename_all = "kebab-case")]
pub struct LimiterCore {
    #[control]
    #[derivative(Default(value = "Normal::minimum()"))]
    minimum: Normal,
    #[control]
    #[derivative(Default(value = "Normal::maximum()"))]
    maximum: Normal,

    #[serde(skip)]
    #[builder(setter(skip))]
    c: Configurables,
}
impl Serializable for LimiterCore {}
impl Configurable for LimiterCore {
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
impl TransformsAudio for LimiterCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let sign = input_sample.0.signum();
        Sample::from(input_sample.0.abs().clamp(self.minimum.0, self.maximum.0) * sign)
    }
}
impl LimiterCore {
    pub fn new_with(minimum: Normal, maximum: Normal) -> Self {
        Self {
            minimum,
            maximum,
            ..Default::default()
        }
    }

    pub fn maximum(&self) -> Normal {
        self.maximum
    }

    pub fn set_maximum(&mut self, max: Normal) {
        self.maximum = max;
    }

    pub fn minimum(&self) -> Normal {
        self.minimum
    }

    pub fn set_minimum(&mut self, min: Normal) {
        self.minimum = min;
    }
}

/// re-enable when moved into new crate
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cores::instruments::TestAudioSource;
    use more_asserts::{assert_gt, assert_lt};

    #[test]
    fn limiter_mainline() {
        // audio sources are at or past boundaries
        assert_gt!(
            TestAudioSource::new_with(TestAudioSource::TOO_LOUD).value(),
            StereoSample::MAX
        );
        assert_eq!(
            TestAudioSource::new_with(TestAudioSource::LOUD).value(),
            StereoSample::MAX
        );
        assert_eq!(
            TestAudioSource::new_with(TestAudioSource::SILENT).value(),
            StereoSample::SILENCE
        );
        assert_eq!(
            TestAudioSource::new_with(TestAudioSource::QUIET).value(),
            StereoSample::MIN
        );
        assert_lt!(
            TestAudioSource::new_with(TestAudioSource::TOO_QUIET).value(),
            StereoSample::MIN
        );

        // Limiter clamps high and low, and doesn't change values inside the range.
        let mut limiter = LimiterCore::default();
        assert_eq!(
            limiter.transform_audio(TestAudioSource::new_with(TestAudioSource::TOO_LOUD).value()),
            StereoSample::MAX
        );
        assert_eq!(
            limiter.transform_audio(TestAudioSource::new_with(TestAudioSource::LOUD).value()),
            StereoSample::MAX
        );
        assert_eq!(
            limiter.transform_audio(TestAudioSource::new_with(TestAudioSource::SILENT).value()),
            StereoSample::SILENCE
        );
        assert_eq!(
            limiter.transform_audio(TestAudioSource::new_with(TestAudioSource::QUIET).value()),
            StereoSample::MIN
        );
        assert_eq!(
            limiter.transform_audio(TestAudioSource::new_with(TestAudioSource::TOO_QUIET).value()),
            StereoSample::MIN
        );
    }

    #[test]
    fn limiter_bias() {
        let mut limiter = LimiterCore::new_with(0.2.into(), 0.8.into());
        assert_eq!(
            limiter.transform_channel(0, Sample::from(0.1)),
            Sample::from(0.2),
            "Limiter failed to clamp min {}",
            0.2
        );
        assert_eq!(
            limiter.transform_channel(0, Sample::from(0.9)),
            Sample::from(0.8),
            "Limiter failed to clamp max {}",
            0.8
        );
        assert_eq!(
            limiter.transform_channel(0, Sample::from(-0.1)),
            Sample::from(-0.2),
            "Limiter failed to clamp min {} for negative values",
            0.2
        );
        assert_eq!(
            limiter.transform_channel(0, Sample::from(-0.9)),
            Sample::from(-0.8),
            "Limiter failed to clamp max {} for negative values",
            0.8
        );
    }
}
