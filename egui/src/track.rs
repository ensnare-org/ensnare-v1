// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    controllers::{live_pattern_sequencer_widget, trip},
    drag_drop::{DragDropManager, DragSource, DropTarget},
    widgets::{
        timeline::{cursor, grid},
        track::title_bar,
    },
};
use eframe::{
    egui::ImageButton,
    emath::RectTransform,
    epaint::{vec2, Color32, Rect, Vec2},
};
use ensnare_core::{
    prelude::*,
    traits::{Displays, IsAction},
};
use std::sync::Arc;
use strum_macros::Display;

/// Wraps a [TrackWidget] as a [Widget](eframe::egui::Widget).
pub fn track_widget<'a>(
    track_uid: TrackUid,
    track: &'a mut Track,
    is_selected: bool,
    cursor: Option<MusicalTime>,
    view_range: &'a ViewRange,
    action: &'a mut Option<TrackWidgetAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        TrackWidget::new(track_uid, track, cursor, view_range, action)
            .is_selected(is_selected)
            .ui(ui)
    }
}

#[derive(Debug, Display)]
pub enum TrackWidgetAction {
    EntitySelected(Uid, String),
}

/// An egui widget that draws a [Track].
#[derive(Debug)]
struct TrackWidget<'a> {
    track_uid: TrackUid,
    track: &'a mut Track,
    is_selected: bool,
    cursor: Option<MusicalTime>,
    view_range: ViewRange,
    action: &'a mut Option<TrackWidgetAction>,
}
impl<'a> TrackWidget<'a> {
    const TIMELINE_HEIGHT: f32 = 64.0;
    const TRACK_HEIGHT: f32 = 96.0;
}
impl<'a> Displays for TrackWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // inner_margin() should be half of the Frame stroke width to leave room
        // for it. Thanks vikrinox on the egui Discord.
        eframe::egui::Frame::default()
            .inner_margin(eframe::egui::Margin::same(0.5))
            .stroke(eframe::epaint::Stroke {
                width: 1.0,
                color: {
                    if self.is_selected {
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
                    let font_galley = self
                        .track
                        .e
                        .title_font_galley
                        .as_ref()
                        .map(|fg| Arc::clone(&fg));
                    let response = ui.add(title_bar(font_galley));

                    // Take up all the space we're given, even if we can't fill
                    // it with widget content.
                    ui.set_min_size(ui.available_size());

                    // The frames shouldn't have space between them.
                    ui.style_mut().spacing.item_spacing = Vec2::ZERO;

                    // Build the track content with the device view beneath it.
                    ui.vertical(|ui| {
                        let can_accept = self.check_drag_source_for_timeline();
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

                            ui.add_enabled_ui(true, |ui| {
                                ui.allocate_ui_at_rect(rect, |ui| {
                                    ui.add(live_pattern_sequencer_widget(
                                        &mut self.track.sequencer,
                                        &self.view_range,
                                    ));
                                });
                            });

                            // Draw control trips.
                            let mut enabled = false;
                            self.track.control_trips.values_mut().for_each(|t| {
                                ui.add_enabled_ui(enabled, |ui| {
                                    ui.allocate_ui_at_rect(rect, |ui| {
                                        ui.add(trip(
                                            t,
                                            &mut self.track.control_router,
                                            self.view_range.clone(),
                                        ));
                                    });
                                });
                                enabled = false;
                            });

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
                        ui.scope(|ui| {
                            let mut action = None;
                            ui.add(signal_chain(self.track_uid, &mut self.track, &mut action));
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
            .inner
    }
}
impl<'a> TrackWidget<'a> {
    fn new(
        track_uid: TrackUid,
        track: &'a mut Track,
        cursor: Option<MusicalTime>,
        view_range: &'a ViewRange,
        action: &'a mut Option<TrackWidgetAction>,
    ) -> Self {
        Self {
            track_uid,
            track,
            is_selected: false,
            cursor,
            view_range: view_range.clone(),
            action,
        }
    }

    fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    // Looks at what's being dragged, if anything, and updates any state needed
    // to handle it. Returns whether we are interested in this drag source.
    fn check_drag_source_for_timeline(&mut self) -> bool {
        if let Some(source) = DragDropManager::source() {
            if matches!(source, DragSource::Pattern(..)) {
                return true;
            }
        }
        false
    }
}

/// Wraps a [SignalChainWidget] as a [Widget](eframe::egui::Widget).
pub fn signal_chain<'a>(
    track_uid: TrackUid,
    track: &'a mut Track,
    action: &'a mut Option<SignalChainWidgetAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| SignalChainWidget::new(track_uid, track, action).ui(ui)
}

#[derive(Debug, Display)]
pub enum SignalChainWidgetAction {
    EntitySelected(Uid, String),
}
impl IsAction for SignalChainWidgetAction {}

struct SignalChainWidget<'a> {
    track_uid: TrackUid,
    track: &'a mut Track,
    action: &'a mut Option<SignalChainWidgetAction>,
}
impl<'a> SignalChainWidget<'a> {
    pub fn new(
        track_uid: TrackUid,
        track: &'a mut Track,
        action: &'a mut Option<SignalChainWidgetAction>,
    ) -> Self {
        Self {
            track_uid,
            track,
            action,
        }
    }

    fn can_accept(&self) -> bool {
        if let Some(source) = DragDropManager::source() {
            matches!(source, DragSource::NewDevice(_))
        } else {
            false
        }
    }
}
impl<'a> Displays for SignalChainWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let stroke = ui.ctx().style().visuals.noninteractive().bg_stroke;
        let response = eframe::egui::Frame::default()
            .stroke(stroke)
            .inner_margin(eframe::egui::Margin::same(stroke.width / 2.0))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    self.track
                        .controllers
                        .iter()
                        .chain(
                            self.track
                                .instruments
                                .iter()
                                .chain(self.track.effects.iter()),
                        )
                        .for_each(|uid| {
                            if let Some(entity) = self.track.entity_store.get_mut(uid) {
                                if ui
                                    .add(signal_chain_item(
                                        *uid,
                                        entity.name(),
                                        entity.as_controller().is_some(),
                                    ))
                                    .clicked()
                                {
                                    *self.action = Some(SignalChainWidgetAction::EntitySelected(
                                        *uid,
                                        entity.name().to_string(),
                                    ));
                                }
                            }
                        });
                    let _ = DragDropManager::drop_target(ui, self.can_accept(), |ui| {
                        (
                            ui.add_enabled(false, eframe::egui::Button::new("Drag Items Here")),
                            DropTarget::Track(self.track_uid),
                        )
                    })
                    .response;
                    ui.allocate_space(ui.available_size());
                })
                .inner
            })
            .response;
        response
    }
}

/// Wraps a [SignalChainItem] as a [Widget](eframe::egui::Widget).
pub fn signal_chain_item<'a>(
    uid: Uid,
    name: &'static str,
    is_control_source: bool,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| SignalChainItem::new(uid, name, is_control_source).ui(ui)
}

struct SignalChainItem {
    uid: Uid,
    name: &'static str,
    is_control_source: bool,
}
impl SignalChainItem {
    fn new(uid: Uid, name: &'static str, is_control_source: bool) -> Self {
        Self {
            uid,
            name,
            is_control_source,
        }
    }
}
impl Displays for SignalChainItem {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if self.is_control_source {
            ui.horizontal(|ui| {
                let icon =
                    eframe::egui::include_image!("../../res/images/md-symbols/drag_indicator.png");
                let response = ui.button(self.name);
                DragDropManager::drag_source(
                    ui,
                    eframe::egui::Id::new(self.name),
                    DragSource::ControlSource(self.uid),
                    |ui| ui.add(ImageButton::new(icon).tint(ui.ctx().style().visuals.text_color())),
                );
                response
            })
            .inner
        } else {
            ui.button(self.name)
        }
    }
}
