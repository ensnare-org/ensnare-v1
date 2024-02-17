// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use eframe::egui::{Frame, Slider, Widget};
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum DcaWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

/// An egui widget for [Dca].
#[derive(Debug)]
pub struct DcaWidget<'a> {
    dca: &'a mut Dca,
    action: &'a mut Option<DcaWidgetAction>,
}
impl<'a> eframe::egui::Widget for DcaWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = {
            let mut value = self.dca.gain().0;
            let (inner_response, response, payload) = {
                // See https://github.com/emilk/egui/issues/4059 for why this
                // code is a bit cumbersome
                let mut inner_response = None;
                let (mut response, payload) =
                    ui.dnd_drop_zone::<ControlLinkSource>(Frame::default(), |ui| {
                        inner_response =
                            Some(ui.add(Slider::new(&mut value, Normal::range()).text("Gain")));
                    });
                if let Some(inner_response) = inner_response.as_ref() {
                    if inner_response.changed() {
                        response.mark_changed();
                    }
                }
                (inner_response, response, payload)
            };
            if let Some(source) = payload {
                *self.action = Some(DcaWidgetAction::Link(*source, Dca::GAIN_INDEX.into()));
            }
            ui.end_row();
            if let Some(inner_response) = inner_response {
                if inner_response.changed() {
                    self.dca.set_gain(Normal::from(value));
                }
            }
            response
        } | {
            let mut value = self.dca.pan().0;
            let (inner_response, response, payload) = {
                let mut inner_response = None;
                let (mut response, payload) =
                    ui.dnd_drop_zone::<ControlLinkSource>(Frame::default(), |ui| {
                        inner_response = Some(ui.add(
                            Slider::new(&mut value, BipolarNormal::range()).text("Pan (L-R)"),
                        ));
                    });
                if let Some(inner_response) = inner_response.as_ref() {
                    if inner_response.changed() {
                        response.mark_changed();
                    }
                }
                (inner_response, response, payload)
            };
            if let Some(source) = payload {
                *self.action = Some(DcaWidgetAction::Link(*source, Dca::PAN_INDEX.into()));
            }
            ui.end_row();
            if let Some(inner_response) = inner_response {
                if inner_response.changed() {
                    self.dca.set_pan(BipolarNormal::from(value));
                }
            }
            response
        };

        response
    }
}
impl<'a> DcaWidget<'a> {
    fn new(dca: &'a mut Dca, action: &'a mut Option<DcaWidgetAction>) -> Self {
        Self { dca, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        dca: &'a mut Dca,
        action: &'a mut Option<DcaWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| DcaWidget::new(dca, action).ui(ui)
    }
}
