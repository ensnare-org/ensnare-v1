// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    control::ControlRouter,
    controllers::ControlAtlas,
    drag_drop::{DragDropEvent, DragDropManager, DragDropSource},
    even_smaller_sequencer::ESSequencer,
    prelude::*,
    track::TrackUid,
    traits::prelude::*,
};
use eframe::{
    egui::{self, vec2, Response, Ui},
    emath::{Align2, RectTransform},
    epaint::{pos2, FontId, Rect, RectShape, Shape},
};

/// Wraps a [Legend] as a [Widget](eframe::egui::Widget). Mutates the given view_range.
pub fn legend(view_range: &mut std::ops::Range<MusicalTime>) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| Legend::new(view_range).ui(ui)
}

/// Wraps a [Grid] as a [Widget](eframe::egui::Widget).
pub fn grid(
    range: std::ops::Range<MusicalTime>,
    view_range: std::ops::Range<MusicalTime>,
) -> impl eframe::egui::Widget {
    move |ui: &mut eframe::egui::Ui| Grid::default().range(range).view_range(view_range).ui(ui)
}

/// Wraps a [Cursor] as a [Widget](eframe::egui::Widget).
pub fn cursor(
    position: MusicalTime,
    view_range: std::ops::Range<MusicalTime>,
) -> impl eframe::egui::Widget {
    move |ui: &mut eframe::egui::Ui| {
        Cursor::default()
            .position(position)
            .view_range(view_range)
            .ui(ui)
    }
}

/// Wraps an [EmptySpace] as a [Widget](eframe::egui::Widget).
pub fn empty_space(
    range: std::ops::Range<MusicalTime>,
    view_range: std::ops::Range<MusicalTime>,
) -> impl eframe::egui::Widget {
    move |ui: &mut eframe::egui::Ui| EmptySpace::new().range(range).view_range(view_range).ui(ui)
}

/// An egui widget that draws a legend on the horizontal axis of the timeline
/// view.
#[derive(Debug)]
pub struct Legend<'a> {
    /// The GUI view's time range.
    view_range: &'a mut std::ops::Range<MusicalTime>,
}
impl<'a> Legend<'a> {
    fn new(view_range: &'a mut std::ops::Range<MusicalTime>) -> Self {
        Self { view_range }
    }

    fn steps(
        view_range: &std::ops::Range<MusicalTime>,
    ) -> std::iter::StepBy<std::ops::Range<usize>> {
        let beat_count = view_range.end.total_beats() - view_range.start.total_beats();
        let step = (beat_count as f32).log10().round() as usize;
        (view_range.start.total_beats()..view_range.end.total_beats()).step_by(step * 2)
    }
}
impl<'a> Displays for Legend<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::click());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                self.view_range.start.total_beats() as f32
                    ..=self.view_range.end.total_beats() as f32,
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
                self.view_range.start = self.view_range.start * 2;
                ui.close_menu();
            }
            if ui.button("Start x0.5").clicked() {
                self.view_range.start = self.view_range.start / 2;
                ui.close_menu();
            }
            if ui.button("Start +4").clicked() {
                self.view_range.start += MusicalTime::new_with_beats(4);
                ui.close_menu();
            }
        })
    }
}

/// An egui widget that draws a grid in the timeline view.
#[derive(Debug, Default)]
pub struct Grid {
    /// The timeline's full time range.
    range: std::ops::Range<MusicalTime>,

    /// The GUI view's time range.
    view_range: std::ops::Range<MusicalTime>,
}
impl Grid {
    fn range(mut self, range: std::ops::Range<MusicalTime>) -> Self {
        self.range = range.clone();
        self
    }
    fn view_range(mut self, view_range: std::ops::Range<MusicalTime>) -> Self {
        self.set_view_range(&view_range);
        self
    }
}
impl DisplaysInTimeline for Grid {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl Displays for Grid {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), 64.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::hover());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                self.view_range.start.total_beats() as f32
                    ..=self.view_range.end.total_beats() as f32,
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
    view_range: std::ops::Range<MusicalTime>,
}
impl Cursor {
    fn position(mut self, position: MusicalTime) -> Self {
        self.position = position;
        self
    }
    fn view_range(mut self, view_range: std::ops::Range<MusicalTime>) -> Self {
        self.set_view_range(&view_range);
        self
    }
}
impl DisplaysInTimeline for Cursor {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl Displays for Cursor {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), 64.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::hover());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                self.view_range.start.total_units() as f32
                    ..=self.view_range.end.total_units() as f32,
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

/// An egui widget that displays nothing in the timeline view. This is useful as
/// a DnD target.
#[derive(Debug, Default)]
pub struct EmptySpace {
    view_range: std::ops::Range<MusicalTime>,
    range: std::ops::Range<MusicalTime>,
}
#[allow(missing_docs)]
impl EmptySpace {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn view_range(mut self, view_range: std::ops::Range<MusicalTime>) -> Self {
        self.set_view_range(&view_range);
        self
    }
    pub fn range(mut self, range: std::ops::Range<MusicalTime>) -> Self {
        self.range = range;
        self
    }
}
impl DisplaysInTimeline for EmptySpace {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl Displays for EmptySpace {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.set_min_height(ui.available_height());

        let full_range_beats =
            (self.view_range.end.total_beats() - self.view_range.start.total_beats() - 1) as f32;
        let range_beats =
            (self.range.end.total_beats() - self.range.start.total_beats() - 1) as f32;
        let range_as_pct = range_beats / full_range_beats;
        let desired_size = vec2(ui.available_width() * range_as_pct, ui.available_height());
        let (rect, response) =
            ui.allocate_exact_size(desired_size, eframe::egui::Sense::click_and_drag());

        let visuals = if ui.is_enabled() {
            ui.ctx().style().visuals.widgets.active
        } else {
            ui.ctx().style().visuals.widgets.noninteractive
        };

        // skip interaction
        ui.painter()
            .rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);
        ui.painter()
            .line_segment([rect.right_top(), rect.left_bottom()], visuals.fg_stroke);
        ui.painter()
            .line_segment([rect.left_top(), rect.right_bottom()], visuals.fg_stroke);
        response
    }
}

/// Draws the content area of a Timeline, which is the view of a [Track].
#[derive(Debug)]
#[deprecated]
struct Timeline<'a> {
    track_uid: TrackUid,

    /// The full timespan of the project.
    range: std::ops::Range<MusicalTime>,

    /// The part of the timeline that is viewable.
    view_range: std::ops::Range<MusicalTime>,

    /// If present, then the moment that's currently playing.
    cursor: Option<MusicalTime>,

    control_atlas: &'a mut ControlAtlas,
    control_router: &'a mut ControlRouter,
    sequencer: &'a mut ESSequencer,
}
impl<'a> DisplaysInTimeline for Timeline<'a> {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl<'a> Displays for Timeline<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut from_screen = RectTransform::identity(Rect::NOTHING);
        let can_accept = self.check_drag_source();
        let response = DragDropManager::drop_target(ui, can_accept, |ui| {
            let desired_size = vec2(ui.available_width(), 64.0);
            let (_id, rect) = ui.allocate_space(desired_size);
            from_screen = RectTransform::from_to(
                rect,
                Rect::from_x_y_ranges(
                    self.view_range.start.total_units() as f32
                        ..=self.view_range.end.total_units() as f32,
                    rect.top()..=rect.bottom(),
                ),
            );
        })
        .response;
        if DragDropManager::is_dropped(ui, &response) {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let time_pos = from_screen * pointer_pos;
                let time = MusicalTime::new_with_units(time_pos.x as usize);
                if let Some(source) = DragDropManager::source() {
                    let event = match source {
                        DragDropSource::NewDevice(key) => {
                            Some(DragDropEvent::TrackAddDevice(self.track_uid, key))
                        }
                        DragDropSource::Pattern(pattern_uid) => Some(
                            DragDropEvent::TrackAddPattern(self.track_uid, pattern_uid, time),
                        ),
                        _ => None,
                    };
                    if let Some(event) = event {
                        DragDropManager::enqueue_event(event);
                    }
                }
            } else {
                eprintln!("Dropped on timeline at unknown position");
            }
        }
        response
    }
}
impl<'a> Timeline<'a> {
    pub fn new(
        track_uid: TrackUid,
        sequencer: &'a mut ESSequencer,
        control_atlas: &'a mut ControlAtlas,
        control_router: &'a mut ControlRouter,
    ) -> Self {
        Self {
            track_uid,
            range: Default::default(),
            view_range: Default::default(),
            cursor: None,
            sequencer,
            control_atlas,
            control_router,
        }
    }

    fn range(mut self, range: std::ops::Range<MusicalTime>) -> Self {
        self.range = range;
        self
    }

    fn cursor(mut self, cursor: Option<MusicalTime>) -> Self {
        self.cursor = cursor;
        self
    }

    fn view_range(mut self, view_range: std::ops::Range<MusicalTime>) -> Self {
        self.set_view_range(&view_range);
        self
    }

    // Looks at what's being dragged, if anything, and updates any state needed
    // to handle it. Returns whether we are interested in this drag source.
    fn check_drag_source(&mut self) -> bool {
        if let Some(source) = DragDropManager::source() {
            if matches!(source, DragDropSource::Pattern(_)) {
                // self.focused = FocusedComponent::Sequencer;
                return true;
            }
        }
        false
    }
}
