// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::Slider;
use ensnare_core::prelude::*;
use ensnare_cores::{
    BitcrusherParams, ChorusParams, CompressorParams, GainParams, LimiterParams, MixerParams,
    ReverbParams,
};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity2, Metadata,
};
use serde::{Deserialize, Serialize};
pub mod filter;

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controls, GeneratesStereoSample, HandlesMidi, SkipInner, Ticks)]
pub struct Bitcrusher {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Bitcrusher,
}
impl Displays for Bitcrusher {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut bits = self.inner.bits();
        let response =
            ui.add(Slider::new(&mut bits, ensnare_cores::Bitcrusher::bits_range()).suffix(" bits"));
        if response.changed() {
            self.inner.set_bits(bits);
        };
        response
    }
}
impl Bitcrusher {
    pub fn new_with(uid: Uid, params: &BitcrusherParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Bitcrusher::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controls, GeneratesStereoSample, HandlesMidi, SkipInner, Ticks)]

pub struct Chorus {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Chorus,
}
impl Displays for Chorus {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Chorus {
    pub fn new_with(uid: Uid, params: &ChorusParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Chorus::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controls, GeneratesStereoSample, HandlesMidi, SkipInner, Ticks)]

pub struct Compressor {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Compressor,
}
impl Displays for Compressor {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut threshold = self.inner.threshold().0;
        let mut ratio = self.inner.ratio();
        let mut attack = self.inner.attack();
        let mut release = self.inner.release();
        let threshold_response = ui.add(
            Slider::new(&mut threshold, Normal::range())
                .fixed_decimals(2)
                .text("Threshold"),
        );
        if threshold_response.changed() {
            self.inner.set_threshold(threshold.into());
        };
        let ratio_response = ui.add(
            Slider::new(&mut ratio, Normal::range())
                .fixed_decimals(2)
                .text("Ratio"),
        );
        if ratio_response.changed() {
            self.inner.set_ratio(ratio);
        };
        let attack_response = ui.add(
            Slider::new(&mut attack, Normal::range())
                .fixed_decimals(2)
                .text("Attack"),
        );
        if attack_response.changed() {
            self.inner.set_attack(attack);
        };
        let release_response = ui.add(
            Slider::new(&mut release, Normal::range())
                .fixed_decimals(2)
                .text("Release"),
        );
        if release_response.changed() {
            self.inner.set_release(release);
        };
        threshold_response | ratio_response | attack_response | release_response
    }
}
impl Compressor {
    pub fn new_with(uid: Uid, params: &CompressorParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Compressor::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controls, GeneratesStereoSample, HandlesMidi, SkipInner, Ticks)]

pub struct Gain {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Gain,
}
impl Displays for Gain {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut ceiling = self.inner.ceiling().to_percentage();
        let response = ui.add(
            Slider::new(&mut ceiling, 0.0..=100.0)
                .fixed_decimals(2)
                .suffix(" %")
                .text("Ceiling"),
        );
        if response.changed() {
            self.inner.set_ceiling(Normal::from_percentage(ceiling));
        };
        response
    }
}
impl Gain {
    pub fn new_with(uid: Uid, params: &GainParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Gain::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controls, GeneratesStereoSample, HandlesMidi, SkipInner, Ticks)]

pub struct Limiter {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Limiter,
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
impl Limiter {
    pub fn new_with(uid: Uid, params: &LimiterParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Limiter::new_with(&params),
        }
    }
}

#[derive(
    Debug, Default, InnerControllable, InnerEffect, IsEntity2, Metadata, Serialize, Deserialize,
)]
#[entity2(
    Configurable,
    Controls,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    SkipInner,
    Ticks
)]

pub struct Mixer {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Mixer,
}
impl Displays for Mixer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Mixer {
    pub fn new_with(uid: Uid, params: &MixerParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Mixer::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controls, GeneratesStereoSample, HandlesMidi, SkipInner, Ticks)]

pub struct Reverb {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Reverb,
}
impl Displays for Reverb {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Reverb {
    pub fn new_with(uid: Uid, params: &ReverbParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Reverb::new_with(&params),
        }
    }
}
