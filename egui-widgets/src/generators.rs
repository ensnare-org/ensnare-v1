// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{prelude::*, waveform};
use eframe::{
    egui::{Frame, Sense, Widget},
    emath,
    epaint::{pos2, Color32, PathShape, Pos2, Rect, Shape, Stroke, Vec2},
};
use ensnare_core::{
    generators::{Envelope, Oscillator},
    prelude::*,
};

/// Wraps an [OscillatorWidget] as a [Widget](eframe::egui::Widget).
pub fn oscillator<'a>(oscillator: &'a mut Oscillator) -> impl Widget + 'a {
    move |ui: &mut eframe::egui::Ui| OscillatorWidget::new(oscillator).ui(ui)
}
/// An egui widget for [Oscillator].
#[derive(Debug)]
struct OscillatorWidget<'a> {
    oscillator: &'a mut Oscillator,
}
impl<'a> Widget for OscillatorWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(waveform(&mut self.oscillator.waveform))
    }
}
impl<'a> OscillatorWidget<'a> {
    fn new(oscillator: &'a mut Oscillator) -> Self {
        Self { oscillator }
    }
}

/// Wraps an [EnvelopeWidget] as a [Widget](eframe::egui::Widget).
pub fn envelope<'a>(envelope: &'a mut Envelope) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| EnvelopeWidget::new(envelope).ui(ui)
}

/// An egui widget that draws an [Envelope].
#[derive(Debug)]
struct EnvelopeWidget<'a> {
    envelope: &'a mut Envelope,
}
impl<'a> EnvelopeWidget<'a> {
    fn new(envelope: &'a mut Envelope) -> Self {
        Self { envelope }
    }
}
impl<'a> Widget for EnvelopeWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut attack = self.envelope.attack();
        let mut decay = self.envelope.decay();
        let mut sustain = self.envelope.sustain();
        let mut release = self.envelope.release();

        let canvas_response = ui.add(envelope_shaper(
            &mut attack,
            &mut decay,
            &mut sustain,
            &mut release,
        ));
        if canvas_response.changed() {
            self.envelope.set_attack(attack);
            self.envelope.set_decay(decay);
            self.envelope.set_sustain(sustain);
            self.envelope.set_release(release);
        }
        let attack_response = ui.add(drag_normal(&mut attack, "Attack: "));
        if attack_response.changed() {
            self.envelope.set_attack(attack);
        }
        ui.end_row();
        let decay_response = ui.add(drag_normal(&mut decay, "Decay: "));
        if decay_response.changed() {
            self.envelope.set_decay(decay);
        }
        ui.end_row();
        let sustain_response = ui.add(drag_normal(&mut sustain, "Sustain: "));
        if sustain_response.changed() {
            self.envelope.set_sustain(sustain);
        }
        ui.end_row();
        let release_response = ui.add(drag_normal(&mut release, "Release: "));
        if release_response.changed() {
            self.envelope.set_release(release);
        }
        ui.end_row();
        canvas_response | attack_response | decay_response | sustain_response | release_response
    }
}

// use eframe::{
//     egui::Sense,
//     emath,
//     epaint::{pos2, Color32, PathShape, Pos2, Rect, Shape, Stroke, Vec2},
// };
// pub fn ui_content(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
//     let (mut response, painter) =
//         ui.allocate_painter(Vec2::new(ui.available_width(), 64.0), Sense::hover());

//     let to_screen = emath::RectTransform::from_to(
//         Rect::from_min_size(Pos2::ZERO, response.rect.size()),
//         response.rect,
//     );

//     let control_point_radius = 8.0;

//     let x_max = response.rect.size().x;
//     let y_max = response.rect.size().y;

//     let attack_x_scaled = self.attack.0 as f32 * x_max / 4.0;
//     let decay_x_scaled = self.decay.0 as f32 * x_max / 4.0;
//     let sustain_y_scaled = (1.0 - self.sustain.0 as f32) * y_max;
//     let release_x_scaled = self.release.0 as f32 * x_max / 4.0;
//     let mut control_points = [
//         pos2(attack_x_scaled, 0.0),
//         pos2(attack_x_scaled + decay_x_scaled, sustain_y_scaled),
//         pos2(
//             attack_x_scaled
//                 + decay_x_scaled
//                 + (x_max - (attack_x_scaled + decay_x_scaled + release_x_scaled)) / 2.0,
//             sustain_y_scaled,
//         ),
//         pos2(x_max - release_x_scaled, sustain_y_scaled),
//     ];

//     let mut which_changed = usize::MAX;
//     let control_point_shapes: Vec<Shape> = control_points
//         .iter_mut()
//         .enumerate()
//         .map(|(i, point)| {
//             let size = Vec2::splat(2.0 * control_point_radius);

//             let point_in_screen = to_screen.transform_pos(*point);
//             let point_rect = Rect::from_center_size(point_in_screen, size);
//             let point_id = response.id.with(i);
//             let point_response = ui.interact(point_rect, point_id, Sense::drag());
//             if point_response.drag_delta() != Vec2::ZERO {
//                 which_changed = i;
//             }

//             // Restrict change to only the dimension we care about, so
//             // it looks less janky.
//             let mut drag_delta = point_response.drag_delta();
//             match which_changed {
//                 0 => drag_delta.y = 0.0,
//                 1 => drag_delta.y = 0.0,
//                 2 => drag_delta.x = 0.0,
//                 3 => drag_delta.y = 0.0,
//                 usize::MAX => {}
//                 _ => unreachable!(),
//             }

//             *point += drag_delta;
//             *point = to_screen.from().clamp(*point);

//             let point_in_screen = to_screen.transform_pos(*point);
//             let stroke = ui.style().interact(&point_response).fg_stroke;

//             Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
//         })
//         .collect();

//     if which_changed != usize::MAX {
//         match which_changed {
//             0 => {
//                 self.set_attack((control_points[0].x / (x_max / 4.0)).into());
//             }
//             1 => {
//                 self.set_decay(
//                     ((control_points[1].x - control_points[0].x) / (x_max / 4.0)).into(),
//                 );
//             }
//             2 => {
//                 self.set_sustain((1.0 - control_points[2].y / y_max).into());
//             }
//             3 => {
//                 self.set_release(((x_max - control_points[3].x) / (x_max / 4.0)).into());
//             }
//             _ => unreachable!(),
//         }
//     }

//     let control_points = [
//         pos2(0.0, y_max),
//         control_points[0],
//         control_points[1],
//         control_points[2],
//         control_points[3],
//         pos2(x_max, y_max),
//     ];
//     let points_in_screen: Vec<Pos2> = control_points.iter().map(|p| to_screen * *p).collect();

//     painter.add(PathShape::line(
//         points_in_screen,
//         Stroke {
//             width: 2.0,
//             color: Color32::YELLOW,
//         },
//     ));
//     painter.extend(control_point_shapes);

//     if which_changed != usize::MAX {
//         response.mark_changed();
//     }
//     response
// }

/// Wraps an [EnvelopeShaperWidget] as a [Widget](eframe::egui::Widget).
pub fn envelope_shaper<'a>(
    attack: &'a mut Normal,
    decay: &'a mut Normal,
    sustain: &'a mut Normal,
    release: &'a mut Normal,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        EnvelopeShaperWidget::new(attack, decay, sustain, release).ui(ui)
    }
}

/// An egui widget that allows visual editing of an [Envelope].
#[derive(Debug)]
struct EnvelopeShaperWidget<'a> {
    attack: &'a mut Normal,
    decay: &'a mut Normal,
    sustain: &'a mut Normal,
    release: &'a mut Normal,
}
impl<'a> EnvelopeShaperWidget<'a> {
    fn new(
        attack: &'a mut Normal,
        decay: &'a mut Normal,
        sustain: &'a mut Normal,
        release: &'a mut Normal,
    ) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
        }
    }
}
impl<'a> eframe::egui::Widget for EnvelopeShaperWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        Frame::canvas(ui.style())
            .show(ui, |ui| {
                let (mut response, painter) =
                    ui.allocate_painter(Vec2::new(128.0, 64.0), Sense::hover());

                let to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, response.rect.size()),
                    response.rect,
                );

                let control_point_radius = 8.0;

                let x_max = response.rect.size().x;
                let y_max = response.rect.size().y;

                let attack_x_scaled = self.attack.0 as f32 * x_max / 4.0;
                let decay_x_scaled = self.decay.0 as f32 * x_max / 4.0;
                let sustain_y_scaled = (1.0 - self.sustain.0 as f32) * y_max;
                let release_x_scaled = self.release.0 as f32 * x_max / 4.0;
                let mut control_points = [
                    pos2(attack_x_scaled, 0.0),
                    pos2(attack_x_scaled + decay_x_scaled, sustain_y_scaled),
                    pos2(
                        attack_x_scaled
                            + decay_x_scaled
                            + (x_max - (attack_x_scaled + decay_x_scaled + release_x_scaled)) / 2.0,
                        sustain_y_scaled,
                    ),
                    pos2(x_max - release_x_scaled, sustain_y_scaled),
                ];

                let mut which_changed = usize::MAX;
                let control_point_shapes: Vec<Shape> = control_points
                    .iter_mut()
                    .enumerate()
                    .map(|(i, point)| {
                        let size = Vec2::splat(2.0 * control_point_radius);

                        let point_in_screen = to_screen.transform_pos(*point);
                        let point_rect = Rect::from_center_size(point_in_screen, size);
                        let point_id = response.id.with(i);
                        let point_response = ui.interact(point_rect, point_id, Sense::drag());
                        if point_response.drag_delta() != Vec2::ZERO {
                            which_changed = i;
                        }

                        // Restrict change to only the dimension we care about, so
                        // it looks less janky.
                        let mut drag_delta = point_response.drag_delta();
                        match which_changed {
                            0 => drag_delta.y = 0.0,
                            1 => drag_delta.y = 0.0,
                            2 => drag_delta.x = 0.0,
                            3 => drag_delta.y = 0.0,
                            usize::MAX => {}
                            _ => unreachable!(),
                        }

                        *point += drag_delta;
                        *point = to_screen.from().clamp(*point);

                        let point_in_screen = to_screen.transform_pos(*point);
                        let stroke = ui.style().interact(&point_response).fg_stroke;

                        Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
                    })
                    .collect();

                if which_changed != usize::MAX {
                    match which_changed {
                        0 => {
                            *self.attack = Normal::from(control_points[0].x / (x_max / 4.0));
                        }
                        1 => {
                            *self.decay = Normal::from(
                                (control_points[1].x - control_points[0].x) / (x_max / 4.0),
                            );
                        }
                        2 => {
                            *self.sustain = Normal::from(1.0 - control_points[2].y / y_max);
                        }
                        3 => {
                            *self.release =
                                Normal::from((x_max - control_points[3].x) / (x_max / 4.0));
                        }
                        _ => unreachable!(),
                    }
                }

                let control_points = [
                    pos2(0.0, y_max),
                    control_points[0],
                    control_points[1],
                    control_points[2],
                    control_points[3],
                    pos2(x_max, y_max),
                ];
                let points_in_screen: Vec<Pos2> =
                    control_points.iter().map(|p| to_screen * *p).collect();

                painter.add(PathShape::line(
                    points_in_screen,
                    Stroke {
                        width: 2.0,
                        color: Color32::YELLOW,
                    },
                ));
                painter.extend(control_point_shapes);

                if which_changed != usize::MAX {
                    response.mark_changed();
                }
                response
            })
            .inner
    }
}
