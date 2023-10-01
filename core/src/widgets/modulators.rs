// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::modulators::Dca;
use crate::traits::prelude::*;
use crate::types::{BipolarNormal, Normal};
use eframe::egui::Slider;

/// Wraps a [DcaWidget] as a [Widget](eframe::egui::Widget).
pub fn dca<'a>(dca: &'a mut Dca) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| DcaWidget::new(dca).ui(ui)
}

/// An egui widget for [Dca].
#[derive(Debug)]
pub struct DcaWidget<'a> {
    dca: &'a mut Dca,
}
impl<'a> Displays for DcaWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = {
            let mut value = self.dca.gain().0;
            ui.label("Gain: ");
            let response = ui.add(Slider::new(&mut value, Normal::range()));
            if response.changed() {
                self.dca.set_gain(Normal::from(value));
            }
            response
        } | {
            let mut value = self.dca.pan().0;
            ui.end_row();
            ui.label("Pan (L-R)");
            let response = ui.add(Slider::new(&mut value, BipolarNormal::range()));
            if response.changed() {
                self.dca.set_pan(BipolarNormal::from(value));
            }
            response
        };
        response
    }
}
impl<'a> DcaWidget<'a> {
    fn new(dca: &'a mut Dca) -> Self {
        Self { dca }
    }
}
