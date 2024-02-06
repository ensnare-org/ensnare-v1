// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    cores::effects,
    egui::{
        BiQuadFilterAllPassWidget, BiQuadFilterBandPassWidget, BiQuadFilterBandStopWidget,
        BiQuadFilterHighPassWidget, BiQuadFilterLowPass24dbWidget,
    },
    prelude::*,
};
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
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterBandPass {
    uid: Uid,
    inner: effects::BiQuadFilterBandPass,
}
impl Displays for BiQuadFilterBandPass {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(BiQuadFilterBandPassWidget::widget(&mut self.inner))
    }
}
impl BiQuadFilterBandPass {
    pub fn new_with(uid: Uid, cutoff: FrequencyHz, bandwidth: ParameterType) -> Self {
        Self {
            uid,
            inner: effects::BiQuadFilterBandPass::new_with(cutoff, bandwidth),
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
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterBandStop {
    uid: Uid,
    inner: effects::BiQuadFilterBandStop,
}
impl Displays for BiQuadFilterBandStop {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(BiQuadFilterBandStopWidget::widget(&mut self.inner))
    }
}
impl BiQuadFilterBandStop {
    pub fn new_with(uid: Uid, cutoff: FrequencyHz, bandwidth: ParameterType) -> Self {
        Self {
            uid,
            inner: effects::BiQuadFilterBandStop::new_with(cutoff, bandwidth),
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
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterLowPass24db {
    uid: Uid,
    inner: effects::BiQuadFilterLowPass24db,
}
impl Displays for BiQuadFilterLowPass24db {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(BiQuadFilterLowPass24dbWidget::widget(&mut self.inner))
    }
}
impl BiQuadFilterLowPass24db {
    pub fn new_with(uid: Uid, cutoff: FrequencyHz, passband_ripple: ParameterType) -> Self {
        Self {
            uid,
            inner: effects::BiQuadFilterLowPass24db::new_with(cutoff, passband_ripple),
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
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterHighPass {
    uid: Uid,
    inner: effects::BiQuadFilterHighPass,
}
impl Displays for BiQuadFilterHighPass {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(BiQuadFilterHighPassWidget::widget(&mut self.inner))
    }
}
impl BiQuadFilterHighPass {
    pub fn new_with(uid: Uid, cutoff: FrequencyHz, q: ParameterType) -> Self {
        Self {
            uid,
            inner: effects::BiQuadFilterHighPass::new_with(cutoff, q),
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
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterAllPass {
    uid: Uid,
    inner: effects::BiQuadFilterAllPass,
}
impl Displays for BiQuadFilterAllPass {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(BiQuadFilterAllPassWidget::widget(&mut self.inner))
    }
}
impl BiQuadFilterAllPass {
    pub fn new_with(uid: Uid, cutoff: FrequencyHz, q: ParameterType) -> Self {
        Self {
            uid,
            inner: effects::BiQuadFilterAllPass::new_with(cutoff, q),
        }
    }
}
