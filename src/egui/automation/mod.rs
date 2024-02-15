// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::{ControlTrip, *};
use eframe::{
    egui::{Sense, Widget},
    emath::RectTransform,
    epaint::{pos2, Color32, Rect, Stroke},
};
use strum_macros::Display;

/// A wrapper for identifiers of ControlLink sources. Both entities and paths
/// can generate Control events, so we express them here as variants.
#[derive(Debug, Display, Copy, Clone)]
pub enum ControlLinkSource {
    Entity(Uid),
    Path(PathUid),
}

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
            let start_pos = to_screen
                * pos2(
                    step.extent.0.start.total_units() as f32,
                    step.value_range.0.start.0 as f32,
                );
            let end_pos = to_screen
                * pos2(
                    step.extent.0.end.total_units() as f32,
                    step.value_range.0.end.0 as f32,
                );

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
            match step.ty {
                SignalStepType::Flat => {
                    painter.line_segment([start_pos, end_pos], stroke);
                }
                SignalStepType::Linear => {
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

/// An egui widget that draws a ControlTrip overlaid in the track view.
#[derive(Debug)]
pub struct ControlTripWidget<'a> {
    control_trip: &'a mut ControlTrip,
}
impl<'a> ControlTripWidget<'a> {
    fn new(control_trip: &'a mut ControlTrip) -> Self {
        Self { control_trip }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(control_trip: &'a mut ControlTrip) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| ControlTripWidget::new(control_trip).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for ControlTripWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("asdlfksd")
    }
}

#[derive(Debug)]
pub struct TripWidget<'a> {
    #[allow(dead_code)] // TODO see commented-out section in ui()
    uid: Uid,
    control_trip: &'a mut ControlTrip,
    control_links: Option<&'a [(Uid, ControlIndex)]>,
    view_range: ViewRange,
}
impl<'a> TripWidget<'a> {
    fn new(
        uid: Uid,
        control_trip: &'a mut ControlTrip,
        control_links: Option<&'a [(Uid, ControlIndex)]>,
        view_range: ViewRange,
    ) -> Self {
        Self {
            uid,
            control_trip,
            control_links,
            view_range,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        uid: Uid,
        trip: &'a mut ControlTrip,
        control_links: Option<&'a [(Uid, ControlIndex)]>,
        view_range: ViewRange,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| {
            TripWidget::new(uid, trip, control_links, view_range).ui(ui)
        }
    }
}
impl<'a> eframe::egui::Widget for TripWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(
                self.view_range.0.start.total_units() as f32
                    ..=self.view_range.0.end.total_units() as f32,
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
                        to_screen * pos2(self.view_range.0.end.total_units() as f32, 0.0);
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
            let label = if let Some(links) = self.control_links {
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
                eprintln!("got a link request");
            }
        }

        response
    }
}
