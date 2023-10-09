// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    drag_drop::{DragDropManager, DragDropSource},
    modulators::{Dca, DcaAction},
    traits::prelude::*,
    types::{BipolarNormal, Normal},
};
use eframe::egui::Slider;

/// Wraps a [DcaWidget] as a [Widget](eframe::egui::Widget).
pub fn dca<'a>(dca: &'a mut Dca) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| DcaWidget::new(dca).ui(ui)
}

/// An egui widget for [Dca].
#[derive(Debug)]
struct DcaWidget<'a> {
    dca: &'a mut Dca,
}
impl<'a> Displays for DcaWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut drop_index = None;
        let response = {
            let mut value = self.dca.gain().0;
            let response = DragDropManager::drop_target(ui, true, |ui| {
                ui.add(Slider::new(&mut value, Normal::range()).text("Gain"))
            })
            .inner;
            if DragDropManager::is_dropped(ui, &response) {
                drop_index = Some(self.dca.control_index_for_name("gain").unwrap());
            }
            ui.end_row();
            if response.changed() {
                self.dca.set_gain(Normal::from(value));
            }
            response
        } | {
            let mut value = self.dca.pan().0;
            let response = DragDropManager::drop_target(ui, true, |ui| {
                ui.add(Slider::new(&mut value, BipolarNormal::range()).text("Pan (L-R)"))
            })
            .inner;
            if DragDropManager::is_dropped(ui, &response) {
                drop_index = Some(self.dca.control_index_for_name("pan").unwrap());
            }
            ui.end_row();
            if response.changed() {
                self.dca.set_pan(BipolarNormal::from(value));
            }
            response
        };
        if let Some(index) = drop_index {
            if let Some(source) = DragDropManager::source() {
                match source {
                    DragDropSource::ControlSource(source_uid) => {
                        self.dca
                            .set_action(DcaAction::LinkControl(source_uid, index));
                    }
                    _ => {}
                }
            }
        }

        response
    }
}
impl<'a> DcaWidget<'a> {
    fn new(dca: &'a mut Dca) -> Self {
        Self { dca }
    }
}
