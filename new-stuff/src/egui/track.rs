// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::cursor::cursor;
use super::signal_chain::{signal_chain_widget, SignalChainItem, SignalChainWidgetAction};
use crate::egui::grid::grid;
use crate::project::ProjectViewState;
use eframe::{
    egui::{style::WidgetVisuals, Frame, Margin, Sense, TextFormat, Widget},
    emath::{Align, RectTransform},
    epaint::{
        pos2, text::LayoutJob, vec2, Color32, FontId, Galley, Rect, RectShape, Shape, Stroke,
        TextShape, Vec2,
    },
};
use ensnare_core::{
    prelude::*,
    types::{ColorScheme, TrackTitle},
};
use ensnare_cores_egui::ColorSchemeConverter;
use ensnare_drag_drop::{DragDropManager, DragSource, DropTarget};
use std::ops::Range;
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

/// Wraps a [TitleBar] as a [Widget](eframe::egui::Widget). Don't have a
/// font_galley? Check out [make_title_bar_galley()].
pub fn title_bar(font_galley: Option<Arc<Galley>>) -> impl eframe::egui::Widget {
    move |ui: &mut eframe::egui::Ui| TitleBar::new(font_galley).ui(ui)
}

/// An egui widget that draws a track's sideways title bar.
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
                            fallback_color: Color32::YELLOW,
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

#[derive(Debug)]
pub struct TrackWidgetInfo<'a> {
    pub track_uid: TrackUid,
    pub signal_items: &'a [SignalChainItem],
    pub title_font_galley: Option<Arc<Galley>>,
    pub color_scheme: ColorScheme,
}

#[derive(Debug, Display)]
pub enum TrackWidgetAction {
    // The user selected an entity with the given uid and name. The UI should
    // show that entity's detail view.
    EntitySelected(Uid, String),
}

/// Wraps a [TrackWidget] as a [Widget](eframe::egui::Widget).
pub fn track_widget<'a>(
    track_info: &'a TrackWidgetInfo<'a>,
    composer: &'a mut Composer,
    view_state: &'a mut ProjectViewState,
    action: &'a mut Option<TrackWidgetAction>,
) -> impl Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        TrackWidget::new(track_info, composer, view_state, action).ui(ui)
    }
}

/// An egui component that draws a track.
#[derive(Debug)]
struct TrackWidget<'a> {
    track_info: &'a TrackWidgetInfo<'a>,
    composer: &'a mut Composer,
    view_state: &'a mut ProjectViewState,

    action: &'a mut Option<TrackWidgetAction>,
}
impl<'a> TrackWidget<'a> {
    const TIMELINE_HEIGHT: f32 = 64.0;
    const TRACK_HEIGHT: f32 = 96.0;

    pub fn new(
        track_info: &'a TrackWidgetInfo<'a>,
        composer: &'a mut Composer,
        view_state: &'a mut ProjectViewState,
        action: &'a mut Option<TrackWidgetAction>,
    ) -> Self {
        Self {
            track_info,
            composer,
            view_state,
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
impl<'a> Widget for TrackWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let track_uid = self.track_info.track_uid;

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
                    let font_galley = self
                        .track_info
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
                        let can_accept = Self::check_drag_source_for_timeline();
                        let _ = DragDropManager::drop_target(ui, can_accept, |ui| {
                            // Determine the rectangle that all the composited
                            // layers will use.
                            let desired_size = vec2(ui.available_width(), Self::TIMELINE_HEIGHT);
                            let (_id, rect) = ui.allocate_space(desired_size);

                            let temp_range =
                                ViewRange(MusicalTime::START..MusicalTime::DURATION_WHOLE);

                            let from_screen = RectTransform::from_to(
                                rect,
                                Rect::from_x_y_ranges(
                                    self.view_state.view_range.0.start.total_units() as f32
                                        ..=self.view_state.view_range.0.end.total_units() as f32,
                                    rect.top()..=rect.bottom(),
                                ),
                            );

                            // The Grid is always disabled and drawn first.
                            let _ = ui
                                .allocate_ui_at_rect(rect, |ui| {
                                    ui.add(grid(
                                        temp_range.clone(),
                                        self.view_state.view_range.clone(),
                                    ))
                                })
                                .inner;

                            // Draw the widget corresponding to the current mode.
                            match self.view_state.arrangement_mode {
                                crate::project::ArrangementViewMode::Composition => {
                                    ui.add_enabled_ui(true, |ui| {
                                        ui.allocate_ui_at_rect(rect, |ui| {
                                            ui.add(track_arrangement(
                                                self.track_info.track_uid,
                                                self.composer,
                                                &self.view_state.view_range,
                                                self.track_info.color_scheme,
                                            ));
                                        });
                                    });
                                }
                                crate::project::ArrangementViewMode::Control(_) => {
                                    ui.add_enabled_ui(true, |ui| {
                                        ui.allocate_ui_at_rect(rect, |ui| ui.label("control!!!!"));
                                    });
                                }
                                crate::project::ArrangementViewMode::SomethingElse => {
                                    ui.add_enabled_ui(true, |ui| {
                                        ui.allocate_ui_at_rect(rect, |ui| {
                                            ui.label("something else111!!!!")
                                        });
                                    });
                                }
                            }

                            // Next, if it's present, draw the cursor.
                            if let Some(position) = self.view_state.cursor {
                                if self.view_state.view_range.0.contains(&position) {
                                    let _ = ui
                                        .allocate_ui_at_rect(rect, |ui| {
                                            ui.add(cursor(
                                                position,
                                                self.view_state.view_range.clone(),
                                            ))
                                        })
                                        .inner;
                                }
                            }

                            let time = if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                let time_pos = from_screen * pointer_pos;
                                let time = MusicalTime::new_with_units(time_pos.x as usize);
                                if self.view_state.view_range.0.contains(&time) {
                                    let _ = ui
                                        .allocate_ui_at_rect(rect, |ui| {
                                            ui.add(cursor(time, self.view_state.view_range.clone()))
                                        })
                                        .inner;
                                }
                                Some(time)
                            } else {
                                None
                            };

                            // Note drag/drop position
                            if let Some(time) = time {
                                ((), DropTarget::TrackPosition(track_uid, time))
                            } else {
                                ((), DropTarget::Track(track_uid))
                            }
                        })
                        .response;

                        // Draw the signal chain view for every kind of track.
                        ui.scope(|ui| {
                            let mut action = None;
                            ui.add(signal_chain_widget(
                                track_uid,
                                self.track_info.signal_items,
                                &mut action,
                            ));

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

/// Wraps a [TrackArrangement] as a [Widget](eframe::egui::Widget).
pub fn track_arrangement<'a>(
    track_uid: TrackUid,
    composer: &'a mut Composer,
    view_range: &'a ViewRange,
    color_scheme: ColorScheme,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        TrackArrangement::new(track_uid, composer, view_range, color_scheme).ui(ui)
    }
}

/// An egui widget that draws a track arrangement overlaid in the track view.
#[derive(Debug)]
struct TrackArrangement<'a> {
    track_uid: TrackUid,
    composer: &'a mut Composer,
    view_range: &'a ViewRange,
    color_scheme: ColorScheme,
}
impl<'a> eframe::egui::Widget for TrackArrangement<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.allocate_ui(vec2(ui.available_width(), 64.0), |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
            let x_range_f32 = self.view_range.0.start.total_units() as f32
                ..=self.view_range.0.end.total_units() as f32;
            let y_range = i8::MAX as f32..=u8::MIN as f32;
            let local_space_rect = Rect::from_x_y_ranges(x_range_f32, y_range);
            let to_screen = RectTransform::from_to(local_space_rect, response.rect);
            let from_screen = to_screen.inverse();

            let (track_foreground_color, track_background_color) =
                ColorSchemeConverter::to_color32(self.color_scheme);

            // Check whether we edited the sequence
            if response.clicked() {
                if let Some(click_pos) = ui.ctx().pointer_interact_pos() {
                    let local_pos = from_screen * click_pos;
                    let time = MusicalTime::new_with_units(local_pos.x as usize).quantized();
                    let key = local_pos.y as u8;
                    let note = Note::new_with(key, time, MusicalTime::DURATION_QUARTER);
                    eprintln!("Saw a click at {time}, note {note:?}");
                    // self.sequencer.toggle_note(note);
                    // self.sequencer.calculate_events();
                }
            }

            let visuals = if ui.is_enabled() {
                ui.ctx().style().visuals.widgets.active
            } else {
                ui.ctx().style().visuals.widgets.inactive
            };

            // Generate all the note shapes
            // let note_shapes: Vec<Shape> = self
            //     .sequencer
            //     .notes()
            //     .iter()
            //     .map(|note| self.shape_for_note(&to_screen, &visuals, note))
            //     .collect();

            // Generate all the pattern note shapes
            if let Some(arrangements) = self.composer.tracks_to_arrangements.get(&self.track_uid) {
                let (pattern_backgrounds, pattern_shapes): (Vec<Shape>, Vec<Shape>) =
                    arrangements.iter().fold(
                        (Vec::default(), Vec::default()),
                        |(mut background_v, mut shape_v), arrangement| {
                            if let Some(pattern) = self.composer.pattern(arrangement.pattern_uid) {
                                background_v.push(Self::background_for_arrangement(
                                    &to_screen,
                                    &visuals,
                                    track_background_color,
                                    arrangement.position
                                        ..(arrangement.position + arrangement.duration),
                                ));
                                pattern.notes().iter().for_each(|note| {
                                    let note = Note {
                                        key: note.key,
                                        range: TimeRange(
                                            (note.range.0.start + arrangement.position)
                                                ..(note.range.0.end + arrangement.position),
                                        ),
                                    };
                                    shape_v.push(Self::shape_for_note(
                                        &to_screen,
                                        &visuals,
                                        track_foreground_color,
                                        &note,
                                    ));
                                });
                            }
                            (background_v, shape_v)
                        },
                    );

                // Paint all the shapes
                // painter.extend(note_shapes);
                painter.extend(pattern_backgrounds);
                painter.extend(pattern_shapes);
            }

            response
        })
        .inner
    }
}
impl<'a> TrackArrangement<'a> {
    fn new(
        track_uid: TrackUid,
        composer: &'a mut Composer,
        view_range: &'a ViewRange,
        color_scheme: ColorScheme,
    ) -> Self {
        Self {
            track_uid,
            composer,
            view_range,
            color_scheme,
        }
    }

    fn shape_for_note(
        to_screen: &RectTransform,
        visuals: &WidgetVisuals,
        foreground_color: Color32,
        note: &Note,
    ) -> Shape {
        let a = to_screen * pos2(note.range.0.start.total_units() as f32, note.key as f32);
        let b = to_screen * pos2(note.range.0.end.total_units() as f32, note.key as f32);
        Shape::Rect(RectShape::new(
            Rect::from_two_pos(a, b),
            visuals.rounding,
            foreground_color,
            Stroke {
                color: foreground_color,
                width: visuals.fg_stroke.width,
            },
        ))
    }

    fn background_for_arrangement(
        to_screen: &RectTransform,
        visuals: &WidgetVisuals,
        background_color: Color32,
        time_range: Range<MusicalTime>,
    ) -> Shape {
        let upper_left = to_screen * pos2(time_range.start.total_units() as f32, 0.0);
        let bottom_right = to_screen * pos2(time_range.end.total_units() as f32, 127.0);
        Shape::Rect(RectShape::new(
            Rect::from_two_pos(upper_left, bottom_right),
            visuals.rounding,
            background_color,
            visuals.fg_stroke,
        ))
    }
}
