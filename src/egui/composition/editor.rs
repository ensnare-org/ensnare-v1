// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::{CarouselAction, CarouselWidget};
use crate::{
    egui::{colors::ColorSchemeConverter, fill_remaining_ui_space},
    prelude::*,
};
use core::ops::{Range, RangeInclusive};
use eframe::{
    egui::{Frame, PointerButton, Sense, Vec2, Widget},
    emath::{Align2, RectTransform},
    epaint::{pos2, vec2, Color32, FontId, Rect, RectShape, Rounding, Shape, Stroke},
};

/// Renders a [Composer].
#[derive(Debug)]
pub struct ComposerWidget<'a> {
    composer: &'a mut Composer,
}
impl<'a> eframe::egui::Widget for ComposerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            let r = {
                ui.horizontal(|ui| {
                    let response = {
                        let item_response = ui.button("New pattern");
                        if item_response.clicked() {
                            let _ = self.composer.add_pattern(
                                PatternBuilder::default()
                                    .time_signature(self.composer.time_signature())
                                    .color_scheme(self.composer.suggest_next_pattern_color_scheme())
                                    .build()
                                    .unwrap(),
                                None,
                            );
                        }
                        item_response
                    } | {
                        let item_response = ui.button("Add random");
                        if item_response.clicked() {
                            let contents = PatternBuilder::default()
                                .time_signature(self.composer.time_signature())
                                .random(&mut self.composer.e.rng)
                                .color_scheme(self.composer.suggest_next_pattern_color_scheme())
                                .build()
                                .unwrap();
                            let _ = self.composer.add_pattern(contents, None);
                        }
                        item_response
                    };
                    response
                })
                .inner
            } | {
                let mut carousel_action = None;
                let item_response = ui.add(CarouselWidget::widget(
                    &self.composer.ordered_pattern_uids,
                    &self.composer.patterns,
                    &mut self.composer.e.pattern_selection_set,
                    &mut carousel_action,
                ));
                if let Some(action) = carousel_action {
                    match action {
                        CarouselAction::DeletePattern(pattern_uid) => {
                            let _ = self.composer.remove_pattern(pattern_uid);
                        }
                    }
                }
                item_response
            } | {
                let item_response = ui.add(ComposerEditorWidget::widget(self.composer));
                if item_response.changed() {
                    self.composer.notify_pattern_change();
                }
                item_response
            };
            r
        })
        .inner
    }
}
impl<'a> ComposerWidget<'a> {
    fn new(composer: &'a mut Composer) -> Self {
        Self { composer }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(composer: &'a mut Composer) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| ComposerWidget::new(composer).ui(ui)
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
            if let Some(pattern) = self.composer.patterns.get_mut(&pattern_uid) {
                // inner_margin() should be half of the Frame stroke width to leave room
                // for it. Thanks vikrinox on the egui Discord.
                Frame::default()
                    .inner_margin(eframe::egui::Margin::same(2.0 / 2.0))
                    .show(ui, |ui| {
                        // Draw top bar
                        ui.label(format!("Time Signature: {}", pattern.time_signature()));
                        let available_size = ui.available_size();
                        let (_id, rect) =
                            ui.allocate_space(vec2(available_size.x, available_size.y - 10.0));

                        // Leave a margin for the editor axis labels.
                        let mut rect = rect;
                        *rect.left_mut() += 20.0;
                        *rect.top_mut() += 20.0;

                        if self.composer.e.editor_bounds_x == Range::<MusicalTime>::default() {
                            self.composer.e.editor_bounds_x =
                                MusicalTime::START..MusicalTime::new_with_beats(4);
                        }
                        if self.composer.e.editor_bounds_y == Range::<MidiNote>::default() {
                            self.composer.e.editor_bounds_y = MidiNote::C4..MidiNote::C5;
                        }

                        let min_window = self.composer.e.editor_bounds_y.start as u8;
                        let max_window = self.composer.e.editor_bounds_y.end as u8;

                        // Overlay the grid
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
                                ui.add(PatternEditorWidget::widget(
                                    pattern,
                                    min_window..=max_window,
                                ))
                            })
                            .inner;

                        // Handle pinch/zoom. TODO: this is being done in the
                        // top-level widget, which didn't know anything about
                        // screen coordinates. Either move this down to a widget
                        // that does, or reconsider the current division into
                        // grid/pattern sub-widgets.
                        if let Some(hover_pos) = response.hover_pos() {
                            let zoom_factor = ui.input(|i| i.zoom_delta_2d());
                            if zoom_factor != Vec2::splat(1.0) {
                                let first_note_f32 = min_window as f32;
                                let last_note_f32 = max_window as f32;
                                let note_range_f32 = last_note_f32..=first_note_f32;
                                let to_screen = RectTransform::from_to(
                                    eframe::epaint::Rect::from_x_y_ranges(
                                        0.0..=1.0,
                                        note_range_f32,
                                    ),
                                    rect,
                                );
                                let from_screen = to_screen.inverse();

                                // See
                                // https://github.com/emilk/egui/blob/master/crates/egui_plot/src/lib.rs,
                                // which I was physically near when I barfed out
                                // this code. Errors and hacks are my own.
                                let center = from_screen * hover_pos;
                                self.composer.e.editor_bounds_y.start = ((center.y
                                    + (self.composer.e.editor_bounds_y.start as u8 as f32
                                        - center.y)
                                        / (zoom_factor.y as f32))
                                    as u8)
                                    .into();
                                self.composer.e.editor_bounds_y.end = ((center.y
                                    + (self.composer.e.editor_bounds_y.end as u8 as f32 - center.y)
                                        / (zoom_factor.y as f32))
                                    as u8)
                                    .into();

                                // Did we zoom into a zero-length range?
                                if self.composer.e.editor_bounds_y.start
                                    == self.composer.e.editor_bounds_y.end
                                {
                                    if self.composer.e.editor_bounds_y.start == MidiNote::G9 {
                                        self.composer.e.editor_bounds_y.start = MidiNote::F9;
                                        self.composer.e.editor_bounds_y.end = MidiNote::G9;
                                    } else {
                                        self.composer.e.editor_bounds_y.end =
                                            self.composer.e.editor_bounds_y.start + 1;
                                    }
                                }
                            }

                            if response.dragged_by(PointerButton::Primary) {
                                let delta = -response.drag_delta();

                                // TODO: this kind of logic needs to be turned into reusable transform/clamp methods
                                if self.composer.e.editor_bounds_y.start + delta.y < MidiNote::CSub0
                                {
                                    self.composer.e.editor_bounds_y.end =
                                        self.composer.e.editor_bounds_y.end
                                            - self.composer.e.editor_bounds_y.start;
                                    self.composer.e.editor_bounds_y.start = MidiNote::CSub0;
                                } else if self.composer.e.editor_bounds_y.end + delta.y
                                    > MidiNote::G9
                                {
                                    self.composer.e.editor_bounds_y.start = MidiNote::G9
                                        - (self.composer.e.editor_bounds_y.end
                                            - self.composer.e.editor_bounds_y.start);
                                    self.composer.e.editor_bounds_y.end = MidiNote::G9;
                                } else {
                                    self.composer.e.editor_bounds_y.start += delta.y;
                                    self.composer.e.editor_bounds_y.end += delta.y;
                                }
                            }
                        }

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
    #[allow(dead_code)]
    fn time_signature(mut self, time_signature: TimeSignature) -> Self {
        self.time_signature = time_signature;
        self
    }

    #[allow(dead_code)]
    fn note_range(mut self, note_range: RangeInclusive<u8>) -> Self {
        self.note_range = note_range;
        self
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
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
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
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
        let from_screen = to_screen.inverse();

        // Identify the local x and y values of the cursor.
        let (cursor_key, cursor_section) = if let Some(screen_pos) = ui.ctx().pointer_interact_pos()
        {
            let local_pos = from_screen * screen_pos;
            (Some(local_pos.y as u8), Some(local_pos.x.floor()))
        } else {
            (None, None)
        };

        let visuals = ui.ctx().style().visuals.widgets.noninteractive;

        let mut background_shapes = Vec::default();
        let mut shapes = Vec::default();

        const COLUMN_ROW_HIGHLIGHT_COLOR: Color32 = Color32::from_rgb(32, 32, 32);

        // TODO: get story straight on inclusive/exclusive range
        let exclusive_note_range = *self.note_range.start()..*self.note_range.end();

        // Draw the horizontal note dividers.
        for key in self.note_range {
            let is_hovering = Some(key) == cursor_key && exclusive_note_range.contains(&key);
            let left = to_screen * pos2(0.0, key as f32);
            let right = to_screen * pos2(sections as f32, key as f32);
            let bottom_right = to_screen * pos2(sections as f32, (key + 1) as f32);
            let stroke = if key % 12 == 0 {
                visuals.fg_stroke
            } else {
                visuals.bg_stroke
            };
            shapes.push(Shape::LineSegment {
                points: [left, right],
                stroke,
            });
            if is_hovering {
                background_shapes.push(Shape::Rect(RectShape::filled(
                    Rect::from_two_pos(left, bottom_right),
                    visuals.rounding,
                    COLUMN_ROW_HIGHLIGHT_COLOR,
                )));
            }
            let (font_id, color, label) = if is_hovering {
                (
                    FontId::monospace(14.0),
                    Color32::YELLOW,
                    format!(
                        "{}",
                        MidiNote::from_repr(key as usize)
                            .unwrap()
                            .note_name_with_octave()
                    ),
                )
            } else {
                (
                    FontId::monospace(9.0),
                    visuals.text_color(),
                    format!("{}", MidiNote::from_repr(key as usize).unwrap().to_string()),
                )
            };
            // TODO: we should be creating and recycling at least one TextShape.
            ui.painter()
                .text(left, Align2::RIGHT_BOTTOM, label, font_id, color);
        }

        // Draw the vertical note dividers.
        for beat in 0..self.time_signature.top {
            let divisions_per_beat = self.time_signature.bottom;
            for division in 0..divisions_per_beat {
                let part = beat * divisions_per_beat + division;
                let is_hovering = if let Some(cursor_section) = cursor_section {
                    cursor_section as usize == part
                } else {
                    false
                };
                let x = part as f32;
                let stroke = if division == 0 {
                    visuals.fg_stroke
                } else {
                    visuals.bg_stroke
                };
                let line_start = to_screen * pos2(x, first_note_f32);
                let line_end = to_screen * pos2(x, last_note_f32);
                let bottom_right = to_screen * pos2(x + 1.0, first_note_f32);
                let line_middle = to_screen * pos2(x + 0.5, last_note_f32);
                if is_hovering {
                    background_shapes.push(Shape::Rect(RectShape::filled(
                        Rect::from_two_pos(line_end, bottom_right),
                        visuals.rounding,
                        COLUMN_ROW_HIGHLIGHT_COLOR,
                    )));
                }
                shapes.push(Shape::LineSegment {
                    points: [line_start, line_end],
                    stroke,
                });
                ui.painter().text(
                    line_middle,
                    Align2::CENTER_BOTTOM,
                    format!("{}.{}", beat + 1, division + 1),
                    FontId::monospace(12.0),
                    Color32::YELLOW,
                );
            }
        }
        ui.painter().extend(background_shapes);
        ui.painter().extend(shapes);

        response
    }
}

#[derive(Debug)]
pub struct PatternEditorWidget<'a> {
    pattern: &'a mut Pattern,
    note_range: RangeInclusive<u8>,
}
impl<'a> PatternEditorWidget<'a> {
    #[allow(dead_code)]
    fn pattern(mut self, pattern: &'a mut Pattern) -> Self {
        self.pattern = pattern;
        self
    }

    #[allow(dead_code)]
    fn note_range(mut self, note_range: RangeInclusive<u8>) -> Self {
        self.note_range = note_range;
        self
    }

    // This is separate from widget() so that we can instantitate the widget for
    // testing.
    fn new(pattern: &'a mut Pattern, note_range: RangeInclusive<u8>) -> Self {
        Self {
            pattern,
            note_range,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        pattern: &'a mut Pattern,
        note_range: RangeInclusive<u8>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| Self::new(pattern, note_range).ui(ui)
    }

    fn rect_for_note(to_screen: &RectTransform, note: &Note) -> Rect {
        // The `/ 4` is correct because a part is a 16th of a beat, and for this
        // rigid pattern widget, we're using only quarter-beat divisions.
        let ul = to_screen
            * pos2(
                (note.extent.0.start.total_parts() / 4) as f32,
                note.key as f32,
            );
        let br = to_screen
            * pos2(
                (note.extent.0.end.total_parts() / 4) as f32,
                (note.key + 1) as f32,
            );
        let note_rect = Rect::from_two_pos(ul, br);

        note_rect
    }

    fn division_duration(&self) -> MusicalTime {
        MusicalTime::DURATION_WHOLE / self.pattern.time_signature().bottom / 4
    }
}
impl<'a> eframe::egui::Widget for PatternEditorWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = ui.available_size();
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
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
        let from_screen = to_screen.inverse();

        // TODO: get story straight on inclusive/exclusive range
        let exclusive_note_range = *self.note_range.start()..*self.note_range.end();

        // Identify the local x and y values of the cursor.
        let (key, position) = if let Some(screen_pos) = ui.ctx().pointer_interact_pos() {
            let local_pos = from_screen * screen_pos;
            (
                // TODO: the min(127) is to catch an overflow in rect_for_note()
                // where this somehow ended up as 255. It might be from_screen
                // giving a degenerate result when the pointer ends up way out
                // of bounds. I don't know how this can happen.
                Some((local_pos.y as u8).min(127)),
                Some(MusicalTime::new_with_parts(local_pos.x as usize * 4)),
            )
        } else {
            (None, None)
        };

        // Select notes and add/remove.
        if response.clicked_by(PointerButton::Primary) {
            if let Some(key) = key {
                if let Some(position) = position {
                    let new_note = Note::new_with(key, position, PatternBuilder::DURATION);
                    self.pattern.add_note(new_note);
                    response.mark_changed();
                }
            }
        } else if response.clicked_by(PointerButton::Secondary) {
            if let Some(key) = key {
                if let Some(position) = position {
                    let new_note = Note::new_with(key, position, PatternBuilder::DURATION);
                    self.pattern.remove_note(&new_note);
                    response.mark_changed();
                }
            }
        }

        let mut shapes = Vec::default();

        let (_foreground_color, background_color) =
            ColorSchemeConverter::to_color32(self.pattern.color_scheme);
        let mut drew_hovered = false;
        for note in self.pattern.notes.iter() {
            if exclusive_note_range.contains(&note.key) {
                let note_rect = Self::rect_for_note(&to_screen, note);

                let hovered = Some(note.key) == key
                    && if let Some(position) = position {
                        note.extent.0.contains(&position)
                    } else {
                        false
                    };
                if hovered {
                    drew_hovered = true;
                }
                let stroke = if hovered {
                    ui.ctx().style().visuals.widgets.active.fg_stroke
                } else {
                    ui.ctx().style().visuals.widgets.active.bg_stroke
                };

                shapes.push(Shape::Rect(RectShape::new(
                    note_rect,
                    Rounding::default(),
                    background_color,
                    stroke,
                )));
            }
        }
        if !drew_hovered {
            if let Some(key) = key {
                if exclusive_note_range.contains(&key) {
                    if let Some(position) = position {
                        // The `* 4` is here because I haven't decided whether a
                        // pattern is always time sig's top x bottom # of divisions,
                        // or else 4x that (each note value divided by 4)
                        shapes.push(Shape::Rect(RectShape::new(
                            Self::rect_for_note(
                                &to_screen,
                                &Note::new_with(key, position, self.division_duration() * 4),
                            ),
                            Rounding::default(),
                            Color32::from_rgb(64, 64, 64),
                            Stroke {
                                width: 1.0,
                                color: Color32::DARK_GRAY,
                            },
                        )))
                    }
                }
            }
        }

        ui.painter().extend(shapes);

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn division_duration_works() {
        let mut pattern = PatternBuilder::default().build().unwrap();
        let note_range = 60..=71;
        let w = PatternEditorWidget::new(&mut pattern, note_range);

        assert_eq!(w.division_duration(), MusicalTime::DURATION_QUARTER / 4);
    }
}
