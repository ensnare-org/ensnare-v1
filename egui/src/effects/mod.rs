// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    entities::{
        effects::{
            bitcrusher::BitcrusherParams, compressor::CompressorParams, limiter::LimiterParams,
            mixer::MixerParams,
        },
        prelude::{BiQuadFilterLowPass24dbParams, ChorusParams, GainParams, ReverbParams},
    },
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEffect, Metadata,
};

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct Bitcrusher {
    uid: Uid,
    inner: ensnare_core::entities::effects::bitcrusher::Bitcrusher,
}
impl Displays for Bitcrusher {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Bitcrusher {
    pub fn new_with(uid: Uid, params: &BitcrusherParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::entities::effects::bitcrusher::Bitcrusher::new_with(&params),
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
    IsEffect,
    Metadata,
)]
pub struct Chorus {
    uid: Uid,
    inner: ensnare_core::entities::effects::chorus::Chorus,
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
            inner: ensnare_core::entities::effects::chorus::Chorus::new_with(&params),
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
    IsEffect,
    Metadata,
)]
pub struct Compressor {
    uid: Uid,
    inner: ensnare_core::entities::effects::compressor::Compressor,
}
impl Displays for Compressor {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Compressor {
    pub fn new_with(uid: Uid, params: &CompressorParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::entities::effects::compressor::Compressor::new_with(&params),
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
    IsEffect,
    Metadata,
)]
pub struct Gain {
    uid: Uid,
    inner: ensnare_core::entities::effects::gain::Gain,
}
impl Displays for Gain {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Gain {
    pub fn new_with(uid: Uid, params: &GainParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::entities::effects::gain::Gain::new_with(&params),
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
    IsEffect,
    Metadata,
)]
pub struct Limiter {
    uid: Uid,
    inner: ensnare_core::entities::effects::limiter::Limiter,
}
impl Displays for Limiter {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Limiter {
    pub fn new_with(uid: Uid, params: &LimiterParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::entities::effects::limiter::Limiter::new_with(&params),
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
    IsEffect,
    Metadata,
)]
pub struct Mixer {
    uid: Uid,
    inner: ensnare_core::entities::effects::mixer::Mixer,
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
            inner: ensnare_core::entities::effects::mixer::Mixer::new_with(&params),
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
    IsEffect,
    Metadata,
)]
pub struct Reverb {
    uid: Uid,
    inner: ensnare_core::entities::effects::reverb::Reverb,
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
            inner: ensnare_core::entities::effects::reverb::Reverb::new_with(&params),
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
    IsEffect,
    Metadata,
)]
pub struct BiQuadFilterLowPass24db {
    uid: Uid,
    inner: ensnare_core::entities::effects::filter::BiQuadFilterLowPass24db,
}
impl Displays for BiQuadFilterLowPass24db {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl BiQuadFilterLowPass24db {
    pub fn new_with(uid: Uid, params: &BiQuadFilterLowPass24dbParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::entities::effects::filter::BiQuadFilterLowPass24db::new_with(
                &params,
            ),
        }
    }
}
