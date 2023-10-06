// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::{
    timeline::{cursor, grid},
    UiSize,
};
use crate::{
    drag_drop::{DragDropEvent, DragDropManager, DragDropSource},
    prelude::*,
    track::{Track, TrackAction, TrackTitle, TrackType},
    traits::prelude::*,
};
use eframe::{
    egui::{Button, Frame, Margin, Sense, TextFormat},
    emath::{Align, RectTransform},
    epaint::{
        text::LayoutJob, vec2, Color32, FontId, Galley, Rect, Shape, Stroke, TextShape, Vec2,
    },
};
use serde::{Deserialize, Serialize}; // See TrackWidget below
use std::{f32::consts::PI, sync::Arc};

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
/// font_galley? Check out [TitleBar::make_galley()].
pub fn title_bar(font_galley: Option<Arc<Galley>>) -> impl eframe::egui::Widget {
    move |ui: &mut eframe::egui::Ui| TitleBar::new(font_galley).ui(ui)
}

/// Wraps a [TrackWidget] as a [Widget](eframe::egui::Widget).
pub fn track<'a>(
    track: &'a mut Track,
    is_selected: bool,
    ui_state: TrackUiState,
    cursor: Option<MusicalTime>,
    action: &'a mut Option<TrackAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        TrackWidget::new(track, cursor, action)
            .is_selected(is_selected)
            .ui_state(ui_state)
            .ui(ui)
    }
}

/// Wraps a [SignalChainWidget] as a [Widget](eframe::egui::Widget). Mutates many things.
pub fn signal_chain<'a>(
    track: &'a mut Track,
    action: &'a mut Option<SignalChainAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| SignalChainWidget::new(track, action).ui(ui)
}

/// An egui widget that draws a [Track]'s sideways title bar.
#[derive(Debug)]
struct TitleBar {
    font_galley: Option<Arc<Galley>>,
}
impl Displays for TitleBar {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
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
                    if let Some(font_galley) = self.font_galley.take() {
                        let t = Shape::Text(TextShape {
                            pos: response.rect.left_bottom(),
                            galley: font_galley,
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

// TODO: the location and dependencies of this enum is weird. TrackWidget needs
// it, but Orchestrator serializes it. So it seems like it should be located
// here, and not with Track, but it is the only thing in widgets that needs
// serde. This is probably the thing that'll push me toward moving
// owner-specific widget code to the owner's file.
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub enum TrackUiState {
    #[default]
    Collapsed,
    Expanded,
}

/// An egui widget that draws a [Track].
#[derive(Debug)]
struct TrackWidget<'a> {
    track: &'a mut Track,
    is_selected: bool,
    ui_state: TrackUiState,
    cursor: Option<MusicalTime>,
    action: &'a mut Option<TrackAction>,
}
impl<'a> Displays for TrackWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // inner_margin() should be half of the Frame stroke width to leave room
        // for it. Thanks vikrinox on the egui Discord.
        Frame::default()
            .inner_margin(Margin::same(0.5))
            .stroke(Stroke {
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
                    ui.set_min_height(track_view_height(self.track.ty(), self.ui_state));

                    // The `Response` is based on the title bar, so
                    // clicking/dragging on the title bar affects the `Track` as a
                    // whole.
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
                        let mut from_screen = RectTransform::identity(Rect::NOTHING);
                        let can_accept = self.check_drag_source();

                        let response = DragDropManager::drop_target(ui, can_accept, |ui| {
                            let desired_size = vec2(ui.available_width(), 64.0);
                            let (_id, rect) = ui.allocate_space(desired_size);

                            // Only MIDI/audio tracks have content.
                            if !matches!(self.track.ty(), TrackType::Aux) {
                                // Reserve space for the device view.
                                ui.set_max_height(track_view_height(
                                    self.track.ty(),
                                    self.ui_state,
                                ));

                                // Determine the rectangle that all the composited layers will use.
                                let desired_size = vec2(ui.available_width(), 64.0);
                                let (_id, rect) = ui.allocate_space(desired_size);

                                let temp_range = MusicalTime::START..MusicalTime::DURATION_WHOLE;
                                let view_range = self.track.view_range().clone();

                                from_screen = RectTransform::from_to(
                                    rect,
                                    Rect::from_x_y_ranges(
                                        view_range.start.total_units() as f32
                                            ..=view_range.end.total_units() as f32,
                                        rect.top()..=rect.bottom(),
                                    ),
                                );

                                // The Grid is always disabled and drawn first.
                                let _ = ui
                                    .allocate_ui_at_rect(rect, |ui| {
                                        ui.add(grid(
                                            temp_range.clone(),
                                            self.track.view_range().clone(),
                                        ))
                                    })
                                    .inner;

                                // Draw the disabled timeline views.
                                let enabled_uid = self.track.foreground_timeline_entity.clone();
                                let entities: Vec<Uid> = self
                                    .track
                                    .timeline_entities
                                    .iter()
                                    .filter(|uid| enabled_uid != Some(**uid))
                                    .cloned()
                                    .collect();
                                entities.iter().for_each(|uid| {
                                    if let Some(e) = self.track.entity_mut(uid) {
                                        if let Some(e) = e.as_displays_in_timeline_mut() {
                                            ui.add_enabled_ui(false, |ui| {
                                                ui.allocate_ui_at_rect(rect, |ui| e.ui(ui)).inner
                                            })
                                            .inner;
                                        }
                                    }
                                });

                                // Draw the one enabled timeline view.
                                if let Some(uid) = enabled_uid {
                                    if let Some(e) = self.track.entity_mut(&uid) {
                                        if let Some(e) = e.as_displays_in_timeline_mut() {
                                            ui.add_enabled_ui(true, |ui| {
                                                ui.allocate_ui_at_rect(rect, |ui| e.ui(ui)).inner
                                            })
                                            .inner;
                                        }
                                    }

                                    // Finally, if it's present, draw the cursor.
                                    if let Some(position) = self.cursor {
                                        if view_range.contains(&position) {
                                            let _ = ui
                                                .allocate_ui_at_rect(rect, |ui| {
                                                    ui.add(cursor(position, view_range.clone()))
                                                })
                                                .inner;
                                        }
                                    }
                                }
                                ui.scope(|ui| {
                                    ui.set_max_height(device_view_height(self.ui_state));
                                    let mut action = None;
                                    ui.add(signal_chain(&mut self.track, &mut action));
                                    if let Some(action) = action {
                                        match action {
                                            SignalChainAction::NewDevice(key) => {
                                                eprintln!("yo"); // BUG! both the device view and the track view are handling the new device case, so we add twice on drag and drop
                                                *self.action = Some(TrackAction::NewDevice(
                                                    self.track.uid(),
                                                    key,
                                                ));
                                            }
                                            SignalChainAction::LinkControl(
                                                source_uid,
                                                target_uid,
                                                control_index,
                                            ) => {
                                                self.track.control_router.link_control(
                                                    source_uid,
                                                    target_uid,
                                                    control_index,
                                                );
                                            }
                                        }
                                    }
                                });

                                if ui.add(Button::new("next")).clicked() {
                                    self.track.select_next_foreground_timeline_entity();
                                }
                            }
                        })
                        .response;

                        if DragDropManager::is_dropped(ui, &response) {
                            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                let time_pos = from_screen * pointer_pos;
                                let time = MusicalTime::new_with_units(time_pos.x as usize);
                                if let Some(source) = DragDropManager::source() {
                                    let event = match source {
                                        DragDropSource::NewDevice(key) => Some(
                                            DragDropEvent::TrackAddDevice(self.track.uid(), key),
                                        ),
                                        DragDropSource::Pattern(pattern_uid) => {
                                            Some(DragDropEvent::TrackAddPattern(
                                                self.track.uid(),
                                                pattern_uid,
                                                time,
                                            ))
                                        }
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
        track: &'a mut Track,
        cursor: Option<MusicalTime>,
        action: &'a mut Option<TrackAction>,
    ) -> Self {
        Self {
            track,
            is_selected: false,
            ui_state: TrackUiState::Collapsed,
            cursor,
            action,
        }
    }

    fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    fn ui_state(mut self, ui_state: TrackUiState) -> Self {
        self.ui_state = ui_state;
        self
    }

    // Looks at what's being dragged, if anything, and updates any state needed
    // to handle it. Returns whether we are interested in this drag source.
    fn check_drag_source(&mut self) -> bool {
        if let Some(source) = DragDropManager::source() {
            if matches!(source, DragDropSource::Pattern(..)) {
                return true;
            }
        }
        false
    }
}

pub(crate) fn track_view_height(track_type: TrackType, ui_state: TrackUiState) -> f32 {
    if matches!(track_type, TrackType::Aux) {
        device_view_height(ui_state)
    } else {
        timeline_view_height(ui_state) + device_view_height(ui_state)
    }
}

pub(crate) const fn timeline_view_height(_ui_state: TrackUiState) -> f32 {
    64.0
}

pub(crate) const fn device_view_height(ui_state: TrackUiState) -> f32 {
    match ui_state {
        TrackUiState::Collapsed => 32.0,
        TrackUiState::Expanded => 96.0,
    }
}

#[derive(Debug)]
pub enum SignalChainAction {
    NewDevice(EntityKey),
    LinkControl(Uid, Uid, ControlIndex),
}

#[derive(Debug)]
struct SignalChainWidget<'a> {
    track: &'a mut Track,
    action: &'a mut Option<SignalChainAction>,
    ui_size: UiSize,
}
impl<'a> SignalChainWidget<'a> {
    pub fn new(track: &'a mut Track, action: &'a mut Option<SignalChainAction>) -> Self {
        Self {
            track,
            action,
            ui_size: Default::default(),
        }
    }

    fn can_accept(&self) -> bool {
        if let Some(source) = DragDropManager::source() {
            matches!(source, DragDropSource::NewDevice(_))
        } else {
            false
        }
    }

    fn check_drop(&mut self) {
        if let Some(source) = DragDropManager::source() {
            if let DragDropSource::NewDevice(key) = source {
                *self.action = Some(SignalChainAction::NewDevice(key))
            }
        }
    }
}
impl<'a> Displays for SignalChainWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        self.ui_size = UiSize::from_height(ui.available_height());
        let desired_size = ui.available_size();

        ui.allocate_ui(desired_size, |ui| {
            let stroke = ui.ctx().style().visuals.noninteractive().bg_stroke;
            eframe::egui::Frame::default()
                .stroke(stroke)
                .inner_margin(eframe::egui::Margin::same(stroke.width / 2.0))
                .show(ui, |ui| {
                    ui.set_min_size(desired_size);
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
                            .filter(|e| !self.track.timeline_entities.contains(e))
                            .for_each(|uid| {
                                if let Some(entity) = self.track.entity_store.get_mut(uid) {
                                    eframe::egui::CollapsingHeader::new(entity.name())
                                        .id_source(entity.uid())
                                        .show_unindented(ui, |ui| {
                                            if entity.as_controller().is_some() {
                                                DragDropManager::drag_source(
                                                    ui,
                                                    eframe::egui::Id::new(entity.name()),
                                                    DragDropSource::ControlSource(entity.uid()),
                                                    |ui| {
                                                        ui.label("control");
                                                    },
                                                )
                                            }
                                            entity.ui(ui);
                                        });
                                }
                            });
                        let response = DragDropManager::drop_target(ui, self.can_accept(), |ui| {
                            ui.label("[+]")
                        })
                        .response;
                        if DragDropManager::is_dropped(ui, &response) {
                            self.check_drop();
                        }
                    })
                    .inner
                });
        })
        .response
    }
}
