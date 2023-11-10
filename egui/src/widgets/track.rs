// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::{
    egui::{Frame, Margin, Sense, TextFormat, Widget},
    emath::Align,
    epaint::{text::LayoutJob, vec2, Color32, FontId, Galley, Shape, Stroke, TextShape},
};
use ensnare_core::types::TrackTitle;
use std::{f32::consts::PI, sync::Arc};

/// Call this once for the TrackTitle, and then provide it on each frame to
/// the widget.
pub fn make_title_bar_galley(ui: &mut eframe::egui::Ui, title: &TrackTitle) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    job.append(
        title.0.as_str(),
        1.0,
        TextFormat {
            color: Color32::YELLOW,
            font_id: FontId::proportional(12.0),
            valign: Align::Center,
            ..Default::default()
        },
    );
    ui.ctx().fonts(|f| f.layout_job(job))
}

/// Wraps a [TitleBar] as a [Widget](eframe::egui::Widget). Don't have a
/// font_galley? Check out [make_title_bar_galley()].
pub fn title_bar(font_galley: Option<Arc<Galley>>) -> impl eframe::egui::Widget {
    move |ui: &mut eframe::egui::Ui| TitleBar::new(font_galley).ui(ui)
}

/// An egui widget that draws a [Track]'s sideways title bar.
#[derive(Debug)]
struct TitleBar {
    font_galley: Option<Arc<Galley>>,
}
impl Widget for TitleBar {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let available_size = vec2(16.0, ui.available_height());
        ui.set_min_size(available_size);

        // When drawing the timeline legend, we need to offset a titlebar-sized
        // space to align with track content. That's one reason why font_galley
        // is optional; we use None as a signal to draw just the empty space
        // that the titlebar would have occupied.
        let fill_color = if self.font_galley.is_some() {
            ui.style().visuals.faint_bg_color
        } else {
            ui.style().visuals.window_fill
        };

        Frame::default()
            .outer_margin(Margin::same(1.0))
            .inner_margin(Margin::same(0.0))
            .fill(fill_color)
            .show(ui, |ui| {
                ui.allocate_ui(available_size, |ui| {
                    let (response, painter) = ui.allocate_painter(available_size, Sense::click());
                    if let Some(font_galley) = &self.font_galley {
                        let t = Shape::Text(TextShape {
                            pos: response.rect.left_bottom(),
                            galley: Arc::clone(font_galley),
                            underline: Stroke::default(),
                            override_text_color: None,
                            angle: 2.0 * PI * 0.75,
                        });
                        painter.add(t);
                    }
                    response
                })
                .inner
            })
            .inner
    }
}
impl TitleBar {
    fn new(font_galley: Option<Arc<Galley>>) -> Self {
        Self { font_galley }
    }
}
