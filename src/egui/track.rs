// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::{
    automation::{SignalPathWidget, SignalPathWidgetAction},
    composition::ArrangementWidget,
    cursor::CursorWidget,
    signal_chain::{SignalChainWidget, SignalChainWidgetAction},
    util::fill_remaining_ui_space,
    GridWidget,
};
use crate::{
    orchestration::{TrackTitle, TrackViewMode},
    prelude::*,
    types::ColorScheme,
};
use eframe::{
    egui::{Frame, Image, ImageButton, Margin, Sense, TextFormat, Widget},
    emath::{Align, RectTransform},
    epaint::{
        text::LayoutJob, vec2, Color32, FontId, Galley, Rect, Shape, Stroke, TextShape, Vec2,
    },
};
use std::{f32::consts::PI, sync::Arc};
use strum_macros::Display;

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

#[derive(Debug, Display)]
pub enum TitleBarWidgetAction {
    /// Switch to the next timeline view.
    NextTimelineView,
    /// Add a new automation lane.
    NewAutomationLane,
}

/// An egui widget that draws a track's sideways title bar.
#[derive(Debug)]
pub struct TitleBarWidget<'a> {
    font_galley: Option<Arc<Galley>>,
    action: &'a mut Option<TitleBarWidgetAction>,
}
impl<'a> eframe::egui::Widget for TitleBarWidget<'a> {
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
                    ui.vertical(|ui| {
                        if self.font_galley.is_some() {
                            ui.vertical(|ui| {
                                if ui
                                    .add(ImageButton::new(
                                        Image::new(eframe::egui::include_image!(
                                            "../../res/images/md-symbols/menu.png"
                                        ))
                                        .fit_to_original_size(0.7),
                                    ).frame(false))
                                    .on_hover_text("Next timeline view")
                                    .clicked()
                                {
                                    *self.action = Some(TitleBarWidgetAction::NextTimelineView);
                                }
                                if ui
                                    .add(ImageButton::new(
                                        Image::new(eframe::egui::include_image!(
                                            "../../res/images/md-symbols/add.png"
                                        ))
                                        .fit_to_original_size(0.7),
                                    ).frame(false))
                                    .on_hover_text("New automation lane")
                                    .clicked()
                                {
                                    *self.action = Some(TitleBarWidgetAction::NewAutomationLane);
                                }
                            });
                        }
                        let (response, painter) =
                            ui.allocate_painter(ui.available_size(), Sense::click());
                        if let Some(font_galley) = &self.font_galley {
                            let t = Shape::Text(TextShape {
                                pos: response.rect.left_bottom(),
                                galley: Arc::clone(font_galley),
                                underline: Stroke::default(),
                                override_text_color: None,
                                angle: 2.0 * PI * 0.75,
                                fallback_color: Color32::YELLOW,
                                opacity_factor: 1.0,
                            });
                            painter.add(t);
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
impl<'a> TitleBarWidget<'a> {
    fn new(font_galley: Option<Arc<Galley>>, action: &'a mut Option<TitleBarWidgetAction>) -> Self {
        Self {
            font_galley,
            action,
        }
    }

    /// Don't have a font_galley? Check out [make_title_bar_galley()].
    pub fn widget(
        font_galley: Option<Arc<Galley>>,
        action: &'a mut Option<TitleBarWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| TitleBarWidget::new(font_galley, action).ui(ui)
    }
}

#[derive(Debug)]
pub enum TrackSource {
    PatternUid(PatternUid),
    ArrangementUid(ArrangementUid),
}

#[derive(Debug)]
pub struct TrackWidgetInfo {
    pub track_uid: TrackUid,
    pub title_font_galley: Option<Arc<Galley>>,
    pub color_scheme: ColorScheme,
}

#[derive(Debug, Display)]
pub enum TrackWidgetAction {
    /// Show the entity's detail view.
    SelectEntity(Uid, String),
    /// Remove the specified entity from the signal chain.
    RemoveEntity(Uid),
    /// Respond to a click on the track's title bar.
    Clicked,
    /// Add a new device to this track.
    NewDevice(EntityKey),
    /// Show the next timeline view.
    AdvanceTimelineView(TrackUid),
    CreateAutomationLane(TrackUid),
    ArrangePattern(PatternUid, MusicalTime),
    MoveArrangement(ArrangementUid, MusicalTime, bool),
    LinkPath(PathUid, Uid, ControlIndex),
    UnlinkPath(PathUid, Uid, ControlIndex),
}

/// An egui component that draws a track.
#[derive(Debug)]
pub struct TrackWidget<'a> {
    track_info: &'a TrackWidgetInfo,
    project: &'a mut Project,
    action: &'a mut Option<TrackWidgetAction>,
}
impl<'a> TrackWidget<'a> {
    const TIMELINE_HEIGHT: f32 = 64.0;
    const TRACK_HEIGHT: f32 = 96.0;

    fn new(
        track_info: &'a TrackWidgetInfo,
        project: &'a mut Project,
        action: &'a mut Option<TrackWidgetAction>,
    ) -> Self {
        Self {
            track_info,
            project,
            action,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        track_info: &'a TrackWidgetInfo,
        project: &'a mut Project,
        action: &'a mut Option<TrackWidgetAction>,
    ) -> impl Widget + 'a {
        move |ui: &mut eframe::egui::Ui| TrackWidget::new(track_info, project, action).ui(ui)
    }
}
impl<'a> Widget for TrackWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let track_uid = self.track_info.track_uid;
        let time_signature = self.project.time_signature().clone();
        let track_info = self.project.e.track_info.entry(track_uid).or_default();

        // inner_margin() should be half of the Frame stroke width to leave room
        // for it. Thanks vikrinox on the egui Discord.
        eframe::egui::Frame::default()
            .inner_margin(eframe::egui::Margin::same(0.5))
            .stroke(eframe::epaint::Stroke {
                width: 1.0,
                color: {
                    if self
                        .project
                        .view_state
                        .track_selection_set
                        .contains(&track_uid)
                    {
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
                        .track_info
                        .title_font_galley
                        .as_ref()
                        .map(|fg| Arc::clone(&fg));
                    let mut action = None;
                    let response = ui.add(TitleBarWidget::widget(font_galley, &mut action));
                    if let Some(action) = action {
                        match action {
                            TitleBarWidgetAction::NextTimelineView => {
                                *self.action =
                                    Some(TrackWidgetAction::AdvanceTimelineView(track_uid));
                            }
                            TitleBarWidgetAction::NewAutomationLane => {
                                *self.action =
                                    Some(TrackWidgetAction::CreateAutomationLane(track_uid));
                            }
                        }
                    }
                    if response.clicked() {
                        *self.action = Some(TrackWidgetAction::Clicked);
                    }

                    // Take up all the space we're given, even if we can't fill
                    // it with widget content.
                    ui.set_min_size(ui.available_size());

                    // The frames shouldn't have space between them.
                    ui.style_mut().spacing.item_spacing = Vec2::ZERO;

                    // Build the track content with the device view beneath it.
                    ui.vertical(|ui| {
                        let mut time = None;
                        let (_, payload) =
                            ui.dnd_drop_zone::<TrackSource>(Frame::default(), |ui| {
                                // Determine the rectangle that all the composited
                                // layers will use.
                                let desired_size =
                                    vec2(ui.available_width(), Self::TIMELINE_HEIGHT);
                                let (_id, rect) = ui.allocate_space(desired_size);

                                let temp_range =
                                    ViewRange(MusicalTime::START..MusicalTime::DURATION_WHOLE);

                                let from_screen = RectTransform::from_to(
                                    rect,
                                    Rect::from_x_y_ranges(
                                        self.project.view_state.view_range.0.start.total_units()
                                            as f32
                                            ..=self
                                                .project
                                                .view_state
                                                .view_range
                                                .0
                                                .end
                                                .total_units()
                                                as f32,
                                        rect.top()..=rect.bottom(),
                                    ),
                                );

                                // The Grid is always disabled and drawn first.
                                let _ = ui.add_enabled_ui(false, |ui| {
                                    ui.allocate_ui_at_rect(rect, |ui| {
                                        ui.add(GridWidget::widget(
                                            temp_range.clone(),
                                            self.project.view_state.view_range.clone(),
                                        ))
                                    })
                                    .inner
                                });

                                // Draw the widget corresponding to the current mode.
                                match self
                                    .project
                                    .view_state
                                    .track_view_mode
                                    .get(&track_uid)
                                    .copied()
                                    .unwrap_or_default()
                                {
                                    TrackViewMode::Composition => {
                                        ui.add_enabled_ui(true, |ui| {
                                            ui.allocate_ui_at_rect(rect, |ui| {
                                                ui.add(ArrangementWidget::widget(
                                                    self.track_info.track_uid,
                                                    &mut self.project.composer,
                                                    &self.project.view_state.view_range,
                                                    self.track_info.color_scheme,
                                                ));
                                            });
                                        });
                                    }
                                    TrackViewMode::Control(path_uid) => {
                                        ui.add_enabled_ui(true, |ui| {
                                            ui.allocate_ui_at_rect(rect, |ui| {
                                                if let Some(signal_path) =
                                                    self.project.automator.paths.get_mut(&path_uid)
                                                {
                                                    let mut signal_path_action = None;
                                                    let response =
                                                        ui.add(SignalPathWidget::widget(
                                                            signal_path,
                                                            &track_info.targets,
                                                            self.project
                                                                .view_state
                                                                .view_range
                                                                .clone(),
                                                            &mut signal_path_action,
                                                        ));
                                                    if let Some(action) = signal_path_action {
                                                        match action {
                                                            SignalPathWidgetAction::LinkTarget(
                                                                uid,
                                                                param,
                                                                should_link,
                                                            ) => {
                                                                if should_link {
                                                                    *self.action = Some(TrackWidgetAction::LinkPath(path_uid, uid, param));
                                                                } else {
                                                                    *self.action = Some(TrackWidgetAction::UnlinkPath(path_uid, uid, param));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    response.dnd_set_drag_payload(
                                                        ControlLinkSource::Path(path_uid),
                                                    )
                                                }
                                            });
                                        });
                                    }
                                }

                                // Next, if it's present, draw the cursor.
                                if let Some(position) = self.project.view_state.cursor {
                                    if self.project.view_state.view_range.0.contains(&position) {
                                        let _ = ui
                                            .allocate_ui_at_rect(rect, |ui| {
                                                ui.add(CursorWidget::widget(
                                                    position,
                                                    self.project.view_state.view_range.clone(),
                                                ))
                                            })
                                            .inner;
                                    }
                                }

                                time = if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                    let time_pos = from_screen * pointer_pos;
                                    let time = MusicalTime::new_with_units(time_pos.x as usize);
                                    if self.project.view_state.view_range.0.contains(&time) {
                                        let _ = ui
                                            .allocate_ui_at_rect(rect, |ui| {
                                                ui.add(CursorWidget::widget(
                                                    time,
                                                    self.project.view_state.view_range.clone(),
                                                ))
                                            })
                                            .inner;
                                    }
                                    Some(time)
                                } else {
                                    None
                                };
                            });
                        if let Some(track_source) = payload {
                            if let Some(time) = time {
                                let position = time.quantized_to_measure(&time_signature);
                                match *track_source {
                                    TrackSource::PatternUid(pattern_uid) => {
                                        *self.action = Some(TrackWidgetAction::ArrangePattern(
                                            pattern_uid,
                                            position,
                                        ));
                                    }
                                    TrackSource::ArrangementUid(arrangement_uid) => {
                                        let shift_pressed = ui.input(|input_state| {
                                            input_state.modifiers.shift_only()
                                        });
                                        *self.action = Some(TrackWidgetAction::MoveArrangement(
                                            arrangement_uid,
                                            position,
                                            shift_pressed,
                                        ));
                                    }
                                }
                            }
                        }

                        // Draw the signal chain view for every kind of track.
                        ui.scope(|ui| {
                            let mut action = None;
                            ui.add(SignalChainWidget::widget(
                                &track_info.signal_chain,
                                &mut action,
                            ));

                            if let Some(action) = action {
                                match action {
                                    SignalChainWidgetAction::Select(uid, name) => {
                                        *self.action =
                                            Some(TrackWidgetAction::SelectEntity(uid, name));
                                    }
                                    SignalChainWidgetAction::Remove(uid) => {
                                        *self.action = Some(TrackWidgetAction::RemoveEntity(uid));
                                    }
                                    SignalChainWidgetAction::NewDevice(key) => {
                                        *self.action = Some(TrackWidgetAction::NewDevice(key))
                                    }
                                }
                            }
                        });

                        fill_remaining_ui_space(ui);

                        response
                    })
                    .inner
                })
                .inner
            })
            .inner
    }
}
