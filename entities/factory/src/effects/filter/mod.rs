// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_cores_egui::effects::bi_quad_filter_low_pass_24db;
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity2, Metadata,
};
use serde::{Deserialize, Serialize};

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
#[entity2(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]

pub struct BiQuadFilterLowPass24db {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::BiQuadFilterLowPass24db,
}
impl Displays for BiQuadFilterLowPass24db {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(bi_quad_filter_low_pass_24db(&mut self.inner))
    }
}
impl BiQuadFilterLowPass24db {
    pub fn new_with(uid: Uid, cutoff: FrequencyHz, passband_ripple: ParameterType) -> Self {
        Self {
            uid,
            inner: ensnare_cores::BiQuadFilterLowPass24db::new_with(cutoff, passband_ripple),
        }
    }
}
