// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{prelude::*, traits::prelude::*};
use eframe::egui::{Slider, Ui};
use ensnare_proc_macros::{Control, IsEffect, Metadata, Params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Control, IsEffect, Params, Metadata, Serialize, Deserialize)]
pub struct Limiter {
    uid: Uid,

    #[control]
    #[params]
    minimum: Normal,
    #[control]
    #[params]
    maximum: Normal,
}
impl Default for Limiter {
    fn default() -> Self {
        Self {
            uid: Default::default(),
            minimum: Normal::minimum(),
            maximum: Normal::maximum(),
        }
    }
}
impl Serializable for Limiter {}
impl Configurable for Limiter {}
impl TransformsAudio for Limiter {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let sign = input_sample.0.signum();
        Sample::from(input_sample.0.abs().clamp(self.minimum.0, self.maximum.0) * sign)
    }
}
impl Limiter {
    pub fn new_with(params: &LimiterParams) -> Self {
        Self {
            minimum: params.minimum(),
            maximum: params.maximum(),
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
impl Displays for Limiter {
    fn ui(&mut self, ui: &mut Ui) -> eframe::egui::Response {
        let mut min = self.minimum().to_percentage();
        let mut max = self.maximum().to_percentage();
        let min_response = ui.add(
            Slider::new(&mut min, 0.0..=max)
                .suffix(" %")
                .text("min")
                .fixed_decimals(2),
        );
        if min_response.changed() {
            self.set_minimum(min.into());
        };
        let max_response = ui.add(
            Slider::new(&mut max, min..=1.0)
                .suffix(" %")
                .text("max")
                .fixed_decimals(2),
        );
        if max_response.changed() {
            self.set_maximum(Normal::from_percentage(max));
        };
        min_response | max_response
    }
}

/// re-enable when moved into new crate
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::factory::test_entities::{TestAudioSource, TestAudioSourceParams};
    use more_asserts::{assert_gt, assert_lt};

    #[test]
    fn limiter_mainline() {
        // audio sources are at or past boundaries
        assert_gt!(
            TestAudioSource::new_with(&TestAudioSourceParams {
                level: TestAudioSource::TOO_LOUD
            })
            .value(),
            StereoSample::MAX
        );
        assert_eq!(
            TestAudioSource::new_with(&TestAudioSourceParams {
                level: TestAudioSource::LOUD
            })
            .value(),
            StereoSample::MAX
        );
        assert_eq!(
            TestAudioSource::new_with(&TestAudioSourceParams {
                level: TestAudioSource::SILENT
            })
            .value(),
            StereoSample::SILENCE
        );
        assert_eq!(
            TestAudioSource::new_with(&TestAudioSourceParams {
                level: TestAudioSource::QUIET
            })
            .value(),
            StereoSample::MIN
        );
        assert_lt!(
            TestAudioSource::new_with(&TestAudioSourceParams {
                level: TestAudioSource::TOO_QUIET
            })
            .value(),
            StereoSample::MIN
        );

        // Limiter clamps high and low, and doesn't change values inside the range.
        let mut limiter = Limiter::default();
        assert_eq!(
            limiter.transform_audio(
                TestAudioSource::new_with(&TestAudioSourceParams {
                    level: TestAudioSource::TOO_LOUD
                })
                .value()
            ),
            StereoSample::MAX
        );
        assert_eq!(
            limiter.transform_audio(
                TestAudioSource::new_with(&TestAudioSourceParams {
                    level: TestAudioSource::LOUD
                })
                .value()
            ),
            StereoSample::MAX
        );
        assert_eq!(
            limiter.transform_audio(
                TestAudioSource::new_with(&TestAudioSourceParams {
                    level: TestAudioSource::SILENT
                })
                .value()
            ),
            StereoSample::SILENCE
        );
        assert_eq!(
            limiter.transform_audio(
                TestAudioSource::new_with(&TestAudioSourceParams {
                    level: TestAudioSource::QUIET
                })
                .value()
            ),
            StereoSample::MIN
        );
        assert_eq!(
            limiter.transform_audio(
                TestAudioSource::new_with(&TestAudioSourceParams {
                    level: TestAudioSource::TOO_QUIET
                })
                .value()
            ),
            StereoSample::MIN
        );
    }

    #[test]
    fn limiter_bias() {
        let mut limiter = Limiter::new_with(&LimiterParams {
            minimum: 0.2.into(),
            maximum: 0.8.into(),
        });
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
