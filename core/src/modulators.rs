// Copyright (c) 2023 Mike Tsao. All rights reserved.

use std::collections::HashMap;

use crate::{
    prelude::*,
    types::{BipolarNormal, Normal},
};
use ensnare_proc_macros::{Control, Params};
use serde::{Deserialize, Serialize};

/// The Digitally Controller Amplifier (DCA) handles gain and pan for many kinds
/// of synths.
///
/// See DSSPC++, Section 7.9 for requirements. TODO: implement
#[derive(Debug, Default, Control, Params, Serialize, Deserialize)]
pub struct Dca {
    #[control]
    #[params]
    gain: Normal,

    #[control]
    #[params]
    pan: BipolarNormal,
}
impl Dca {
    pub fn new_with(params: &DcaParams) -> Self {
        Self {
            gain: params.gain(),
            pan: params.pan(),
        }
    }

    pub fn transform_audio_to_stereo(&mut self, input_sample: Sample) -> StereoSample {
        // See Pirkle, DSSPC++, p.73
        let input_sample: f64 = input_sample.0 * self.gain.0;
        let left_pan: f64 = 1.0 - 0.25 * (self.pan.0 + 1.0f64).powi(2);
        let right_pan: f64 = 1.0 - (0.5 * self.pan.0 - 0.5f64).powi(2);
        StereoSample::new(
            (left_pan * input_sample).into(),
            (right_pan * input_sample).into(),
        )
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

    pub fn update_from_params(&mut self, params: &DcaParams) {
        self.set_gain(params.gain());
        self.set_pan(params.pan());
    }
}
impl Displays for Dca {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MainMixer {
    pub track_output: HashMap<TrackUid, Normal>,
    pub track_mute: HashMap<TrackUid, bool>,
    pub solo_track: Option<TrackUid>,
}
impl MainMixer {
    pub fn set_track_output(&mut self, track_uid: TrackUid, output: Normal) {
        self.track_output.insert(track_uid, output);
    }

    pub fn mute_track(&mut self, track_uid: TrackUid, muted: bool) {
        self.track_mute.insert(track_uid, muted);
    }

    pub fn solo_track(&self) -> Option<TrackUid> {
        self.solo_track
    }

    pub fn set_solo_track(&mut self, track_uid: TrackUid) {
        self.solo_track = Some(track_uid)
    }

    pub fn end_solo(&mut self) {
        self.solo_track = None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dca_mainline() {
        let mut dca = Dca::new_with(&&DcaParams::default());
        const VALUE_IN: Sample = Sample(0.5);
        const VALUE: Sample = Sample(0.5);
        assert_eq!(
            dca.transform_audio_to_stereo(VALUE_IN),
            StereoSample::new(VALUE * 0.75, VALUE * 0.75),
            "Pan center should give 75% equally to each channel"
        );

        dca.set_pan(BipolarNormal::new(-1.0));
        assert_eq!(
            dca.transform_audio_to_stereo(VALUE_IN),
            StereoSample::new(VALUE, 0.0.into()),
            "Pan left should give 100% to left channel"
        );

        dca.set_pan(BipolarNormal::new(1.0));
        assert_eq!(
            dca.transform_audio_to_stereo(VALUE_IN),
            StereoSample::new(0.0.into(), VALUE),
            "Pan right should give 100% to right channel"
        );
    }
}
