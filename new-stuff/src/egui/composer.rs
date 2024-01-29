// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::composition::Composer;
use eframe::{egui::Widget, epaint::vec2};
use ensnare_core::{composition::PatternBuilder, traits::Configurable};
use ensnare_cores_egui::widgets::pattern::{
    CarouselAction, CarouselWidget, GridWidget, PatternWidget,
};

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
                            ui.allocate_ui_at_rect(rect, |ui| {
                                ui.add(GridWidget::widget(pattern.duration))
                            })
                            .inner
                        });
                        ui.allocate_ui_at_rect(rect, |ui| ui.add(PatternWidget::widget(pattern)))
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

    pub fn widget(inner: &'a mut Composer) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| ComposerWidget::new(inner).ui(ui)
    }
}
