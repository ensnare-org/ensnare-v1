// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{
    egui::{colors::ColorSchemeConverter, fill_remaining_ui_space},
    prelude::*,
    types::ColorScheme,
};
use derivative::Derivative;
use eframe::{
    egui::{PointerButton, Pos2, Sense, Vec2, Widget},
    emath::{Align2, RectTransform},
    epaint::{pos2, vec2, FontId, Rect, Rounding, Shape, Stroke},
};

pub type NoteLabelerFn = dyn Fn(MidiNote) -> String + 'static + Send + Sync;

pub struct NoteLabeler {
    function: Box<NoteLabelerFn>,
}
impl core::fmt::Debug for NoteLabeler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NoteLabeler").finish()
    }
}

impl NoteLabeler {
    pub fn new(function: impl Fn(MidiNote) -> String + 'static + Send + Sync) -> Self {
        Self {
            function: Box::new(function),
        }
    }

    fn format(&self, note: MidiNote) -> String {
        (self.function)(note)
    }
}

impl Default for NoteLabeler {
    fn default() -> Self {
        Self {
            function: Box::new(move |note| note.note_name_with_octave().to_string()),
        }
    }
}

pub type TimeLabelerFn = dyn Fn(&TimeSignature, MusicalTime) -> String + 'static + Send + Sync;

pub struct TimeLabeler {
    function: Box<TimeLabelerFn>,
}

impl TimeLabeler {
    pub fn new(
        function: impl Fn(&TimeSignature, MusicalTime) -> String + 'static + Send + Sync,
    ) -> Self {
        Self {
            function: Box::new(function),
        }
    }

    fn format(&self, time_signature: &TimeSignature, time: MusicalTime) -> String {
        (self.function)(time_signature, time)
    }
}

impl Default for TimeLabeler {
    fn default() -> Self {
        Self {
            function: Box::new(move |time_signature, time| time.to_visible_string(time_signature)),
        }
    }
}

#[derive(Derivative, Clone)]
#[derivative(Default)]
struct PrototypeComposerWidgetMemory {
    #[derivative(Default(value = "pos2(0.5, 0.5)"))]
    zoom_center: Pos2,
    #[derivative(Default(value = "vec2(1.0, 0.10)"))]
    zoom_factor: Vec2,
}

/// Renders a [Composer].
pub struct ComposerWidget<'a> {
    time_signature: TimeSignature,
    notes: &'a mut Vec<Note>,
    note_range: MidiNoteRange,
    time_labeler: TimeLabeler,
    note_labeler: NoteLabeler,
    color_scheme: ColorScheme,
}
impl<'a> eframe::egui::Widget for ComposerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        const ROUGH_LABEL_HEIGHT: f32 = 12.0;
        let (fg_color, bg_color) = ColorSchemeConverter::to_color32(self.color_scheme);

        // Draw top bar
        ui.label(format!("Time Signature: {}", self.time_signature));

        // Get our space
        let (id, full_rect) = ui.allocate_space(ui.available_size());

        // Leave a margin for the editor axis labels.
        const AXIS_SPACE: Vec2 = vec2(20.0, 10.0);
        let rect = full_rect.shrink2(AXIS_SPACE / 2.0).translate(AXIS_SPACE);

        // Create interaction response.
        let response = ui.interact(rect, id, Sense::click_and_drag());
        let mem: PrototypeComposerWidgetMemory = ui
            .ctx()
            .memory(|m| m.data.get_temp(response.id))
            .unwrap_or_default();

        // Create data-unit ranges (Sections by MidiNotes).
        let section_count = self.time_signature.bottom * 4;
        let x_section_data_start = 0;
        let x_section_data_end = section_count;
        let x_section_data_range = x_section_data_start..=x_section_data_end;
        let y_note_data_range = self.note_range.0.clone();

        // Create the view rect.
        //
        // Start with dimensions that are the same size as the data set.
        let width = (x_section_data_end - x_section_data_start) as f32;
        let height = <MidiNote as Into<f32>>::into(*y_note_data_range.end())
            - <MidiNote as Into<f32>>::into(*y_note_data_range.start());

        // Scale the dimensions according to the current zoom factor, and round
        // to the next-higher units. This is because we'd rather the high ends
        // of the axes clip a cell rather than omitting it, because omitting it
        // would mean the overall size of the editor grid would wiggle.
        let scaled_width = (width * mem.zoom_factor.x).ceil();
        let scaled_height: f32 = (height * mem.zoom_factor.y).ceil();

        // Build an origin-centered rect from the full dimensions.
        let data_range_rect = Rect::from_min_size(Pos2::default(), vec2(width, height));

        // Build a smaller rect centered around the current center point.
        let data_range_center = pos2(
            (mem.zoom_center.x * width).round(),
            (mem.zoom_center.y * height).round(),
        );
        let unclipped_view_rect =
            Rect::from_center_size(data_range_center, vec2(scaled_width, scaled_height));

        // If the smaller rect isn't completely inside the larger one, shove it
        // until it is.
        let view_rect = if data_range_rect.contains_rect(unclipped_view_rect) {
            unclipped_view_rect
        } else {
            let x_delta = if unclipped_view_rect.left() < data_range_rect.left() {
                data_range_rect.left() - unclipped_view_rect.left()
            } else if unclipped_view_rect.right() > data_range_rect.right() {
                data_range_rect.right() - unclipped_view_rect.right()
            } else {
                0.0
            };
            let y_delta = if unclipped_view_rect.top() < data_range_rect.top() {
                data_range_rect.top() - unclipped_view_rect.top()
            } else if unclipped_view_rect.bottom() > data_range_rect.bottom() {
                data_range_rect.bottom() - unclipped_view_rect.bottom()
            } else {
                0.0
            };
            unclipped_view_rect.translate(vec2(x_delta, y_delta))
        };

        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(
                view_rect.left()..=view_rect.right(),
                view_rect.bottom()..=view_rect.top(),
            ),
            rect,
        );
        let from_screen = to_screen.inverse();
        // The y axis is reversed because we have the lowest notes at the bottom
        // of the screen.
        let screen_to_normalized =
            RectTransform::from_to(rect, Rect::from_x_y_ranges(0.0..=1.0, 1.0..=0.0));
        let normalized_to_screen = screen_to_normalized.inverse();
        let time_to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(
                0.0..=MusicalTime::new_with_beats(self.time_signature.bottom).total_units() as f32,
                view_rect.bottom()..=view_rect.top(),
            ),
            rect,
        );

        let mut shapes = Vec::default();
        let mut label_shapes = Vec::default();

        let bg_stroke = Stroke::new(1.0, fg_color);
        let fg_stroke = Stroke::new(2.0, fg_color);
        shapes.push(Shape::rect_filled(rect, Rounding::default(), bg_color));

        let (hover_section, hover_note) = if let Some(hover_pos) = response.hover_pos() {
            let local_pos = from_screen * hover_pos;
            (
                Some(local_pos.x as usize),
                MidiNote::from_repr(local_pos.y as usize),
            )
        } else {
            (None, None)
        };

        let x_usize_range = view_rect.left() as usize..=view_rect.right() as usize;
        let y_usize_range = view_rect.top() as usize..=view_rect.bottom() as usize;
        let label_color = ui.style().noninteractive().text_color();
        let highlighted_label_color = ui.style().interact(&response).text_color();
        let mut prior_highlighted = false;
        for x in x_usize_range {
            let is_hovered_column = if let Some(hover_section) = hover_section {
                x == hover_section
            } else {
                false
            };
            let x_f32 = x as f32;
            let start = to_screen * pos2(x_f32, view_rect.top());
            let end = to_screen * pos2(x_f32, view_rect.bottom());
            shapes.push(Shape::line_segment(
                [start, end],
                if is_hovered_column || prior_highlighted {
                    fg_stroke
                } else {
                    bg_stroke
                },
            ));
            let time = MusicalTime::new_with_fractional_beats(x as f64 / 4.0);
            ui.fonts(|r| {
                label_shapes.push(Shape::text(
                    r,
                    end,
                    Align2::LEFT_BOTTOM,
                    self.time_labeler.format(&self.time_signature, time),
                    FontId::proportional(ROUGH_LABEL_HEIGHT),
                    if is_hovered_column {
                        highlighted_label_color
                    } else {
                        label_color
                    },
                ))
            });
            prior_highlighted = is_hovered_column;
        }

        // Determine whether the grid cell size is small enough that the Y-axis
        // labels would overlap.
        let test_label_height_pos_0 = to_screen * pos2(0.0, 0.0);
        let test_label_height_pos_1 = to_screen * pos2(0.0, 1.0);
        let test_label_height = test_label_height_pos_0.y - test_label_height_pos_1.y;
        let skip_some_labels = test_label_height < ROUGH_LABEL_HEIGHT;

        prior_highlighted = false;
        for y in y_usize_range {
            let is_hovered_row = if let Some(hover_note) = hover_note {
                hover_note as usize == y
            } else {
                false
            };
            let y_f32 = y as f32;
            let start = to_screen * pos2(view_rect.left(), y_f32);
            let end = to_screen * pos2(view_rect.right(), y_f32);
            shapes.push(Shape::line_segment(
                [start, end],
                if is_hovered_row || prior_highlighted {
                    fg_stroke
                } else {
                    bg_stroke
                },
            ));
            if y_f32 >= view_rect.bottom() {
                // We're being a little lazy and using a single larger clip rect
                // for both axes, so we have to manually make sure the final Y label isn't
                // extending into the X axis's area.
                continue;
            }

            let skip_label = skip_some_labels && !is_hovered_row && (y % 12) != 0;

            if !skip_label {
                // The 0.5 assumes we're incrementing by 1 each time
                let label = to_screen * pos2(view_rect.left(), y_f32 + 0.5);
                let note = MidiNote::from_repr(y).unwrap();
                ui.fonts(|r| {
                    label_shapes.push(Shape::text(
                        r,
                        label,
                        Align2::RIGHT_CENTER,
                        self.note_labeler.format(note),
                        FontId::proportional(12.0),
                        if is_hovered_row {
                            highlighted_label_color
                        } else {
                            label_color
                        },
                    ))
                });
            }
            prior_highlighted = is_hovered_row;
        }

        // Check for mouse input.
        if let Some(hover_pos) = response.hover_pos() {
            // Handle zoom wheel
            let zoom_factor = ui.input(|i| i.zoom_delta());
            if zoom_factor != 1.0 {
                let mut mem = mem.clone();
                mem.zoom_factor = vec2(
                    mem.zoom_factor.x,
                    (mem.zoom_factor.y / zoom_factor).min(1.0),
                );
                let new_center = screen_to_normalized * hover_pos;
                mem.zoom_center.y =
                    mem.zoom_center.y + (new_center.y - mem.zoom_center.y) * mem.zoom_factor.y;

                ui.ctx()
                    .memory_mut(|m| m.data.insert_temp(response.id, mem));
            }

            // Add or remove notes
            let hover_pos_data = from_screen * hover_pos;
            let note = MidiNote::from_repr(hover_pos_data.y.floor() as usize).unwrap();
            let section = hover_pos_data.x.floor() as usize;
            if response.clicked() {
                self.notes.push(Self::create_note(note, section));
            } else if response.clicked_by(PointerButton::Secondary) {
                let note_to_remove = Self::create_note(note, section);
                self.notes.retain(|n| *n != note_to_remove);
            }
        }
        if response.dragged_by(PointerButton::Primary) {
            let vert_only_drag_delta = vec2(0.0, response.drag_delta().y);
            let center_in_screen_units =
                normalized_to_screen * mem.zoom_center - vert_only_drag_delta;

            let mut mem = mem.clone();
            mem.zoom_center = screen_to_normalized * center_in_screen_units;
            ui.ctx()
                .memory_mut(|m| m.data.insert_temp(response.id, mem));
        }

        for note in self.notes {
            let lt =
                time_to_screen * pos2(note.extent.start().total_units() as f32, note.key as f32);
            let br = time_to_screen
                * pos2(
                    note.extent.end().total_units() as f32,
                    (note.key + 1) as f32,
                );
            shapes.push(Shape::rect_filled(
                Rect::from_two_pos(lt, br),
                Rounding::default(),
                fg_color,
            ));
        }

        ui.painter().extend(label_shapes);
        ui.painter_at(rect).extend(shapes);
        fill_remaining_ui_space(ui);

        response
    }
}

impl<'a> ComposerWidget<'a> {
    /// Creates a new widget.
    pub fn new(notes: &'a mut Vec<Note>) -> Self {
        Self {
            time_signature: Default::default(),
            notes,
            note_range: Default::default(),
            time_labeler: Default::default(),
            note_labeler: Default::default(),
            color_scheme: Default::default(),
        }
    }

    /// Provides an optional function that provides labels for the [MusicalTime]
    /// ticks on the X axis.
    pub fn time_labeler(mut self, time_labeler: TimeLabeler) -> Self {
        self.time_labeler = time_labeler;
        self
    }

    /// Provides an optional function that provides labels for the [MidiNote]
    /// ticks on the Y axis.
    pub fn note_labeler(mut self, note_labeler: NoteLabeler) -> Self {
        self.note_labeler = note_labeler;
        self
    }

    /// Specifies the range of notes that the Y axis should cover. Defaults to
    /// the full MIDI note range.
    pub fn note_range(mut self, note_range: MidiNoteRange) -> Self {
        self.note_range = note_range;
        self
    }

    /// Sets the color scheme for drawing notes.
    pub fn color_scheme(mut self, color_scheme: ColorScheme) -> Self {
        self.color_scheme = color_scheme;
        self
    }

    fn create_note(midi_note: MidiNote, section: usize) -> Note {
        Note {
            key: midi_note as u8,
            extent: TimeRange::new_with_start_and_duration(
                MusicalTime::new_with_fractional_beats(section as f64 / 4.0),
                MusicalTime::DURATION_QUARTER,
            ),
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(notes: &'a mut Vec<Note>) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| ComposerWidget::new(notes).ui(ui)
    }
}
