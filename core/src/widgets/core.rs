// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{prelude::*, time::Transport, traits::prelude::*, types::Normal};
use eframe::{
    egui::{DragValue, Label, Layout, RichText, TextStyle},
    emath::Align,
    epaint::vec2,
};

/// Wraps a [Transport] as a [Widget](eframe::egui::Widget).
pub fn transport(transport: &mut Transport) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| TransportWidget::new(transport).ui(ui)
}

/// Wraps a [DragNormal] as a [Widget](eframe::egui::Widget).
pub fn drag_normal<'a>(normal: &'a mut Normal, prefix: &'a str) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| DragNormal::new(normal).prefix(prefix).ui(ui)
}

#[derive(Debug)]
struct TransportWidget<'a> {
    transport: &'a mut Transport,
}
impl<'a> TransportWidget<'a> {
    fn new(transport: &'a mut Transport) -> Self {
        Self { transport }
    }
}
impl<'a> Displays for TransportWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.allocate_ui(vec2(72.0, 20.0), |ui| {
            ui.set_min_width(128.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(
                    DragValue::new(&mut self.transport.tempo.0)
                        .clamp_range(Tempo::range())
                        .min_decimals(1)
                        .speed(0.1)
                        .suffix(" BPM"),
                )
            })
            .inner
        })
        .inner
            | ui.allocate_ui(vec2(72.0, 20.0), |ui| {
                ui.set_min_width(128.0);
                ui.add(Label::new(
                    RichText::new(format!("{}", self.transport.current_time()))
                        .text_style(TextStyle::Monospace),
                ));
            })
            .response
    }
}

/// An egui widget that makes it easier to work with a [DragValue] and a Normal.
pub struct DragNormal<'a> {
    normal: &'a mut Normal,
    prefix: Option<String>,
}
impl<'a> Displays for DragNormal<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut value = self.normal.value() * 100.0;
        let mut dv = DragValue::new(&mut value)
            .clamp_range(0.0..=100.0)
            .suffix("%");
        if let Some(prefix) = &self.prefix {
            dv = dv.prefix(prefix);
        }
        let response = ui.add(dv);
        if response.changed() {
            *self.normal = Normal::from(value / 100.0);
        }
        response
    }
}
impl<'a> DragNormal<'a> {
    pub fn new(normal: &'a mut Normal) -> Self {
        Self {
            normal,
            prefix: None,
        }
    }

    pub fn prefix(mut self, prefix: impl ToString) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }
}
