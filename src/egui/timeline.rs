// Copyright (c) 2024 Mike Tsao. All rights reserved.

use eframe::{
    egui::{Image, ImageButton, Widget},
    epaint::Vec2,
};

#[derive(Debug)]
pub enum TimelineIconStripAction {
    NextTimelineView,
    ShowComposer,
}

/// An egui widget that displays an icon strip that goes above the timeline view.
#[derive(Debug)]
pub struct TimelineIconStripWidget<'a> {
    action: &'a mut Option<TimelineIconStripAction>,
}
impl<'a> TimelineIconStripWidget<'a> {
    fn new(action: &'a mut Option<TimelineIconStripAction>) -> Self {
        Self { action }
    }

    pub fn widget(
        action: &'a mut Option<TimelineIconStripAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| TimelineIconStripWidget::new(action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for TimelineIconStripWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.horizontal(|ui| {
            ui.spacing_mut().button_padding = Vec2::splat(0.0);
            ui.set_min_height(30.0); // TODO: I just want the image unscaled. How do I do that?
            let next_response = ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../res/images/md-symbols/view_timeline.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Next Timeline View");
            if next_response.clicked() {
                *self.action = Some(TimelineIconStripAction::NextTimelineView);
            }
            let composer_response = ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../res/images/md-symbols/piano.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Show Piano Roll");
            if composer_response.clicked() {
                *self.action = Some(TimelineIconStripAction::ShowComposer);
            }

            next_response | composer_response
        })
        .inner
    }
}
