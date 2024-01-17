// Copyright (c) 2024 Mike Tsao. All rights reserved.

use eframe::{
    egui::{Image, ImageButton, Widget},
    epaint::Vec2,
};

/// Wraps a [TimelineIconStrip] as a [Widget](eframe::egui::Widget).
pub fn timeline_icon_strip<'a>(
    action: &'a mut Option<TimelineIconStripAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| TimelineIconStrip::new(action).ui(ui)
}

#[derive(Debug)]
pub enum TimelineIconStripAction {
    NextTimelineView,
    ShowComposer,
}

/// An egui widget that displays an icon strip that goes above the timeline view.
#[derive(Debug)]
pub struct TimelineIconStrip<'a> {
    action: &'a mut Option<TimelineIconStripAction>,
}
impl<'a> TimelineIconStrip<'a> {
    fn new(action: &'a mut Option<TimelineIconStripAction>) -> Self {
        Self { action }
    }
}
impl<'a> eframe::egui::Widget for TimelineIconStrip<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.horizontal(|ui| {
            ui.spacing_mut().button_padding = Vec2::splat(0.0);
            ui.set_min_height(30.0); // TODO: I just want the image unscaled. How do I do that?
            let next_response = ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/view_timeline.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Next Timeline View");
            if next_response.clicked() {
                *self.action = Some(TimelineIconStripAction::NextTimelineView);
            }
            let piano_roll_response = ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/piano.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Show Piano Roll");
            if piano_roll_response.clicked() {
                *self.action = Some(TimelineIconStripAction::ShowComposer);
            }

            next_response | piano_roll_response
        })
        .inner
    }
}
