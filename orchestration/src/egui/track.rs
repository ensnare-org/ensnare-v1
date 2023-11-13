// Copyright (c) 2023 Mike and Makeda Tsao. All rights reserved.

use crate::track::{SignalChainWidgetAction, TrackWidgetAction};
use eframe::{
    egui::{Frame, Margin, Sense, TextFormat, Widget},
    emath::{Align, RectTransform},
    epaint::{
        text::LayoutJob, vec2, Color32, FontId, Galley, Rect, Shape, Stroke, TextShape, Vec2,
    },
};
use ensnare_core::{
    time::{MusicalTime, ViewRange},
    types::TrackTitle,
    uid::TrackUid,
};
use ensnare_cores_egui::widgets::timeline::{cursor, grid};
use ensnare_drag_drop::{DragDropManager, DragSource, DropTarget};
use std::{f32::consts::PI, sync::Arc};

/// Wraps an [NewTrackWidget] as a [Widget](eframe::egui::Widget).
pub fn new_track_widget<'a>(
    track_uid: TrackUid,
    view_range: ViewRange,
    cursor: Option<MusicalTime>,
    title_font_galley: Option<Arc<Galley>>,
    action: &'a mut Option<TrackWidgetAction>,
) -> impl Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        NewTrackWidget::new(track_uid, view_range, cursor, title_font_galley, action).ui(ui)
    }
}

/// An egui component that draws anything implementing [Orchestrates].
#[derive(Debug)]
struct NewTrackWidget<'a> {
    track_uid: TrackUid,
    view_range: ViewRange,
    cursor: Option<MusicalTime>,
    title_font_galley: Option<Arc<Galley>>,

    action: &'a mut Option<TrackWidgetAction>,
}
impl<'a> NewTrackWidget<'a> {
    const TIMELINE_HEIGHT: f32 = 64.0;
    const TRACK_HEIGHT: f32 = 96.0;

    pub fn new(
        track_uid: TrackUid,
        view_range: ViewRange,
        cursor: Option<MusicalTime>,
        title_font_galley: Option<Arc<Galley>>,
        action: &'a mut Option<TrackWidgetAction>,
    ) -> Self {
        Self {
            track_uid,
            view_range,
            cursor,
            title_font_galley,
            action,
        }
    }

    // Looks at what's being dragged, if anything, and updates any state needed
    // to handle it. Returns whether we are interested in this drag source.
    fn check_drag_source_for_timeline() -> bool {
        if let Some(source) = DragDropManager::source() {
            if matches!(source, DragSource::Pattern(..)) {
                return true;
            }
        }
        false
    }
}
impl<'a> Widget for NewTrackWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // inner_margin() should be half of the Frame stroke width to leave room
        // for it. Thanks vikrinox on the egui Discord.
        eframe::egui::Frame::default()
            .inner_margin(eframe::egui::Margin::same(0.5))
            .stroke(eframe::epaint::Stroke {
                width: 1.0,
                color: {
                    if false {
                        Color32::YELLOW
                    } else {
                        Color32::DARK_GRAY
                    }
                },
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(Self::TRACK_HEIGHT);

                    // The `Response` is based on the title bar, so
                    // clicking/dragging on the title bar affects the `Track` as
                    // a whole.
                    let font_galley = self.title_font_galley.as_ref().map(|fg| Arc::clone(&fg));
                    let response = ui.add(title_bar(font_galley));

                    // Take up all the space we're given, even if we can't fill
                    // it with widget content.
                    ui.set_min_size(ui.available_size());

                    // The frames shouldn't have space between them.
                    ui.style_mut().spacing.item_spacing = Vec2::ZERO;

                    // Build the track content with the device view beneath it.
                    ui.vertical(|ui| {
                        let can_accept = Self::check_drag_source_for_timeline();
                        let _ = DragDropManager::drop_target(ui, can_accept, |ui| {
                            // Determine the rectangle that all the composited
                            // layers will use.
                            let desired_size = vec2(ui.available_width(), Self::TIMELINE_HEIGHT);
                            let (_id, rect) = ui.allocate_space(desired_size);

                            let temp_range = MusicalTime::START..MusicalTime::DURATION_WHOLE;

                            let from_screen = RectTransform::from_to(
                                rect,
                                Rect::from_x_y_ranges(
                                    self.view_range.start.total_units() as f32
                                        ..=self.view_range.end.total_units() as f32,
                                    rect.top()..=rect.bottom(),
                                ),
                            );

                            // The Grid is always disabled and drawn first.
                            let _ = ui
                                .allocate_ui_at_rect(rect, |ui| {
                                    ui.add(grid(temp_range.clone(), self.view_range.clone()))
                                })
                                .inner;

                            // The following code is incomplete. I want to check
                            // in anyway because the changes are getting too
                            // big.
                            //
                            // The intent is this (similar to code from a couple
                            // revs ago):
                            //
                            // 1. Have a way of representing which item is
                            //    frontmost. Maybe a smart enum.
                            // 2. Cycle through and render all but the frontmost
                            //    item, but disabled.
                            // 3. Render the frontmost, enabled.

                            // ui.add_enabled_ui(true, |ui| {
                            //     ui.allocate_ui_at_rect(rect, |ui| {
                            //         ui.add(live_pattern_sequencer_widget(
                            //             &mut self.track.sequencer,
                            //             &self.view_range,
                            //         ));
                            //     });
                            // });

                            // Draw control trips.
                            // let mut enabled = false;
                            // self.track.control_trips.iter_mut().for_each(|(uid, t)| {
                            //     ui.add_enabled_ui(enabled, |ui| {
                            //         ui.allocate_ui_at_rect(rect, |ui| {
                            //             ui.add(trip(
                            //                 *uid,
                            //                 t,
                            //                 self.track.control_router.control_links(*uid),
                            //                 self.view_range.clone(),
                            //             ));
                            //         });
                            //     });
                            //     enabled = false;
                            // });

                            // Finally, if it's present, draw the cursor.
                            if let Some(position) = self.cursor {
                                if self.view_range.contains(&position) {
                                    let _ = ui
                                        .allocate_ui_at_rect(rect, |ui| {
                                            ui.add(cursor(position, self.view_range.clone()))
                                        })
                                        .inner;
                                }
                            }
                            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                let time_pos = from_screen * pointer_pos;
                                let time = MusicalTime::new_with_units(time_pos.x as usize);
                                ((), DropTarget::TrackPosition(self.track_uid, time))
                            } else {
                                ((), DropTarget::Track(self.track_uid))
                            }
                        })
                        .response;

                        // Draw the signal chain view for every kind of track.
                        ui.scope(|_ui| {
                            let action = None;
                            // ui.add(signal_chain(self.track_uid, self.track, &mut action));
                            if let Some(action) = action {
                                match action {
                                    SignalChainWidgetAction::EntitySelected(uid, name) => {
                                        *self.action =
                                            Some(TrackWidgetAction::EntitySelected(uid, name));
                                    }
                                }
                            }
                        });

                        // This must be last. It makes sure we fill the
                        // remaining space.
                        ui.allocate_space(ui.available_size());

                        response
                    })
                    .inner
                })
                .inner
            })
            .inner;
        ui.label("asdf")
    }
}

/// Call this once for the TrackTitle, and then provide it on each frame to
/// the widget.
pub fn make_title_bar_galley(ui: &mut eframe::egui::Ui, title: &TrackTitle) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    job.append(
        title.0.as_str(),
        1.0,
        TextFormat {
            color: Color32::YELLOW,
            font_id: FontId::proportional(12.0),
            valign: Align::Center,
            ..Default::default()
        },
    );
    ui.ctx().fonts(|f| f.layout_job(job))
}

/// Wraps a [TitleBar] as a [Widget](eframe::egui::Widget). Don't have a
/// font_galley? Check out [make_title_bar_galley()].
pub fn title_bar(font_galley: Option<Arc<Galley>>) -> impl eframe::egui::Widget {
    move |ui: &mut eframe::egui::Ui| TitleBar::new(font_galley).ui(ui)
}

/// An egui widget that draws a [Track]'s sideways title bar.
#[derive(Debug)]
struct TitleBar {
    font_galley: Option<Arc<Galley>>,
}
impl eframe::egui::Widget for TitleBar {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let available_size = vec2(16.0, ui.available_height());
        ui.set_min_size(available_size);

        // When drawing the timeline legend, we need to offset a titlebar-sized
        // space to align with track content. That's one reason why font_galley
        // is optional; we use None as a signal to draw just the empty space
        // that the titlebar would have occupied.
        let fill_color = if self.font_galley.is_some() {
            ui.style().visuals.faint_bg_color
        } else {
            ui.style().visuals.window_fill
        };

        Frame::default()
            .outer_margin(Margin::same(1.0))
            .inner_margin(Margin::same(0.0))
            .fill(fill_color)
            .show(ui, |ui| {
                ui.allocate_ui(available_size, |ui| {
                    let (response, painter) = ui.allocate_painter(available_size, Sense::click());
                    if let Some(font_galley) = &self.font_galley {
                        let t = Shape::Text(TextShape {
                            pos: response.rect.left_bottom(),
                            galley: Arc::clone(font_galley),
                            underline: Stroke::default(),
                            override_text_color: None,
                            angle: 2.0 * PI * 0.75,
                        });
                        painter.add(t);
                    }
                    response
                })
                .inner
            })
            .inner
    }
}
impl TitleBar {
    fn new(font_galley: Option<Arc<Galley>>) -> Self {
        Self { font_galley }
    }
}
