// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::{
    egui::{self, Sense},
    emath::RectTransform,
    epaint::{pos2, Color32, Rect, Stroke},
};
use ensnare_core::{
    control::ControlRouter,
    controllers::{ControlTrip, ControlTripPath},
    prelude::{ControlIndex, ControlValue},
    time::{MusicalTime, ViewRange},
    traits::{Displays, HasMetadata},
    uid::Uid,
};

/// Wraps a [ControlTrip] as a [Widget](eframe::egui::Widget).
pub fn trip<'a>(
    trip: &'a mut ControlTrip,
    control_router: &'a mut ControlRouter,
    view_range: ViewRange,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| Trip::new(trip, control_router, view_range).ui(ui)
}

#[derive(Debug)]
struct Trip<'a> {
    control_trip: &'a mut ControlTrip,
    control_router: &'a mut ControlRouter,
    view_range: ViewRange,
}
impl<'a> Trip<'a> {
    fn new(
        control_trip: &'a mut ControlTrip,
        control_router: &'a mut ControlRouter,
        view_range: ViewRange,
    ) -> Self {
        Self {
            control_trip,
            control_router,
            view_range,
        }
    }
}
impl<'a> Displays for Trip<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(
                self.view_range.start.total_units() as f32
                    ..=self.view_range.end.total_units() as f32,
                ControlValue::MAX.0 as f32..=ControlValue::MIN.0 as f32,
            ),
            response.rect,
        );

        // The first step always starts at the left of the view range.
        let mut pos = to_screen
            * pos2(
                MusicalTime::START.total_units() as f32,
                if let Some(step) = self.control_trip.steps.first() {
                    step.value.0 as f32
                } else {
                    0.0
                },
            );
        let stroke = if ui.is_enabled() {
            ui.ctx().style().visuals.widgets.active.fg_stroke
        } else {
            ui.ctx().style().visuals.widgets.inactive.fg_stroke
        };
        let steps_len = self.control_trip.steps.len();
        self.control_trip
            .steps
            .iter_mut()
            .enumerate()
            .for_each(|(index, step)| {
                // Get the next step position, adjusting if it's the last one.
                let second_pos = if index + 1 == steps_len {
                    let value = pos.y;
                    // Last step. Extend to end of view range.
                    let mut tmp_pos =
                        to_screen * pos2(self.view_range.end.total_units() as f32, 0.0);
                    tmp_pos.y = value;
                    tmp_pos
                } else {
                    // Not last step. Get the actual value.
                    to_screen * pos2(step.time.total_units() as f32, step.value.0 as f32)
                };

                // If we're hovering over this step, highlight it.
                let stroke = if response.hovered() {
                    if let Some(hover_pos) = ui.ctx().pointer_interact_pos() {
                        if hover_pos.x >= pos.x && hover_pos.x < second_pos.x {
                            if response.clicked() {
                                let from_screen = to_screen.inverse();
                                let hover_pos_local = from_screen * hover_pos;
                                step.value = ControlValue::from(hover_pos_local.y);
                            } else if response.secondary_clicked() {
                                step.path = step.path.next();
                            }

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
                match step.path {
                    ControlTripPath::None => {}
                    ControlTripPath::Flat => {
                        painter.line_segment([pos, pos2(pos.x, second_pos.y)], stroke);
                        painter.line_segment([pos2(pos.x, second_pos.y), second_pos], stroke);
                    }
                    ControlTripPath::Linear => {
                        painter.line_segment([pos, second_pos], stroke);
                    }
                    ControlTripPath::Logarithmic => todo!(),
                    ControlTripPath::Exponential => todo!(),
                }
                pos = second_pos;
            });

        if ui.is_enabled() {
            let label =
                if let Some(links) = self.control_router.control_links(self.control_trip.uid()) {
                    let link_texts = links.iter().fold(Vec::default(), |mut v, (uid, index)| {
                        // TODO: this can be a descriptive list of controlled things
                        v.push(format!("{uid}-{index:?} "));
                        v
                    });
                    link_texts.join("/")
                } else {
                    String::from("none")
                };
            if ui
                .allocate_ui_at_rect(response.rect, |ui| ui.button(&label))
                .inner
                .clicked()
            {
                // TODO: this is incomplete. It's a placeholder while I figure
                // out the best way to present this information (it might
                // actually be DnD rather than menu-driven).
                self.control_router.link_control(
                    self.control_trip.uid(),
                    Uid(234),
                    ControlIndex(456),
                );
            }
        }

        response
    }
}
