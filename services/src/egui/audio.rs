// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::AudioSettings;
use eframe::egui::{CollapsingHeader, Widget};

/// Wraps an [AudioSettingsWidget] as a [Widget](eframe::egui::Widget). Mutates the given view_range.
pub fn audio_settings(settings: &mut AudioSettings) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| AudioSettingsWidget::new_with(settings).ui(ui)
}

#[derive(Debug)]
struct AudioSettingsWidget<'a> {
    settings: &'a mut AudioSettings,
}
impl<'a> AudioSettingsWidget<'a> {
    pub fn new_with(settings: &'a mut AudioSettings) -> Self {
        Self { settings }
    }
}
impl<'a> eframe::egui::Widget for AudioSettingsWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        CollapsingHeader::new("Audio")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!("Sample rate: {}", self.settings.sample_rate()));
                ui.label(format!("Channels: {}", self.settings.channel_count()));
            })
            .header_response
    }
}
