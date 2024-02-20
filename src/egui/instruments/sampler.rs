// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{cores::instruments::SamplerCore, prelude::*};
use eframe::egui::Widget;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum SamplerWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

#[derive(Debug)]
pub struct SamplerWidget<'a> {
    inner: &'a mut SamplerCore,
    action: &'a mut Option<SamplerWidgetAction>,
}
impl<'a> SamplerWidget<'a> {
    fn new(inner: &'a mut SamplerCore, action: &'a mut Option<SamplerWidgetAction>) -> Self {
        Self { inner, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        inner: &'a mut SamplerCore,
        action: &'a mut Option<SamplerWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| SamplerWidget::new(inner, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for SamplerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(format!("Filename: {:?}", self.inner.path()))
    }
}
