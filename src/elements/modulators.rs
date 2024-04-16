// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// The Digitally Controller Amplifier (DCA) handles gain and pan for many kinds
/// of synths.
///
/// See DSSPC++, Section 7.9 for requirements. TODO: implement
#[derive(Clone, Copy, Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Dca {
    #[control]
    gain: Normal,

    #[control]
    pan: BipolarNormal,
}
impl Dca {
    #[allow(missing_docs)]
    pub fn new_with(gain: Normal, pan: BipolarNormal) -> Self {
        Self { gain, pan }
    }

    /// Transforms a mono sample into a stereo sample using our gain/pan parameters
    pub fn transform_audio_to_stereo_non_batch(&mut self, input_sample: Sample) -> StereoSample {
        // See Pirkle, DSSPC++, p.73
        let input_sample: f64 = input_sample.0 * self.gain.0;
        let left_pan: f64 = 1.0 - 0.25 * (self.pan.0 + 1.0f64).powi(2);
        let right_pan: f64 = 1.0 - (0.5 * self.pan.0 - 0.5f64).powi(2);
        StereoSample::new(
            (left_pan * input_sample).into(),
            (right_pan * input_sample).into(),
        )
    }

    /// Transforms batch of mono samples into stereo samples using our gain/pan parameters
    pub fn transform_audio_to_stereo_batch(
        &mut self,
        inputs: &[Sample],
        outputs: &mut [StereoSample],
    ) {
        // See Pirkle, DSSPC++, p.73
        inputs
            .iter()
            .zip(outputs.iter_mut())
            .for_each(|(input, output)| {
                let input_sample: f64 = input.0 * self.gain.0;
                let left_pan: f64 = 1.0 - 0.25 * (self.pan.0 + 1.0f64).powi(2);
                let right_pan: f64 = 1.0 - (0.5 * self.pan.0 - 0.5f64).powi(2);
                *output = StereoSample::new(
                    (left_pan * input_sample).into(),
                    (right_pan * input_sample).into(),
                );
            });
    }

    pub fn gain(&self) -> Normal {
        self.gain
    }

    pub fn set_gain(&mut self, gain: Normal) {
        self.gain = gain;
    }

    pub fn pan(&self) -> BipolarNormal {
        self.pan
    }

    pub fn set_pan(&mut self, pan: BipolarNormal) {
        self.pan = pan;
    }
}
impl CanPrototype for Dca {
    fn update_from_prototype(&mut self, prototype: &Self) -> &Self {
        self.set_gain(prototype.gain());
        self.set_pan(prototype.pan());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dca_mainline() {
        let mut dca = Dca::new_with(Normal::default(), BipolarNormal::default());
        const VALUE_IN: Sample = Sample(0.5);
        const VALUE: Sample = Sample(0.5);
        assert_eq!(
            dca.transform_audio_to_stereo_non_batch(VALUE_IN),
            StereoSample::new(VALUE * 0.75, VALUE * 0.75),
            "Pan center should give 75% equally to each channel"
        );

        dca.set_pan(BipolarNormal::new(-1.0));
        assert_eq!(
            dca.transform_audio_to_stereo_non_batch(VALUE_IN),
            StereoSample::new(VALUE, 0.0.into()),
            "Pan left should give 100% to left channel"
        );

        dca.set_pan(BipolarNormal::new(1.0));
        assert_eq!(
            dca.transform_audio_to_stereo_non_batch(VALUE_IN),
            StereoSample::new(0.0.into(), VALUE),
            "Pan right should give 100% to right channel"
        );
    }
}
