// Copyright (c) 2023 Mike Tsao. All rights reserved.

use core::borrow::Borrow;

use super::{DragSource, DropTarget};
use crate::prelude::*;
use eframe::egui::{Frame, Slider, Widget};

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
            let (response, payload) = ui.dnd_drop_zone::<DragSource>(Frame::default(), |ui| {
                ui.add(Slider::new(&mut value, Normal::range()).text("Gain"));
                // Some(DropTarget::Controllable(
                //     self.controllable_uid,
                //     Dca::GAIN_INDEX.into(),
                // )),
            });
            if let Some(payload) = payload {
                match payload.borrow() {
                    DragSource::NewDevice(_) => todo!(),
                    DragSource::Pattern(_) => todo!(),
                    DragSource::ControlSource(uid) => eprintln!(
                        "connect source {uid} to {}:{}",
                        self.controllable_uid,
                        Dca::GAIN_INDEX
                    ),
                }
            }
            ui.end_row();
            if response.changed() {
                self.dca.set_gain(Normal::from(value));
            }
            response
        } | {
            let mut value = self.dca.pan().0;
            let (response, payload) = ui.dnd_drop_zone::<DragSource>(Frame::default(), |ui| {
                ui.add(Slider::new(&mut value, BipolarNormal::range()).text("Pan (L-R)"));
            });
            if let Some(payload) = payload {
                match payload.borrow() {
                    DragSource::NewDevice(_) => todo!(),
                    DragSource::Pattern(_) => todo!(),
                    DragSource::ControlSource(uid) => eprintln!(
                        "connect source {uid} to {}:{}",
                        self.controllable_uid,
                        Dca::PAN_INDEX
                    ),
                }
            }
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
