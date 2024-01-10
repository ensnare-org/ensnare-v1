// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    pattern::pattern,
    widgets::pattern::{self, grid},
};
use eframe::{egui::Widget, epaint::vec2};

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
            let response = ui.add(pattern::carousel(
                &self.entity.ordered_pattern_uids,
                &self.entity.uids_to_patterns,
                &mut self.entity.pattern_selection_set,
            )) | self.ui_pattern_edit(ui);
            response
        })
        .inner
    }
}
