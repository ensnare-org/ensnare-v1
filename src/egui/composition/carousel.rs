// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{egui::colors::ColorSchemeConverter, prelude::*, util::SelectionSet};
use eframe::{
    egui::{Id as EguiId, Image, ImageButton, Sense, Widget},
    emath::RectTransform,
    epaint::{pos2, Color32, Pos2, Rect, RectShape, Rounding, Shape, Stroke, Vec2},
};
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub enum CarouselAction {
    DeletePattern(PatternUid),
}

/// Displays a row of selectable icons, each with a drag source.
#[derive(Debug)]
pub struct CarouselWidget<'a> {
    pattern_uids: &'a [PatternUid],
    uids_to_patterns: &'a FxHashMap<PatternUid, Pattern>,
    selection_set: &'a mut SelectionSet<PatternUid>,
    action: &'a mut Option<CarouselAction>,
}
impl<'a> CarouselWidget<'a> {
    /// Creates a new [Carousel].
    pub fn new(
        pattern_uids: &'a [PatternUid],
        uids_to_patterns: &'a FxHashMap<PatternUid, Pattern>,
        selection_set: &'a mut SelectionSet<PatternUid>,
        action: &'a mut Option<CarouselAction>,
    ) -> Self {
        Self {
            pattern_uids,
            uids_to_patterns,
            selection_set,
            action,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        pattern_uids: &'a [PatternUid],
        uids_to_patterns: &'a FxHashMap<PatternUid, Pattern>,
        selection_set: &'a mut SelectionSet<PatternUid>,
        action: &'a mut Option<CarouselAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| {
            CarouselWidget::new(pattern_uids, uids_to_patterns, selection_set, action).ui(ui)
        }
    }
}
impl<'a> eframe::egui::Widget for CarouselWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.horizontal_top(|ui| {
            let icon_width = ui.available_width() / self.pattern_uids.len() as f32;
            ui.set_max_width(ui.available_width());
            ui.set_height(64.0);
            self.pattern_uids.iter().for_each(|pattern_uid| {
                ui.vertical(|ui| {
                    ui.set_max_width(icon_width);
                    if let Some(pattern) = self.uids_to_patterns.get(pattern_uid) {
                        let colors: (Color32, Color32) =
                            ColorSchemeConverter::to_color32(pattern.color_scheme);
                        let icon_response = ui.add(IconWidget::widget(
                            pattern.duration(),
                            pattern.notes(),
                            colors,
                            self.selection_set.contains(pattern_uid),
                        ));
                        icon_response.dnd_set_drag_payload(pattern_uid.clone());
                        let context_response = icon_response.context_menu(|ui| {
                            if ui.button("Delete pattern").clicked() {
                                *self.action = Some(CarouselAction::DeletePattern(*pattern_uid));
                            }
                        });
                        if icon_response.clicked() {
                            self.selection_set.click(pattern_uid, false);
                        };
                    }
                    let dd_id = EguiId::new("piano roll").with(pattern_uid);
                });
            });
        })
        .response
    }
}

/// Displays an iconic representation of a sequence of [Note]s. Intended to be a
/// drag-and-drop source.
#[derive(Debug, Default)]
pub struct IconWidget<'a> {
    duration: MusicalTime,
    notes: &'a [Note],
    colors: (Color32, Color32),
    is_selected: bool,
}
impl<'a> IconWidget<'a> {
    /// Creates a new [Icon].
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the duration of the pattern implied by the notes.
    pub fn duration(mut self, duration: MusicalTime) -> Self {
        self.duration = duration;
        self
    }
    /// Sets the sequence of [Note]s that determine the icon's appearance.
    pub fn notes(mut self, notes: &'a [Note]) -> Self {
        self.notes = notes;
        self
    }
    /// Sets the colors of the icon.
    pub fn colors(mut self, foreground: Color32, background: Color32) -> Self {
        self.colors = (foreground, background);
        self
    }
    /// Sets whether this widget is selected in the UI.
    pub fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        duration: MusicalTime,
        notes: &[Note],
        colors: (Color32, Color32),
        is_selected: bool,
    ) -> impl eframe::egui::Widget + '_ {
        move |ui: &mut eframe::egui::Ui| {
            let r = IconWidget::new()
                .duration(duration)
                .notes(notes)
                .colors(colors.0, colors.1)
                .is_selected(is_selected);
            r.ui(ui)
        }
    }
}
impl<'a> eframe::egui::Widget for IconWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = ui.spacing().interact_size.y * eframe::egui::vec2(3.0, 3.0);
        let (rect, response) =
            ui.allocate_exact_size(desired_size, eframe::egui::Sense::click_and_drag());

        let visuals = if ui.is_enabled() {
            ui.ctx().style().visuals.widgets.active
        } else {
            ui.ctx().style().visuals.widgets.inactive
        };

        if self.is_selected {
            ui.painter()
                .rect(rect, visuals.rounding, self.colors.1, visuals.fg_stroke);
        } else {
            ui.painter()
                .rect(rect, visuals.rounding, self.colors.1, visuals.bg_stroke);
        }
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                MusicalTime::START.total_parts() as f32..=self.duration.total_parts() as f32,
                128.0..=0.0,
            ),
            rect,
        );
        let note_stroke = Stroke {
            width: visuals.fg_stroke.width,
            color: self.colors.0,
        };
        for note in self.notes {
            let key = note.key as f32;
            let p1 =
                to_screen * eframe::epaint::pos2(note.extent.0.start.total_parts() as f32, key);
            let mut p2 =
                to_screen * eframe::epaint::pos2(note.extent.0.end.total_parts() as f32, key);

            // Even very short notes should be visible.
            if p1.x == p2.x {
                p2.x += 1.0;
            }
            ui.painter().line_segment([p1, p2], note_stroke);
        }
        response
    }
}
