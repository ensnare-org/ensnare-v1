// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::{FrequencyWidget, WaveformWidget};
use crate::{
    composition::{NoteSequencer, PatternSequencer},
    cores::controllers::{ArpeggiatorCore, ArpeggioMode},
    prelude::*,
};
use eframe::egui::Widget;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct PatternSequencerWidget<'a> {
    sequencer: &'a mut PatternSequencer,
    view_range: ViewRange,
}
impl<'a> PatternSequencerWidget<'a> {
    fn new(sequencer: &'a mut PatternSequencer, view_range: &'a ViewRange) -> Self {
        Self {
            sequencer,
            view_range: view_range.clone(),
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        sequencer: &'a mut PatternSequencer,
        view_range: &'a ViewRange,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| PatternSequencerWidget::new(sequencer, view_range).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for PatternSequencerWidget<'a> {
    fn ui(self, _ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        todo!()
    }
}

/// An egui widget that draws a legend on the horizontal axis of the timeline
/// view.
#[derive(Debug)]
pub struct NoteSequencerWidget<'a> {
    sequencer: &'a mut NoteSequencer,
    view_range: ViewRange,
}
impl<'a> NoteSequencerWidget<'a> {
    fn new(sequencer: &'a mut NoteSequencer, view_range: &'a ViewRange) -> Self {
        Self {
            sequencer,
            view_range: view_range.clone(),
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        sequencer: &'a mut NoteSequencer,
        view_range: &'a ViewRange,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| NoteSequencerWidget::new(sequencer, view_range).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for NoteSequencerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}

/// Renders [Arpeggiator] in egui.
#[derive(Debug)]
pub struct ArpeggiatorWidget<'a> {
    inner: &'a mut ArpeggiatorCore,
}
impl<'a> ArpeggiatorWidget<'a> {
    fn new(entity: &'a mut ArpeggiatorCore) -> Self {
        Self { inner: entity }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(entity: &'a mut ArpeggiatorCore) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| ArpeggiatorWidget::new(entity).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for ArpeggiatorWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut r = eframe::egui::ComboBox::from_label("Scale")
            .selected_text(self.inner.mode.to_string())
            .show_ui(ui, |ui| {
                let mut bool_response = false;
                for mode in ArpeggioMode::iter() {
                    let mode_str: &'static str = mode.into();
                    if ui
                        .selectable_value(&mut self.inner.mode, mode, mode_str)
                        .changed()
                    {
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

#[derive(Debug)]
pub struct LfoControllerWidget<'a> {
    waveform: &'a mut Waveform,
    frequency: &'a mut FrequencyHz,
}
impl<'a> eframe::egui::Widget for LfoControllerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(WaveformWidget::widget(self.waveform))
            | ui.add(FrequencyWidget::widget(
                FrequencyRange::Subaudible,
                self.frequency,
            ))
    }
}
impl<'a> LfoControllerWidget<'a> {
    fn new(waveform: &'a mut Waveform, frequency: &'a mut FrequencyHz) -> Self {
        Self {
            waveform,
            frequency,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        waveform: &'a mut Waveform,
        frequency: &'a mut FrequencyHz,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| LfoControllerWidget::new(waveform, frequency).ui(ui)
    }
}
