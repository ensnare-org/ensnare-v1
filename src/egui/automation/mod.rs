// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use eframe::{
    egui::{Sense, Widget},
    emath::RectTransform,
    epaint::{pos2, Color32, Rect, Stroke},
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
                ControlValue::MAX.0 as f32..=ControlValue::MIN.0 as f32,
            ),
            response.rect,
        );

        let stroke = if ui.is_enabled() {
            ui.ctx().style().visuals.widgets.active.fg_stroke
        } else {
            ui.ctx().style().visuals.widgets.inactive.fg_stroke
        };

        let mut prior_end_pos = None;
        self.signal_path.steps.iter_mut().for_each(|step| {
            let (start_pos, end_pos) = {
                match &step.ty {
                    SignalStepType::Flat(value) => {
                        let v = to_screen
                            * pos2(step.extent.0.start.total_units() as f32, value.0 as f32);
                        (v, v)
                    }
                    SignalStepType::Linear(range) => {
                        let start_pos = to_screen
                            * pos2(
                                step.extent.0.start.total_units() as f32,
                                range.0.start.0 as f32,
                            );
                        let end_pos = to_screen
                            * pos2(step.extent.0.end.total_units() as f32, range.0.end.0 as f32);
                        (start_pos, end_pos)
                    }
                    SignalStepType::Logarithmic => todo!(),
                    SignalStepType::Exponential => todo!(),
                }
            };

            // If we're hovering over this step, highlight it.
            let stroke = if response.hovered() {
                if let Some(hover_pos) = ui.ctx().pointer_interact_pos() {
                    if hover_pos.x >= start_pos.x && hover_pos.x < end_pos.x {
                        // if response.clicked() {
                        //     let from_screen = to_screen.inverse();
                        //     let hover_pos_local = from_screen * hover_pos;
                        //     step.value_range =
                        //         ControlRange::from(hover_pos_local.y..hover_pos_local.y);
                        // } else if response.secondary_clicked() {
                        //     // step.path = step.path.next();
                        //     // TODO huh?
                        // }

                        Stroke {
                            width: stroke.width * 2.0,
                            color: Color32::YELLOW,
                        }
                    } else {
                        stroke
                    }
                } else {
                    stroke
                }
            } else {
                stroke
            };

            // Draw according to the step type.
            match &step.ty {
                SignalStepType::Flat(..) | SignalStepType::Linear(..) => {
                    painter.line_segment([start_pos, end_pos], stroke);
                }
                SignalStepType::Logarithmic => todo!(),
                SignalStepType::Exponential => todo!(),
            }

            // The line should be continuous even if consecutive steps aren't lined up.
            if let Some(prior_end_pos) = prior_end_pos {
                painter.line_segment([prior_end_pos, start_pos], stroke);
            }
            prior_end_pos = Some(end_pos);
        });

        response
    }
}
