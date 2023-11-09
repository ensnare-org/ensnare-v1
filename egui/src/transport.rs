// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use ensnare_core::time::{Tempo, Transport};
use ensnare_entity::traits::Displays;

/// Wraps a [Transport] as a [Widget](eframe::egui::Widget).
pub fn transport(transport: &mut Transport) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| TransportWidget::new(transport).ui(ui)
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
        ui.horizontal_centered(|ui| {
            ui.add(
                eframe::egui::DragValue::new(&mut self.transport.tempo.0)
                    .clamp_range(Tempo::range())
                    .min_decimals(1)
                    .speed(0.1)
                    .suffix(" BPM"),
            ) | ui.add(eframe::egui::Label::new(
                eframe::egui::RichText::new(format!("{}", self.transport.current_time()))
                    .text_style(eframe::egui::TextStyle::Monospace),
            ))
        })
        .inner
    }
}
