// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::MidiSettings;
use eframe::egui::{CollapsingHeader, ComboBox, Widget};
use ensnare_core::midi_interface::MidiPortDescriptor;

/// Wraps a [MidiSettingsWidget] as a [Widget](eframe::egui::Widget). Mutates the given view_range.
pub fn midi_settings<'a>(
    settings: &'a mut MidiSettings,
    inputs: &'a [MidiPortDescriptor],
    outputs: &'a [MidiPortDescriptor],
    new_input: &'a mut Option<MidiPortDescriptor>,
    new_output: &'a mut Option<MidiPortDescriptor>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        MidiSettingsWidget::new_with(settings, inputs, outputs, new_input, new_output).ui(ui)
    }
}

#[derive(Debug)]
struct MidiSettingsWidget<'a> {
    pub(crate) settings: &'a mut MidiSettings,
    inputs: &'a [MidiPortDescriptor],
    outputs: &'a [MidiPortDescriptor],
    new_input: &'a mut Option<MidiPortDescriptor>,
    new_output: &'a mut Option<MidiPortDescriptor>,
}
impl<'a> MidiSettingsWidget<'a> {
    pub fn new_with(
        settings: &'a mut MidiSettings,
        inputs: &'a [MidiPortDescriptor],
        outputs: &'a [MidiPortDescriptor],
        new_input: &'a mut Option<MidiPortDescriptor>,
        new_output: &'a mut Option<MidiPortDescriptor>,
    ) -> Self {
        Self {
            settings,
            inputs,
            outputs,
            new_input,
            new_output,
        }
    }
}
impl<'a> eframe::egui::Widget for MidiSettingsWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        CollapsingHeader::new("MIDI")
            .default_open(true)
            .show(ui, |ui| {
                let mut cb = ComboBox::from_label("MIDI in");
                let (mut selected_index, _selected_text) =
                    if let Some(selected) = &self.settings.selected_input {
                        cb = cb.selected_text(selected.name.clone());
                        (selected.index, selected.name.as_str())
                    } else {
                        (usize::MAX, "None")
                    };
                cb.show_ui(ui, |ui| {
                    for port in self.inputs.iter() {
                        if ui
                            .selectable_value(&mut selected_index, port.index, port.name.clone())
                            .changed()
                        {
                            self.settings.set_input(Some(port.clone()));
                            *self.new_input = Some(port.clone());
                        }
                    }
                });
                ui.end_row();

                let mut cb = ComboBox::from_label("MIDI out");
                let (mut selected_index, _selected_text) =
                    if let Some(selected) = &self.settings.selected_output {
                        cb = cb.selected_text(selected.name.clone());
                        (selected.index, selected.name.as_str())
                    } else {
                        (usize::MAX, "None")
                    };
                cb.show_ui(ui, |ui| {
                    for port in self.outputs.iter() {
                        if ui
                            .selectable_value(&mut selected_index, port.index, port.name.clone())
                            .changed()
                        {
                            self.settings.set_output(Some(port.clone()));
                            *self.new_output = Some(port.clone());
                        }
                    }
                });
                ui.end_row();
            })
            .header_response
    }
}
