// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{cores::instruments::DrumkitCore, prelude::*};
use eframe::egui::Widget;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum SamplerWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

#[derive(Debug, Display)]
pub enum DrumkitWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

#[derive(Debug)]
pub struct DrumkitWidget<'a> {
    inner: &'a mut DrumkitCore,
    action: &'a mut Option<DrumkitWidgetAction>,
}
impl<'a> DrumkitWidget<'a> {
    fn new(inner: &'a mut DrumkitCore, action: &'a mut Option<DrumkitWidgetAction>) -> Self {
        Self { inner, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        inner: &'a mut DrumkitCore,
        action: &'a mut Option<DrumkitWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| DrumkitWidget::new(inner, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for DrumkitWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(format!("Name: {:?}", self.inner.name()))
    }
}
