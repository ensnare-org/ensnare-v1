// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::epaint::{pos2, Rect, Rounding};

/// Draws a column with a specified amount of yellow, heavier-than-air substance
/// in it.
pub fn level_indicator(level: f32) -> impl eframe::egui::Widget + 'static {
    move |ui: &mut eframe::egui::Ui| {
        let desired_size = eframe::egui::vec2(2.0, 16.0);
        let (rect, response) = ui.allocate_exact_size(
            desired_size,
            eframe::egui::Sense::focusable_noninteractive(),
        );

        let percent = ui.ctx().animate_value_with_time(response.id, level, 0.1);

        if ui.is_rect_visible(rect) {
            ui.painter().rect(
                rect,
                Rounding::default(),
                ui.visuals().faint_bg_color,
                ui.visuals().noninteractive().bg_stroke,
            );
            let sound_rect = Rect::from_two_pos(
                rect.left_bottom(),
                pos2(rect.right(), rect.bottom() - rect.height() * percent),
            );
            ui.painter().rect(
                sound_rect,
                Rounding::default(),
                ui.visuals().strong_text_color(),
                ui.visuals().noninteractive().bg_stroke,
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
                ui.visuals().strong_text_color().linear_multiply(how_on),
                ui.visuals().window_stroke,
            );
        }

        response
    }
}
