// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{egui::DragNormalWidget, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls)]
pub struct ToyEffect {
    uid: Uid,
    #[serde(skip)]
    inner: crate::cores::ToyEffect,
}
impl Displays for ToyEffect {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(DragNormalWidget::widget(
            &mut self.inner.magnitude,
            "Magnitude: ",
        ))
    }
}
impl ToyEffect {
    pub fn new_with(uid: Uid, magnitude: Normal) -> Self {
        Self {
            uid,
            inner: crate::cores::ToyEffect::new_with(magnitude),
        }
    }
}
