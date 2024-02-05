// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::repositories::{EntityRepository, TrackRepository};
use anyhow::Result;
use delegate::delegate;
use ensnare_core::prelude::*;
use ensnare_entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, option::Option};

/// Controls the wet/dry mix of arranged effects.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Humidifier {
    uid_to_humidity: HashMap<Uid, Normal>,
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

    pub(super) fn transform_channel(
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
