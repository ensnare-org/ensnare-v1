// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::legend::Legend;
use eframe::{
    egui::{vec2, Widget},
    emath::RectTransform,
    epaint::{pos2, RectShape, Shape},
};
use ensnare_core::time::ViewRange;

/// Wraps a [Grid] as a [Widget](eframe::egui::Widget).
pub fn grid<'a>(range: ViewRange, view_range: ViewRange) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| Grid::default().range(range).view_range(view_range).ui(ui)
}

/// An egui widget that draws a grid in the timeline view.

#[derive(Debug, Default)]
pub struct Grid {
    /// The timeline's full time range.
    range: ViewRange,

    /// The GUI view's time range.
    view_range: ViewRange,
}
impl Grid {
    fn range(mut self, range: ViewRange) -> Self {
        self.range = range;
        self
    }
    fn view_range(mut self, view_range: ViewRange) -> Self {
        self.view_range = view_range;
        self
    }
}
impl eframe::egui::Widget for Grid {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), 64.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::hover());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                self.view_range.0.start.total_beats() as f32
                    ..=self.view_range.0.end.total_beats() as f32,
                0.0..=1.0,
            ),
            rect,
        );
        let visuals = ui.ctx().style().visuals.widgets.noninteractive;

        let mut shapes = vec![Shape::Rect(RectShape::filled(
            rect,
            visuals.rounding,
            visuals.bg_fill,
        ))];

        for x in Legend::steps(&self.view_range) {
            shapes.push(Shape::LineSegment {
                points: [
                    to_screen * pos2(x as f32, 0.0),
                    to_screen * pos2(x as f32, 1.0),
                ],
                stroke: visuals.bg_stroke,
            });
        }
        ui.painter().extend(shapes);

        response
    }
}
