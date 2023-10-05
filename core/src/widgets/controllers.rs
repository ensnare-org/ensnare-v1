// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::audio::{frequency, waveform};
use crate::{
    even_smaller_sequencer::ESSequencer, generators::Waveform, piano_roll::Note, prelude::*,
    traits::prelude::*, types::FrequencyRange,
};
use eframe::{
    egui::{style::WidgetVisuals, Sense},
    emath::RectTransform,
    epaint::{pos2, vec2, Rect, RectShape, Shape},
};

/// Wraps an [ESSequencer] as a [Widget](eframe::egui::Widget).
pub fn es_sequencer(sequencer: &mut ESSequencer) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| {
        SequencerWidget::new(sequencer, sequencer.view_range().clone()).ui(ui)
    }
}

/// Wraps an [LfoControllerWidget] as a [Widget](eframe::egui::Widget).
pub fn lfo_controller<'a>(
    waveform: &'a mut Waveform,
    frequency: &'a mut FrequencyHz,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| LfoControllerWidget::new(waveform, frequency).ui(ui)
}

#[derive(Debug)]
struct SequencerWidget<'a> {
    sequencer: &'a mut ESSequencer,
    view_range: std::ops::Range<MusicalTime>,
}
impl<'a> SequencerWidget<'a> {
    fn new(sequencer: &'a mut ESSequencer, view_range: std::ops::Range<MusicalTime>) -> Self {
        Self {
            sequencer,
            view_range,
        }
    }

    fn shape_for_note(
        &self,
        to_screen: &RectTransform,
        visuals: &WidgetVisuals,
        note: &Note,
    ) -> Shape {
        Shape::Rect(RectShape {
            rect: Rect::from_two_pos(
                to_screen * pos2(note.range.start.total_units() as f32, note.key as f32),
                to_screen * pos2(note.range.end.total_units() as f32, note.key as f32),
            ),
            rounding: visuals.rounding,
            fill: visuals.bg_fill,
            stroke: visuals.fg_stroke,
        })
    }
}
impl<'a> DisplaysInTimeline for SequencerWidget<'a> {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl<'a> Displays for SequencerWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.allocate_ui(vec2(ui.available_width(), 64.0), |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
            let x_range_f32 = self.view_range.start.total_units() as f32
                ..=self.view_range.end.total_units() as f32;
            let y_range = i8::MAX as f32..=u8::MIN as f32;
            let local_space_rect = Rect::from_x_y_ranges(x_range_f32, y_range);
            let to_screen = RectTransform::from_to(local_space_rect, response.rect);
            let from_screen = to_screen.inverse();

            // Check whether we edited the sequence
            if response.clicked() {
                if let Some(click_pos) = ui.ctx().pointer_interact_pos() {
                    let local_pos = from_screen * click_pos;
                    let time = MusicalTime::new_with_units(local_pos.x as usize).quantized();
                    let key = local_pos.y as u8;
                    let note = Note::new_with(key, time, MusicalTime::DURATION_QUARTER);
                    eprintln!("Saw a click at {time}, note {note:?}");
                    self.sequencer.toggle_note(note);
                    self.sequencer.calculate_events();
                }
            }

            let visuals = if ui.is_enabled() {
                ui.ctx().style().visuals.widgets.active
            } else {
                ui.ctx().style().visuals.widgets.inactive
            };

            // Generate all the note shapes
            let note_shapes: Vec<Shape> = self
                .sequencer
                .notes()
                .iter()
                .map(|note| self.shape_for_note(&to_screen, &visuals, note))
                .collect();

            // Generate all the pattern note shapes
            let pattern_shapes: Vec<Shape> = self.sequencer.patterns().iter().fold(
                Vec::default(),
                |mut v, (position, pattern)| {
                    pattern.notes().iter().for_each(|note| {
                        let note = Note {
                            key: note.key,
                            range: (note.range.start + *position)..(note.range.end + *position),
                        };
                        v.push(self.shape_for_note(&to_screen, &visuals, &note));
                    });
                    v
                },
            );

            // Paint all the shapes
            painter.extend(note_shapes);
            painter.extend(pattern_shapes);

            response
        })
        .inner
    }
}

#[derive(Debug)]
struct LfoControllerWidget<'a> {
    waveform: &'a mut Waveform,
    frequency: &'a mut FrequencyHz,
}
impl<'a> Displays for LfoControllerWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(frequency(FrequencyRange::Subaudible, self.frequency))
            | ui.add(waveform(self.waveform))
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
