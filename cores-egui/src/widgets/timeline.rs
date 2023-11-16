// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::{
    egui::{vec2, Widget},
    emath::{Align2, RectTransform},
    epaint::{pos2, FontId, RectShape, Shape},
};
use ensnare_core::prelude::*;
use ensnare_egui_widgets::ViewRange;

/// Wraps a [TimelineIconStrip] as a [Widget](eframe::egui::Widget).
pub fn timeline_icon_strip<'a>(
    action: &'a mut Option<TimelineIconStripAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| TimelineIconStrip::new(action).ui(ui)
}

/// Wraps a [Legend] as a [Widget](eframe::egui::Widget).
pub fn legend(view_range: &mut ViewRange) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| Legend::new(view_range).ui(ui)
}

/// Wraps a [Grid] as a [Widget](eframe::egui::Widget).
pub fn grid<'a>(range: ViewRange, view_range: ViewRange) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| Grid::default().range(range).view_range(view_range).ui(ui)
}

/// Wraps a [Cursor] as a [Widget](eframe::egui::Widget).
pub fn cursor<'a>(position: MusicalTime, view_range: ViewRange) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        Cursor::default()
            .position(position)
            .view_range(view_range)
            .ui(ui)
    }
}

/// An egui widget that draws a legend on the horizontal axis of the timeline
/// view.
#[derive(Debug)]
pub struct Legend<'a> {
    /// The GUI view's time range.
    view_range: &'a mut ViewRange,
}
impl<'a> Legend<'a> {
    fn new(view_range: &'a mut ViewRange) -> Self {
        Self { view_range }
    }

    fn steps(view_range: &ViewRange) -> std::iter::StepBy<std::ops::Range<usize>> {
        let beat_count = view_range.0.end.total_beats() - view_range.0.start.total_beats();
        let step = (beat_count as f32).log10().round() as usize;
        (view_range.0.start.total_beats()..view_range.0.end.total_beats()).step_by(step * 2)
    }
}
impl<'a> eframe::egui::Widget for Legend<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::click());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                self.view_range.0.start.total_beats() as f32
                    ..=self.view_range.0.end.total_beats() as f32,
                rect.top()..=rect.bottom(),
            ),
            rect,
        );

        let font_id = FontId::proportional(12.0);
        for beat in Self::steps(self.view_range) {
            let beat_plus_one = beat + 1;
            let pos = to_screen * pos2(beat as f32, rect.top());
            ui.painter().text(
                pos,
                Align2::CENTER_TOP,
                format!("{beat_plus_one}"),
                font_id.clone(),
                ui.style().noninteractive().text_color(),
            );
        }
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            ui.style().noninteractive().bg_stroke,
        );

        response.context_menu(|ui| {
            if ui.button("Start x2").clicked() {
                self.view_range.0.start = self.view_range.0.start * 2;
                ui.close_menu();
            }
            if ui.button("Start x0.5").clicked() {
                self.view_range.0.start = self.view_range.0.start / 2;
                ui.close_menu();
            }
            if ui.button("Start +4").clicked() {
                self.view_range.0.start += MusicalTime::new_with_beats(4);
                ui.close_menu();
            }
        })
    }
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

/// An egui widget that draws a representation of the playback cursor.
#[derive(Debug, Default)]
pub struct Cursor {
    /// The cursor position.
    position: MusicalTime,

    /// The GUI view's time range.
    view_range: ViewRange,
}
impl Cursor {
    fn position(mut self, position: MusicalTime) -> Self {
        self.position = position;
        self
    }
    fn view_range(mut self, view_range: ViewRange) -> Self {
        self.view_range = view_range;
        self
    }
}
impl eframe::egui::Widget for Cursor {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), 64.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::hover());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                self.view_range.0.start.total_units() as f32
                    ..=self.view_range.0.end.total_units() as f32,
                0.0..=1.0,
            ),
            rect,
        );
        let visuals = ui.ctx().style().visuals.widgets.noninteractive;
        let start = to_screen * pos2(self.position.total_units() as f32, 0.0);
        let end = to_screen * pos2(self.position.total_units() as f32, 1.0);
        ui.painter().line_segment([start, end], visuals.fg_stroke);
        response
    }
}

#[derive(Debug)]
pub enum TimelineIconStripAction {
    NextTimelineView,
    ShowPianoRoll,
}

/// An egui widget that displays an icon strip that goes above the timeline view.
#[derive(Debug)]
pub struct TimelineIconStrip<'a> {
    action: &'a mut Option<TimelineIconStripAction>,
}
impl<'a> TimelineIconStrip<'a> {
    fn new(action: &'a mut Option<TimelineIconStripAction>) -> Self {
        Self { action }
    }
}
impl<'a> eframe::egui::Widget for TimelineIconStrip<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.horizontal(|ui| {
            let next_response = ui.button("Next timeline view");
            if next_response.clicked() {
                *self.action = Some(TimelineIconStripAction::NextTimelineView);
            }
            let piano_roll_response = ui.button("Piano Roll");
            if piano_roll_response.clicked() {
                *self.action = Some(TimelineIconStripAction::ShowPianoRoll);
            }

            next_response | piano_roll_response
        })
        .inner
    }
}
