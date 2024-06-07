// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub mod filter;

use crate::cores::effects::{self, BitcrusherCore, ChorusCore, CompressorCore, LimiterCore};
use ensnare::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, GeneratesStereoSample, HandlesMidi, SkipInner)]
pub struct Bitcrusher {
    uid: Uid,
    inner: BitcrusherCore,
}
impl Bitcrusher {
    pub fn new_with(uid: Uid, inner: BitcrusherCore) -> Self {
        Self { uid, inner }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, GeneratesStereoSample, HandlesMidi, SkipInner)]

pub struct Chorus {
    uid: Uid,
    inner: ChorusCore,
}
impl Chorus {
    pub fn new_with(uid: Uid, inner: ChorusCore) -> Self {
        Self { uid, inner }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, GeneratesStereoSample, HandlesMidi, SkipInner)]

pub struct Compressor {
    uid: Uid,
    inner: CompressorCore,
}
impl Compressor {
    pub fn new_with(uid: Uid, inner: CompressorCore) -> Self {
        Self { uid, inner }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, GeneratesStereoSample, HandlesMidi, SkipInner)]

pub struct Limiter {
    uid: Uid,
    inner: LimiterCore,
}
impl Limiter {
    pub fn new_with(uid: Uid, inner: LimiterCore) -> Self {
        Self { uid, inner }
    }
}

#[cfg(feature = "egui")]
mod egui {
    use self::effects::BitcrusherCore;
    use super::*;
    use eframe::egui::Slider;

    impl Displays for Bitcrusher {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut bits = self.inner.bits();
            let response =
                ui.add(Slider::new(&mut bits, BitcrusherCore::bits_range()).suffix(" bits"));
            if response.changed() {
                self.inner.set_bits(bits);
            };
            response
        }
    }

    impl Displays for Chorus {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.label("Coming soon!")
        }
    }

    impl Displays for Compressor {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut threshold = self.inner.threshold().0;
            let mut ratio = self.inner.ratio().0;
            let mut attack = self.inner.attack().0;
            let mut release = self.inner.release().0;
            let threshold_response = ui.add(
                Slider::new(&mut threshold, Normal::range())
                    .fixed_decimals(2)
                    .text("Threshold"),
            );
            if threshold_response.changed() {
                self.inner.set_threshold(threshold.into());
            };
            let ratio_response = ui.add(
                Slider::new(&mut ratio, 0.05..=2.0)
                    .fixed_decimals(2)
                    .text("Ratio"),
            );
            if ratio_response.changed() {
                self.inner.set_ratio(ratio.into());
            };
            let attack_response = ui.add(
                Slider::new(&mut attack, Normal::range())
                    .fixed_decimals(2)
                    .text("Attack"),
            );
            if attack_response.changed() {
                self.inner.set_attack(attack.into());
            };
            let release_response = ui.add(
                Slider::new(&mut release, Normal::range())
                    .fixed_decimals(2)
                    .text("Release"),
            );
            if release_response.changed() {
                self.inner.set_release(release.into());
            };
            threshold_response | ratio_response | attack_response | release_response
        }
    }

    impl Displays for Limiter {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut min = self.inner.minimum().to_percentage();
            let mut max = self.inner.maximum().to_percentage();
            let min_response = ui.add(
                Slider::new(&mut min, 0.0..=max)
                    .suffix(" %")
                    .text("min")
                    .fixed_decimals(2),
            );
            if min_response.changed() {
                self.inner.set_minimum(min.into());
            };
            let max_response = ui.add(
                Slider::new(&mut max, min..=1.0)
                    .suffix(" %")
                    .text("max")
                    .fixed_decimals(2),
            );
            if max_response.changed() {
                self.inner.set_maximum(Normal::from_percentage(max));
            };
            min_response | max_response
        }
    }
}
