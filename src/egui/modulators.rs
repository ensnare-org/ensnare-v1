// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::{DragDropManager, DropTarget};
use crate::prelude::*;
use eframe::egui::{Slider, Widget};

/// An egui widget for [Dca].
#[derive(Debug)]
pub struct DcaWidget<'a> {
    dca: &'a mut Dca,
    controllable_uid: Uid,
}
impl<'a> eframe::egui::Widget for DcaWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = {
            let mut value = self.dca.gain().0;
            let response = DragDropManager::drop_target(ui, true, |ui| {
                (
                    ui.add(Slider::new(&mut value, Normal::range()).text("Gain")),
                    DropTarget::Controllable(self.controllable_uid, Dca::GAIN_INDEX.into()),
                )
            })
            .inner;
            ui.end_row();
            if response.changed() {
                self.dca.set_gain(Normal::from(value));
            }
            response
        } | {
            let mut value = self.dca.pan().0;
            let response = DragDropManager::drop_target(ui, true, |ui| {
                (
                    ui.add(Slider::new(&mut value, BipolarNormal::range()).text("Pan (L-R)")),
                    DropTarget::Controllable(self.controllable_uid, Dca::PAN_INDEX.into()),
                )
            })
            .inner;
            ui.end_row();
            if response.changed() {
                self.dca.set_pan(BipolarNormal::from(value));
            }
            response
        };

        response
    }
}
impl<'a> DcaWidget<'a> {
    fn new(dca: &'a mut Dca, controllable_uid: Uid) -> Self {
        Self {
            dca,
            controllable_uid,
        }
    }

    pub fn widget(dca: &'a mut Dca, controllable_uid: Uid) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| DcaWidget::new(dca, controllable_uid).ui(ui)
    }
}
