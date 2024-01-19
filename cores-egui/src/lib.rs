// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui logic for drawing ensnare entities.

use eframe::{
    egui::Widget,
    epaint::{vec2, Color32},
};
use ensnare_core::{
    composition::{Composer, PatternBuilder},
    types::ColorScheme,
};
use widgets::pattern::{self, grid, pattern_widget};

pub mod controllers;
pub mod effects;
pub mod instruments;
pub mod modulators;
pub mod transport;
pub mod widgets;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{composer, controllers::trip, transport::transport};
}

/// Wraps a [ComposerWidget] as a [Widget](eframe::egui::Widget).
pub fn composer<'a>(inner: &'a mut Composer) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| ComposerWidget::new(inner).ui(ui)
}

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
                let _ = self
                    .inner
                    .add_pattern(PatternBuilder::default().build().unwrap(), None);
            }
            response = new_pattern_response;
            let mut carousel_action = None;
            let carousel_response = ui.add(pattern::carousel(
                &self.inner.ordered_pattern_uids,
                &self.inner.patterns,
                &mut self.inner.e.pattern_selection_set,
                &mut carousel_action,
            ));
            if let Some(action) = carousel_action {
                match action {
                    pattern::CarouselAction::DeletePattern(pattern_uid) => {
                        let _ = self.inner.remove_pattern(pattern_uid);
                    }
                }
            }
            response |= carousel_response;
            if let Some(pattern_uid) = self
                .inner
                .e
                .pattern_selection_set
                .single_selection()
                .cloned()
            {
                if let Some(pattern) = self.inner.pattern_mut(pattern_uid) {
                    ui.label(format!("Time Signature: {}", pattern.time_signature()));
                    let pattern_edit_response = {
                        ui.set_min_height(256.0);
                        let desired_size = vec2(ui.available_width(), 96.0);
                        let (_id, rect) = ui.allocate_space(desired_size);
                        ui.add_enabled_ui(false, |ui| {
                            ui.allocate_ui_at_rect(rect, |ui| ui.add(grid(pattern.duration)))
                                .inner
                        });
                        ui.allocate_ui_at_rect(rect, |ui| ui.add(pattern_widget(pattern)))
                            .inner
                    };
                    response |= pattern_edit_response;
                }
            }

            response
        })
        .inner
    }
}
impl<'a> ComposerWidget<'a> {
    fn new(inner: &'a mut Composer) -> Self {
        Self { inner }
    }
}

pub struct ColorSchemeConverter {}
impl ColorSchemeConverter {
    pub fn to_color32(color_scheme: ColorScheme) -> (Color32, Color32) {
        match color_scheme {
            // https://www.rapidtables.com/web/color/RGB_Color.html
            // https://www.sttmedia.com/colornames
            ColorScheme::Red => (Color32::BLACK, Color32::from_rgb(255, 153, 153)),
            ColorScheme::Vermilion => (Color32::BLACK, Color32::from_rgb(255, 178, 153)),
            ColorScheme::Orange => (Color32::BLACK, Color32::from_rgb(255, 204, 153)),
            ColorScheme::Amber => (Color32::BLACK, Color32::from_rgb(255, 229, 153)),
            ColorScheme::Yellow => (Color32::BLACK, Color32::from_rgb(254, 255, 153)),
            ColorScheme::Lime => (Color32::BLACK, Color32::from_rgb(229, 255, 153)),
            ColorScheme::Chartreuse => (Color32::BLACK, Color32::from_rgb(204, 255, 153)),
            ColorScheme::Ddahal => (Color32::BLACK, Color32::from_rgb(178, 255, 153)),
            ColorScheme::Green => (Color32::BLACK, Color32::from_rgb(153, 255, 153)),
            ColorScheme::Erin => (Color32::BLACK, Color32::from_rgb(153, 255, 178)),
            ColorScheme::Spring => (Color32::BLACK, Color32::from_rgb(153, 255, 204)),
            ColorScheme::Gashyanta => (Color32::BLACK, Color32::from_rgb(153, 255, 229)),
            ColorScheme::Cyan => (Color32::BLACK, Color32::from_rgb(153, 254, 255)),
            ColorScheme::Capri => (Color32::BLACK, Color32::from_rgb(153, 229, 255)),
            ColorScheme::Azure => (Color32::BLACK, Color32::from_rgb(153, 203, 255)),
            ColorScheme::Cerulean => (Color32::BLACK, Color32::from_rgb(153, 178, 255)),
            ColorScheme::Blue => (Color32::BLACK, Color32::from_rgb(153, 153, 255)),
            ColorScheme::Volta => (Color32::BLACK, Color32::from_rgb(178, 153, 255)),
            ColorScheme::Violet => (Color32::BLACK, Color32::from_rgb(203, 153, 255)),
            ColorScheme::Llew => (Color32::BLACK, Color32::from_rgb(229, 153, 255)),
            ColorScheme::Magenta => (Color32::BLACK, Color32::from_rgb(255, 153, 254)),
            ColorScheme::Cerise => (Color32::BLACK, Color32::from_rgb(255, 153, 229)),
            ColorScheme::Rose => (Color32::BLACK, Color32::from_rgb(255, 153, 204)),
            ColorScheme::Crimson => (Color32::BLACK, Color32::from_rgb(255, 153, 178)),
            ColorScheme::Gray1 => (Color32::WHITE, Color32::from_rgb(0, 0, 0)),
            ColorScheme::Gray2 => (Color32::WHITE, Color32::from_rgb(32, 32, 32)),
            ColorScheme::Gray3 => (Color32::WHITE, Color32::from_rgb(64, 64, 64)),
            ColorScheme::Gray4 => (Color32::WHITE, Color32::from_rgb(96, 96, 96)),
            ColorScheme::Gray5 => (Color32::WHITE, Color32::from_rgb(128, 128, 128)),
            ColorScheme::Gray6 => (Color32::BLACK, Color32::from_rgb(160, 160, 160)),
            ColorScheme::Gray7 => (Color32::BLACK, Color32::from_rgb(192, 192, 192)),
            ColorScheme::Gray8 => (Color32::BLACK, Color32::from_rgb(224, 224, 224)),
        }
    }
}
