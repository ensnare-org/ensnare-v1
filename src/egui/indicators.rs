// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::epaint::Rounding;

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
