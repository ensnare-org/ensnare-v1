// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::audio::{frequency, waveform};
use crate::{generators::Waveform, prelude::*, traits::prelude::*, types::FrequencyRange};

/// Wraps an [LfoControllerWidget] as a [Widget](eframe::egui::Widget).
pub fn lfo_controller<'a>(
    waveform: &'a mut Waveform,
    frequency: &'a mut FrequencyHz,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| LfoControllerWidget::new(waveform, frequency).ui(ui)
}

#[derive(Debug)]
struct LfoControllerWidget<'a> {
    waveform: &'a mut Waveform,
    frequency: &'a mut FrequencyHz,
}
impl<'a> Displays for LfoControllerWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(waveform(self.waveform))
            | ui.add(frequency(FrequencyRange::Subaudible, self.frequency))
    }
}
impl<'a> LfoControllerWidget<'a> {
    pub fn new(waveform: &'a mut Waveform, frequency: &'a mut FrequencyHz) -> Self {
        Self {
            waveform,
            frequency,
        }
    }
}
