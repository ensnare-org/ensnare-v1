// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::widgets::pattern::{self, grid};
use eframe::{
    egui::{Sense, Widget},
    emath::RectTransform,
    epaint::{pos2, vec2, Color32, Pos2, Rect, RectShape, Rounding, Shape, Stroke},
};
use ensnare_core::{
    piano_roll::{Note, Pattern, PatternBuilder},
    prelude::*,
};

/// Wraps a [PianoRollWidget] as a [Widget](eframe::egui::Widget).
pub fn piano_roll<'a>(
    entity: &'a mut ensnare_core::piano_roll::PianoRoll,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| PianoRollWidget::new(entity).ui(ui)
}

struct PianoRollWidget<'a> {
    entity: &'a mut ensnare_core::piano_roll::PianoRoll,
}
impl<'a> PianoRollWidget<'a> {
    pub fn new(entity: &'a mut ensnare_core::piano_roll::PianoRoll) -> Self {
        Self { entity }
    }

    fn ui_pattern_edit(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if let Some(pattern_uid) = self.entity.pattern_selection_set.single_selection() {
            ui.set_min_height(192.0);
            if let Some(pat) = self.entity.uids_to_patterns.get_mut(pattern_uid) {
                let desired_size = vec2(ui.available_width(), 96.0);
                let (_id, rect) = ui.allocate_space(desired_size);
                ui.add_enabled_ui(false, |ui| {
                    ui.allocate_ui_at_rect(rect, |ui| ui.add(grid(pat.duration)))
                        .inner
                });
                return ui
                    .allocate_ui_at_rect(rect, |ui| ui.add(pattern(pat)))
                    .inner;
            }
        }

        ui.set_min_height(0.0);
        // This is here so that we can return a Response. I don't know of a
        // better way to do it.
        ui.add_visible_ui(false, |_| {}).response
    }
}
impl<'a> eframe::egui::Widget for PianoRollWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            ui.add(pattern::carousel(
                &self.entity.ordered_pattern_uids,
                &self.entity.uids_to_patterns,
                &mut self.entity.pattern_selection_set,
            ));
            self.ui_pattern_edit(ui);
        })
        .response
    }
}

pub fn pattern<'a>(inner: &'a mut Pattern) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| PatternWidget::new(inner).ui(ui)
}

#[derive(Debug)]
struct PatternWidget<'a> {
    inner: &'a mut Pattern,
}
impl<'a> PatternWidget<'a> {
    pub fn new(inner: &'a mut Pattern) -> Self {
        Self { inner }
    }

    #[allow(dead_code)]
    pub fn make_note_shapes(
        &self,
        note: &Note,
        to_screen: &RectTransform,
        is_selected: bool,
        is_highlighted: bool,
    ) -> Vec<Shape> {
        let rect = to_screen
            .transform_rect(self.rect_for_note(note))
            .shrink(1.0);
        let color = if is_selected {
            Color32::LIGHT_GRAY
        } else if is_highlighted {
            Color32::WHITE
        } else {
            Color32::DARK_BLUE
        };
        let rect = if (rect.right() - rect.left()).abs() < 1.0 {
            Rect::from_two_pos(rect.left_top(), pos2(rect.left() + 1.0, rect.bottom()))
        } else {
            rect
        };
        let rect = if (rect.bottom() - rect.top()).abs() < 1.0 {
            Rect::from_two_pos(rect.left_top(), pos2(rect.right(), rect.top() + 1.0))
        } else {
            rect
        };
        debug_assert!(rect.area() != 0.0);
        vec![Shape::Rect(RectShape::new(
            rect,
            Rounding::default(),
            Color32::LIGHT_BLUE,
            Stroke { width: 2.0, color },
        ))]
    }

    #[allow(dead_code)]
    fn rect_for_note(&self, note: &Note) -> Rect {
        let notes_vert = 24.0;
        const FIGURE_THIS_OUT: f32 = 16.0;
        let ul = Pos2 {
            x: note.range.0.start.total_parts() as f32 / FIGURE_THIS_OUT,
            y: (note.key as f32) / notes_vert,
        };
        let br = Pos2 {
            x: note.range.0.end.total_parts() as f32 / FIGURE_THIS_OUT,
            y: (1.0 + note.key as f32) / notes_vert,
        };
        Rect::from_two_pos(ul, br)
    }
}
impl<'a> eframe::egui::Widget for PatternWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                MusicalTime::START.total_parts() as f32..=self.inner.duration.total_parts() as f32,
                128.0..=0.0,
            ),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        // Identify the local x and y values of the cursor.
        let mut key = 255;
        let mut position = MusicalTime::TIME_MAX;
        if let Some(screen_pos) = ui.ctx().pointer_interact_pos() {
            let local_pos = from_screen * screen_pos;
            key = local_pos.y as u8;
            position = MusicalTime::new_with_parts(local_pos.x as usize);
        }

        // Add or remove a note.
        if response.clicked() {
            let new_note = Note {
                key,
                range: TimeRange(position..position + PatternBuilder::DURATION),
            };
            if self.inner.notes.contains(&new_note) {
                self.inner.notes.retain(|n| &new_note != n);
            } else {
                self.inner.notes.push(new_note);
            }
        }

        let fill = ui.ctx().style().visuals.widgets.active.bg_fill;
        let shapes: Vec<Shape> = self
            .inner
            .notes
            .iter()
            .map(|note| {
                let rect = Rect::from_two_pos(
                    to_screen * pos2(note.range.0.start.total_parts() as f32, note.key as f32),
                    to_screen * pos2(note.range.0.end.total_parts() as f32, note.key as f32 + 1.0),
                );
                let hovered = note.key == key && note.range.0.contains(&position);
                let stroke = if hovered {
                    ui.ctx().style().visuals.widgets.active.fg_stroke
                } else {
                    ui.ctx().style().visuals.widgets.active.bg_stroke
                };

                Shape::Rect(RectShape::new(rect, Rounding::default(), fill, stroke))
            })
            .collect();

        painter.extend(shapes);

        response
    }
}
