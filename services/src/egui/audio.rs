// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::AudioSettings;
use eframe::egui::Widget;

#[derive(Debug)]
pub struct AudioSettingsWidget<'a> {
    settings: &'a mut AudioSettings,
}
impl<'a> AudioSettingsWidget<'a> {
    fn new_with(settings: &'a mut AudioSettings) -> Self {
        Self { settings }
    }

    pub fn widget(settings: &mut AudioSettings) -> impl eframe::egui::Widget + '_ {
        move |ui: &mut eframe::egui::Ui| AudioSettingsWidget::new_with(settings).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for AudioSettingsWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(format!("Sample rate: {}", self.settings.sample_rate()))
            | ui.label(format!("Channels: {}", self.settings.channel_count()))
    }
}
