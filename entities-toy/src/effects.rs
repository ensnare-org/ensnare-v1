// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_cores::toys::ToyEffectParams;
use ensnare_egui_widgets::drag_normal;
use ensnare_entity::traits::Displays;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEffect, Metadata,
};

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct ToyEffect {
    uid: Uid,

    inner: ensnare_cores::toys::ToyEffect,
}
impl Displays for ToyEffect {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(drag_normal(&mut self.inner.magnitude, "Magnitude: "))
    }
}
impl ToyEffect {
    pub fn new_with(uid: Uid, params: &ToyEffectParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::toys::ToyEffect::new_with(params),
        }
    }
}
