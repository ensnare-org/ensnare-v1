// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use core::fmt::Debug;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Controls the wet/dry mix of arranged effects.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Humidifier {
    uid_to_humidity: FxHashMap<Uid, Normal>,

    #[serde(skip)]
    transformation_buffer: GenerationBuffer<StereoSample>,
}
impl Humidifier {
    pub fn get_humidity(&self, uid: &Uid) -> Normal {
        self.uid_to_humidity.get(uid).cloned().unwrap_or_default()
    }

    pub fn set_humidity(&mut self, uid: Uid, humidity: Normal) {
        self.uid_to_humidity.insert(uid, humidity);
    }

    pub fn transform_batch(
        &mut self,
        humidity: Normal,
        effect: &mut Box<dyn EntityBounds>,
        samples: &mut [StereoSample],
    ) {
        self.transformation_buffer.resize(samples.len());
        self.transformation_buffer
            .buffer_mut()
            .copy_from_slice(samples);
        effect.transform(samples);

        for (pre, post) in self
            .transformation_buffer
            .buffer()
            .iter()
            .zip(samples.iter_mut())
        {
            *post = StereoSample(
                self.transform_channel(humidity, 0, pre.0, post.0),
                self.transform_channel(humidity, 1, pre.1, post.1),
            )
        }
    }

    pub fn transxxxform_audio(
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

    pub(super) fn transform_channel(
        &self,
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
