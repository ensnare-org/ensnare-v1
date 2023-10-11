// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::types::Normal;
use eframe::epaint::{pos2, Color32, Rect, Rounding, Stroke};

/// Draws a column with a specified amount of yellow, heavier-than-air substance
/// in it.
pub fn level_indicator(level: Normal) -> impl eframe::egui::Widget + 'static {
    move |ui: &mut eframe::egui::Ui| {
        let desired_size = eframe::egui::vec2(2.0, 16.0);
        let (rect, response) = ui.allocate_exact_size(
            desired_size,
            eframe::egui::Sense::focusable_noninteractive(),
        );

        let percent = ui
            .ctx()
            .animate_value_with_time(response.id, level.into(), 0.1);

        if ui.is_rect_visible(rect) {
            ui.painter().rect(
                rect,
                Rounding::default(),
                Color32::BLACK,
                Stroke {
                    width: 1.0,
                    color: Color32::DARK_GRAY,
                },
            );
            let sound_rect = Rect::from_two_pos(
                rect.left_bottom(),
                pos2(rect.right(), rect.bottom() - rect.height() * percent),
            );
            ui.painter().rect(
                sound_rect,
                Rounding::default(),
                Color32::YELLOW,
                Stroke {
                    width: 1.0,
                    color: Color32::YELLOW,
                },
            );
        }

        response
    }
}

/// Draws an animated activity indicator that lights up immediately upon
/// activity and then fades if the activity stops.
pub fn activity_indicator(is_active: bool) -> impl eframe::egui::Widget + 'static {
    move |ui: &mut eframe::egui::Ui| {
        // This item is not clickable, but interact_size is convenient to use as
        // a size.
        let (rect, response) = ui.allocate_exact_size(
            eframe::egui::vec2(4.0, 4.0),
            eframe::egui::Sense::focusable_noninteractive(),
        );

        let how_on = if is_active {
            ui.ctx().animate_bool_with_time(response.id, true, 0.0);
            1.0f32
        } else {
            ui.ctx().animate_bool_with_time(response.id, false, 0.25)
        };

        if ui.is_rect_visible(rect) {
            ui.painter().rect(
                rect,
                Rounding::default(),
                Color32::YELLOW.linear_multiply(how_on),
                ui.visuals().window_stroke,
            );
        }

        response
    }
}
