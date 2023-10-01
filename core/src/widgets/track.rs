// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::timeline::timeline;
use crate::{
    entities::factory::EntityStore,
    time::MusicalTime,
    track::{
        DeviceChain, DeviceChainAction, Track, TrackAction, TrackTitle, TrackType, TrackUiState,
        TrackUid,
    },
    traits::prelude::*,
    uid::Uid,
};
use eframe::{
    egui::{Frame, Margin, Sense, TextFormat},
    emath::Align,
    epaint::{text::LayoutJob, vec2, Color32, FontId, Galley, Shape, Stroke, TextShape, Vec2},
};
use std::{f32::consts::PI, sync::Arc};

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

/// Wraps a [DeviceChain] as a [Widget](eframe::egui::Widget). Mutates many things.
pub fn device_chain<'a>(
    track_uid: TrackUid,
    store: &'a mut EntityStore,
    controllers: &'a mut Vec<Uid>,
    instruments: &'a mut Vec<Uid>,
    effects: &'a mut Vec<Uid>,
    action: &'a mut Option<DeviceChainAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        DeviceChain::new(track_uid, store, controllers, instruments, effects, action).ui(ui)
    }
}

/// An egui widget that draws a [Track]'s sideways title bar.
#[derive(Debug)]
pub struct TitleBar {
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

    /// Call this once for the TrackTitle, and then provide it on each frame to
    /// the widget.
    pub fn make_galley(ui: &mut eframe::egui::Ui, title: &TrackTitle) -> Arc<Galley> {
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
}

/// An egui widget that draws a [Track].
#[derive(Debug)]
pub struct TrackWidget<'a> {
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
                    ui.set_min_height(Track::track_view_height(self.track.ty(), self.ui_state));

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
                        // Only MIDI/audio tracks have content.
                        if !matches!(self.track.ty(), TrackType::Aux) {
                            // Reserve space for the device view.
                            ui.set_max_height(Track::track_view_height(
                                self.track.ty(),
                                self.ui_state,
                            ));

                            let view_range = self.track.view_range().clone();
                            ui.add(timeline(
                                self.track.uid(),
                                &mut self.track.sequencer,
                                &mut self.track.control_atlas,
                                &mut self.track.control_router,
                                MusicalTime::START..MusicalTime::DURATION_WHOLE, // TODO - do we really need this?
                                view_range,
                                self.cursor,
                                super::timeline::FocusedComponent::Sequencer,
                            ));
                        }
                        ui.scope(|ui| {
                            ui.set_max_height(Track::device_view_height(self.ui_state));
                            let mut action = None;
                            ui.add(device_chain(
                                self.track.uid(),
                                &mut self.track.entity_store,
                                &mut self.track.controllers,
                                &mut self.track.instruments,
                                &mut self.track.effects,
                                &mut action,
                            ));
                            if let Some(action) = action {
                                let DeviceChainAction::NewDevice(key) = action;
                                *self.action = Some(TrackAction::NewDevice(self.track.uid(), key));
                            }
                        });
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
}
