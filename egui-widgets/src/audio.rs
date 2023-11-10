// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{ComboBox, Slider, Widget};
use ensnare_core::{
    generators::Waveform,
    types::{FrequencyHz, FrequencyRange},
};
use strum::IntoEnumIterator;

/// A [Widget](eframe::egui::Widget) for picking an oscillator waveform.
pub fn waveform<'a>(waveform: &'a mut Waveform) -> impl Widget + 'a {
    move |ui: &mut eframe::egui::Ui| WaveformWidget::new(waveform).ui(ui)
}

#[derive(Debug)]
struct WaveformWidget<'a> {
    waveform: &'a mut Waveform,
}
impl<'a> WaveformWidget<'a> {
    pub fn new(waveform: &'a mut Waveform) -> Self {
        Self { waveform }
    }
}
impl<'a> Widget for WaveformWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut r = ComboBox::new(ui.next_auto_id(), "Waveform")
            .selected_text(self.waveform.to_string())
            .show_ui(ui, |ui| {
                let mut bool_response = false;
                for w in Waveform::iter() {
                    let s: &'static str = w.into();
                    if ui.selectable_value(self.waveform, w, s).changed() {
                        bool_response = true;
                    }
                }
                bool_response
            });
        if let Some(inner) = r.inner {
            if inner {
                r.response.mark_changed();
            }
        }
        r.response
    }
}

/// A [Widget](eframe::egui::Widget) for picking a frequency.
pub fn frequency<'a>(
    range: FrequencyRange,
    frequency: &'a mut FrequencyHz,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| FrequencyWidget::new(range, frequency).ui(ui)
}

#[derive(Debug)]
struct FrequencyWidget<'a> {
    range: FrequencyRange,
    frequency: &'a mut FrequencyHz,
}
impl<'a> FrequencyWidget<'a> {
    pub fn new(range: FrequencyRange, frequency: &'a mut FrequencyHz) -> Self {
        Self { range, frequency }
    }
}
impl<'a> Widget for FrequencyWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut frequency = self.frequency.0;
        let range = self.range.as_range_frequency_hz();
        let slider = Slider::new(&mut frequency, range.start().0..=range.end().0);
        let response = ui.add(
            slider
                .fixed_decimals(self.range.fixed_digit_count())
                .suffix(FrequencyHz::UNITS_SUFFIX)
                .text("Frequency"),
        );
        if response.changed() {
            *self.frequency = FrequencyHz(frequency);
        }
        response
    }
}
