// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui logic for drawing ensnare entities.

use crate::pattern::pattern;
use eframe::{egui::Widget, epaint::vec2};
use widgets::pattern::{self, grid};

pub mod controllers;
pub mod effects;
pub mod instruments;
pub mod modulators;
pub mod piano_roll;
pub mod transport;
pub mod widgets;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        composer,
        controllers::{live_pattern_sequencer_widget, trip},
        transport::transport,
    };
}

pub struct Track {}
impl Track {
    /// The [TitleBar] widget needs a Galley so that it can display the title
    /// sideways. But widgets live for only a frame, so it can't cache anything.
    /// Caller to the rescue! We generate the Galley and save it.
    ///
    /// TODO: when we allow title editing, we should set the galley to None so
    /// it can be rebuilt on the next frame.
    pub fn update_font_galley(&mut self, _ui: &mut eframe::egui::Ui) {
        // if self.e.title_font_galley.is_none() && !self.title.0.is_empty() {
        //     self.e.title_font_galley = Some(make_title_bar_galley(ui, &self.title));
        // }
        todo!()
    }
}

/// Wraps a [FmSynthWidget] as a [Widget](eframe::egui::Widget).
pub fn composer<'a>(inner: &'a mut ensnare_cores::Composer) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| ComposerWidget::new(inner).ui(ui)
}

#[derive(Debug)]
pub struct ComposerWidget<'a> {
    inner: &'a mut ensnare_cores::Composer,
}
impl<'a> eframe::egui::Widget for ComposerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            let response = ui.add(pattern::carousel(
                &self.inner.ordered_pattern_uids,
                &self.inner.patterns,
                &mut self.inner.e.pattern_selection_set,
            )) | self.ui_pattern_edit(ui);
            response
        })
        .inner
    }
}
impl<'a> ComposerWidget<'a> {
    fn new(inner: &'a mut ensnare_cores::Composer) -> Self {
        Self { inner }
    }

    fn ui_pattern_edit(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if let Some(pattern_uid) = self.inner.e.pattern_selection_set.single_selection() {
            ui.set_min_height(192.0);
            if let Some(pat) = self.inner.patterns.get_mut(pattern_uid) {
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
