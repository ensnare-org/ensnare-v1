// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::composition::Composer;
use eframe::{
    egui::{Frame, Widget},
    emath::{Align2, RectTransform},
    epaint::{pos2, vec2, Color32, FontId, Rect, RectShape, Rounding, Shape, Stroke},
};
use ensnare_core::{
    composition::{Pattern, PatternBuilder},
    midi::MidiNote,
    time::TimeSignature,
    traits::Configurable,
};
use ensnare_cores_egui::widgets::pattern::{CarouselAction, CarouselWidget};
use ensnare_egui_widgets::fill_remaining_ui_space;
use std::ops::RangeInclusive;

#[derive(Debug)]
pub struct ComposerWidget<'a> {
    inner: &'a mut Composer,
}
impl<'a> eframe::egui::Widget for ComposerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            let mut response;
            let new_pattern_response = ui.button("New Pattern");
            if new_pattern_response.clicked() {
                let _ = self.inner.add_pattern(
                    PatternBuilder::default()
                        .time_signature(self.inner.time_signature())
                        .build()
                        .unwrap(),
                    None,
                );
            }
            response = new_pattern_response;
            let mut carousel_action = None;
            let carousel_response = ui.add(CarouselWidget::widget(
                &self.inner.ordered_pattern_uids,
                &self.inner.patterns,
                &mut self.inner.e.pattern_selection_set,
                &mut carousel_action,
            ));
            if let Some(action) = carousel_action {
                match action {
                    CarouselAction::DeletePattern(pattern_uid) => {
                        let _ = self.inner.remove_pattern(pattern_uid);
                    }
                }
            }
            response |= carousel_response;
            // if let Some(pattern_uid) = self
            //     .inner
            //     .e
            //     .pattern_selection_set
            //     .single_selection()
            //     .cloned()
            // {
            //     if let Some(pattern) = self.inner.pattern_mut(pattern_uid) {
            //         ui.label(format!("Time Signature: {}", pattern.time_signature()));
            //         let pattern_edit_response = {
            //             ui.set_min_height(256.0);
            //             let desired_size = vec2(ui.available_width(), 96.0);
            //             let (_id, rect) = ui.allocate_space(desired_size);
            //             ui.add_enabled_ui(false, |ui| {
            //                 ui.allocate_ui_at_rect(rect, |ui| {
            //                     ui.add(GridWidget::widget(pattern.duration))
            //                 })
            //                 .inner
            //             });
            //             ui.allocate_ui_at_rect(rect, |ui| ui.add(PatternWidget::widget(pattern)))
            //                 .inner
            //         };
            //         response |= pattern_edit_response;
            //     }
            // }
            response |= ui.add(ComposerEditorWidget::widget(self.inner));

            response
        })
        .inner
    }
}
impl<'a> ComposerWidget<'a> {
    fn new(inner: &'a mut Composer) -> Self {
        Self { inner }
    }

    pub fn widget(inner: &'a mut Composer) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| ComposerWidget::new(inner).ui(ui)
    }
}

#[derive(Debug)]
pub struct ComposerEditorWidget<'a> {
    composer: &'a mut Composer,
}
impl<'a> eframe::egui::Widget for ComposerEditorWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if let Some(pattern_uid) = self
            .composer
            .e
            .pattern_selection_set
            .single_selection()
            .cloned()
        {
            if let Some(pattern) = self.composer.pattern_mut(pattern_uid) {
                // inner_margin() should be half of the Frame stroke width to leave room
                // for it. Thanks vikrinox on the egui Discord.
                Frame::default()
                    .inner_margin(eframe::egui::Margin::same(2.0 / 2.0))
                    .stroke(Stroke {
                        width: 2.0,
                        color: Color32::YELLOW,
                    })
                    .show(ui, |ui| {
                        // Draw top bar
                        ui.label(format!("Time Signature: {}", pattern.time_signature()));
                        let available_size = ui.available_size();
                        let (_id, rect) =
                            ui.allocate_space(vec2(available_size.x, available_size.y - 5.0));
                        let mut rect = rect;
                        *rect.left_mut() += 16.0;
                        *rect.top_mut() += 16.0;

                        // Overlay the grid
                        let min_note = pattern
                            .notes
                            .iter()
                            .map(|note| note.key)
                            .min()
                            .unwrap_or(MidiNote::C3 as u8);
                        let max_note = pattern
                            .notes
                            .iter()
                            .map(|note| note.key)
                            .max()
                            .unwrap_or(MidiNote::C3 as u8);
                        let min_window = if min_note < MidiNote::C0 as u8 {
                            MidiNote::CSub0 as u8
                        } else {
                            min_note - 12
                        };
                        let max_window = if max_note > MidiNote::G8 as u8 {
                            MidiNote::G9 as u8
                        } else {
                            max_note + 12
                        };
                        ui.add_enabled_ui(false, |ui| {
                            ui.allocate_ui_at_rect(rect, |ui| {
                                ui.add(PatternGridWidget::widget(
                                    pattern.time_signature(),
                                    min_window..=max_window,
                                ))
                            })
                        });

                        // Draw the note content
                        let response = ui
                            .allocate_ui_at_rect(rect, |ui| {
                                ui.add(PatternWidget::widget(pattern, min_window..=max_window))
                            })
                            .inner;

                        //   let response = ui.allocate_ui_at_rect(rect, |ui| ui.label("hi mom")).inner;

                        fill_remaining_ui_space(ui);
                        response
                    })
                    .inner
            } else {
                ui.label("huh?")
            }
        } else {
            ui.label("Select one to see editor")
        }
    }
}

impl<'a> ComposerEditorWidget<'a> {
    pub fn widget(composer: &'a mut Composer) -> impl eframe::egui::Widget + 'a {
        let w = Self { composer };
        move |ui: &mut eframe::egui::Ui| w.ui(ui)
    }
}

#[derive(Debug)]
pub struct PatternGridWidget {
    time_signature: TimeSignature,
    note_range: RangeInclusive<u8>,
}
impl PatternGridWidget {
    fn time_signature(mut self, time_signature: TimeSignature) -> Self {
        self.time_signature = time_signature;
        self
    }

    fn note_range(mut self, note_range: RangeInclusive<u8>) -> Self {
        self.note_range = note_range;
        self
    }

    pub fn widget(
        time_signature: TimeSignature,
        note_range: RangeInclusive<u8>,
    ) -> impl eframe::egui::Widget {
        move |ui: &mut eframe::egui::Ui| {
            Self {
                time_signature,
                note_range,
            }
            .ui(ui)
        }
    }
}
impl eframe::egui::Widget for PatternGridWidget {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::hover());
        let sections = self.time_signature.bottom * 4;
        let first_note = *self.note_range.start();
        let last_note = *self.note_range.end();
        let first_note_f32 = first_note as f32;
        let last_note_f32 = last_note as f32;
        let note_range_f32 = last_note_f32..=first_note_f32;
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(0.0..=sections as f32, note_range_f32),
            rect,
        );
        let visuals = ui.ctx().style().visuals.widgets.noninteractive;

        let mut shapes = vec![Shape::Rect(RectShape::filled(
            rect,
            visuals.rounding,
            visuals.bg_fill,
        ))];

        // Draw the horizontal note dividers.
        for key in self.note_range {
            let left = to_screen * pos2(0.0, key as f32);
            let right = to_screen * pos2(sections as f32, key as f32);
            let stroke = if (key - MidiNote::C0 as u8) % 12 == 0 {
                visuals.fg_stroke
            } else {
                visuals.bg_stroke
            };
            shapes.push(Shape::LineSegment {
                points: [left, right],
                stroke,
            });
            ui.painter().text(
                left,
                Align2::RIGHT_BOTTOM,
                format!("{key}"),
                FontId::monospace(12.0),
                Color32::RED,
            );
        }

        // Draw the vertical note dividers.
        for part in 0..sections {
            let x = part as f32;
            let stroke = if part % self.time_signature.bottom == 0 {
                visuals.fg_stroke
            } else {
                visuals.bg_stroke
            };
            let line_start = to_screen * pos2(x, first_note_f32);
            let line_end = to_screen * pos2(x, last_note_f32);
            let line_middle = to_screen * pos2(x + 0.5, last_note_f32);
            shapes.push(Shape::LineSegment {
                points: [line_start, line_end],
                stroke,
            });
            ui.painter().text(
                line_middle,
                Align2::CENTER_BOTTOM,
                format!("{part}"),
                FontId::monospace(12.0),
                Color32::YELLOW,
            );
        }
        ui.painter().extend(shapes);

        response
    }
}

#[derive(Debug)]
pub struct PatternWidget<'a> {
    pattern: &'a mut Pattern,
    note_range: RangeInclusive<u8>,
}
impl<'a> PatternWidget<'a> {
    fn pattern(mut self, pattern: &'a mut Pattern) -> Self {
        self.pattern = pattern;
        self
    }

    fn note_range(mut self, note_range: RangeInclusive<u8>) -> Self {
        self.note_range = note_range;
        self
    }

    pub fn widget(
        pattern: &'a mut Pattern,
        note_range: RangeInclusive<u8>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| {
            Self {
                pattern,
                note_range,
            }
            .ui(ui)
        }
    }
}
impl<'a> eframe::egui::Widget for PatternWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::hover());
        let sections = self.pattern.time_signature().bottom * 4;
        let first_note = *self.note_range.start();
        let last_note = *self.note_range.end();
        let first_note_f32 = first_note as f32;
        let last_note_f32 = last_note as f32;
        let note_range_f32 = last_note_f32..=first_note_f32;
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(0.0..=sections as f32, note_range_f32),
            rect,
        );
        let mut shapes = Vec::default();

        // The `/ 4` is correct because a part is a 16th of a beat, and for this
        // rigid pattern widget, we're using only quarter-beat divisions.
        for note in self.pattern.notes.iter() {
            let ul = to_screen
                * pos2(
                    (note.range.0.start.total_parts() / 4) as f32,
                    note.key as f32,
                );
            let br = to_screen
                * pos2(
                    (note.range.0.end.total_parts() / 4) as f32,
                    (note.key + 1) as f32,
                );
            let note_rect = Rect::from_two_pos(ul, br);
            shapes.push(Shape::Rect(RectShape::filled(
                note_rect,
                Rounding::default(),
                Color32::YELLOW,
            )));
        }

        ui.painter().extend(shapes);

        response
    }
}
