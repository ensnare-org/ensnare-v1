// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    traits::TransformsAudio,
    types::{Normal, Sample, StereoSample},
    uid::Uid,
};
use std::collections::HashMap;

/// Controls the wet/dry mix of arranged effects.
#[derive(Debug, Default, PartialEq)]
pub struct Humidifier {
    uid_to_humidity: HashMap<Uid, Normal>,
}
impl Humidifier {
    pub fn get_humidity_by_uid(&self, uid: &Uid) -> Normal {
        if let Some(humidity) = self.uid_to_humidity.get(uid) {
            *humidity
        } else {
            Normal::default()
        }
    }

    pub fn set_humidity_by_uid(&mut self, uid: Uid, humidity: Normal) {
        self.uid_to_humidity.insert(uid, humidity);
    }

    pub fn transform_batch(
        &mut self,
        humidity: Normal,
        effect: &mut dyn TransformsAudio,
        samples: &mut [StereoSample],
    ) {
        for sample in samples {
            *sample = self.transform_audio(humidity, *sample, effect.transform_audio(*sample));
        }
    }

    pub fn transform_audio(
        &mut self,
        humidity: Normal,
        pre_effect: StereoSample,
        post_effect: StereoSample,
    ) -> StereoSample {
        StereoSample(
            self.transform_channel(humidity, 0, pre_effect.0, post_effect.0),
            self.transform_channel(humidity, 1, pre_effect.1, post_effect.1),
        )
    }

    fn transform_channel(
        &mut self,
        humidity: Normal,
        _: usize,
        pre_effect: Sample,
        post_effect: Sample,
    ) -> Sample {
        let humidity: f64 = humidity.into();
        let aridity = 1.0 - humidity;
        post_effect * humidity + pre_effect * aridity
    }
}

#[cfg(test)]
mod tests {
    use crate::humidifier::Humidifier;
    use ensnare_core::{
        traits::TransformsAudio,
        types::{Normal, Sample},
        uid::Uid,
    };
    use ensnare_cores::TestEffectNegatesInput;
}
