// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use eframe::{
    egui::{
        Sense,
        Shape::{self, LineSegment},
        Vec2, Widget,
    },
    emath::RectTransform,
    epaint::{pos2, Rect},
};

/// An egui widget that draws a SignalPath overlaid in the track view.
#[derive(Debug)]
pub struct SignalPathWidget<'a> {
    signal_path: &'a mut SignalPath,
    view_range: ViewRange,
}
impl<'a> SignalPathWidget<'a> {
    fn new(signal_path: &'a mut SignalPath, view_range: ViewRange) -> Self {
        Self {
            signal_path,
            view_range,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        signal_path: &'a mut SignalPath,
        view_range: ViewRange,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| SignalPathWidget::new(signal_path, view_range).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for SignalPathWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(
                self.view_range.0.start.total_units() as f32
                    ..=self.view_range.0.end.total_units() as f32,
                BipolarNormal::MAX as f32..=BipolarNormal::MIN as f32,
            ),
            response.rect,
        );
        let from_screen = to_screen.inverse();
        let stroke = ui.ctx().style().visuals.widgets.inactive.fg_stroke;

        let mut shapes = Vec::default();
        let mut point_shapes = Vec::default();
        let mut prior_when = None;
        let mut prior_value = None;
        let mut point_to_remove = None;
        let mut point_to_add = None;

        let _context_response = response.context_menu(|ui| {
            if !ui.ctx().is_context_menu_open() {
                if let Some(interact_pos) = response.interact_pointer_pos() {
                    ui.ctx()
                        .memory_mut(|m| m.data.insert_temp(response.id, interact_pos));
                }
            }
            let button_response = ui.button("Add");
            if button_response.clicked() {
                ui.close_menu();
                if let Some(interact_pos) = ui.ctx().memory(|m| m.data.get_temp(response.id)) {
                    point_to_add = Some(MusicalTime::new_with_units(
                        (from_screen * interact_pos).x as usize,
                    ));
                }
            }
        });

        let mut right_limits: Vec<f32> = self
            .signal_path
            .points
            .iter()
            .map(|p| p.when.total_units() as f32)
            .collect();
        // Effectively shift limits and add infinite right so that everyone has
        // a right limit.
        right_limits.remove(0);
        right_limits.push(MusicalTime::TIME_MAX.total_units() as f32);

        self.signal_path
            .points
            .iter_mut()
            .enumerate()
            .for_each(|(index, point)| {
                if prior_when.is_none() {
                    prior_when = Some(MusicalTime::START);
                }
                if prior_value.is_none() {
                    prior_value = Some(point.value);
                }
                let prior_when_unwrapped = prior_when.unwrap();
                let prior_value_unwrapped = prior_value.unwrap();
                let when_range = prior_when_unwrapped.total_units() as f32..right_limits[index];

                let (start_pos, end_pos) = {
                    let start_pos = to_screen
                        * pos2(
                            prior_when_unwrapped.total_units() as f32,
                            prior_value_unwrapped.0 as f32,
                        );
                    let end_pos =
                        to_screen * pos2(point.when.total_units() as f32, point.value.0 as f32);
                    (start_pos, end_pos)
                };

                shapes.push(LineSegment {
                    points: [start_pos, end_pos],
                    stroke,
                });

                const CONTROL_POINT_RADIUS: f32 = 6.0;
                const CONTROL_POINT_VISUAL_RADIUS: f32 = 4.0;
                let size = Vec2::splat(2.0 * CONTROL_POINT_RADIUS);
                let point_rect = Rect::from_center_size(end_pos, size);
                let point_id = response.id.with(index);
                let point_response = ui.interact(point_rect, point_id, Sense::click_and_drag());
                let _context_response = point_response.context_menu(|ui| {
                    if ui.button("Remove").clicked() {
                        ui.close_menu();
                        point_to_remove = Some(point.clone());
                    }
                });
                if point_response.drag_delta() != Vec2::ZERO {
                    let updated_point_pos = end_pos + point_response.drag_delta();
                    let back_to_local = (from_screen * updated_point_pos).clamp(
                        pos2(when_range.start, BipolarNormal::MIN as f32),
                        pos2(when_range.end, BipolarNormal::MAX as f32),
                    );
                    point.when = MusicalTime::new_with_units(back_to_local.x as usize);
                    point.value = BipolarNormal::from(back_to_local.y);
                }

                point_shapes.push(Shape::circle_filled(
                    end_pos,
                    CONTROL_POINT_VISUAL_RADIUS,
                    ui.style().interact(&point_response).fg_stroke.color,
                ));

                prior_when = Some(point.when);
                prior_value = Some(point.value);
            });
        if let Some(point) = point_to_remove {
            self.signal_path.remove_point(point);
        }
        if let Some(when) = point_to_add {
            self.signal_path.add_point(when);
        }
        if let Some(when) = prior_when {
            if let Some(value) = prior_value {
                if when != MusicalTime::TIME_MAX {
                    let start_pos = to_screen * pos2(when.total_units() as f32, value.0 as f32);
                    let end_pos = to_screen * pos2(MusicalTime::TIME_MAX.total_units() as f32, 1.0);
                    shapes.push(LineSegment {
                        points: [start_pos, end_pos],
                        stroke,
                    });
                }
            }
        }

        painter.extend(shapes);
        painter.extend(point_shapes);

        response
    }
}
