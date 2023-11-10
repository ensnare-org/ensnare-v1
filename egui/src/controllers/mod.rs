// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::control::ControlRouter;
use eframe::{
    egui::{style::WidgetVisuals, Sense, Widget},
    emath::RectTransform,
    epaint::{pos2, vec2, Color32, Rect, RectShape, Shape, Stroke},
};
use ensnare_core::{
    controllers::{ControlTripPath, LivePatternSequencer},
    generators::Waveform,
    piano_roll::Note,
    prelude::*,
    stuff::arpeggiator::{Arpeggiator, ArpeggioMode},
    time::{MusicalTime, ViewRange},
    types::FrequencyRange,
    uid::Uid,
};
use ensnare_egui_widgets::{frequency, waveform};
use strum::IntoEnumIterator;

/// Wraps a [Trip] as a [Widget](eframe::egui::Widget).
pub fn trip<'a>(
    uid: Uid,
    trip: &'a mut ensnare_core::controllers::ControlTrip,
    control_router: &'a mut ControlRouter,
    view_range: ViewRange,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| Trip::new(uid, trip, control_router, view_range).ui(ui)
}

#[derive(Debug)]
struct Trip<'a> {
    uid: Uid,
    control_trip: &'a mut ensnare_core::controllers::ControlTrip,
    control_router: &'a mut ControlRouter,
    view_range: ViewRange,
}
impl<'a> Trip<'a> {
    fn new(
        uid: Uid,
        control_trip: &'a mut ensnare_core::controllers::ControlTrip,
        control_router: &'a mut ControlRouter,
        view_range: ViewRange,
    ) -> Self {
        Self {
            uid,
            control_trip,
            control_router,
            view_range,
        }
    }
}
impl<'a> Widget for Trip<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(
                self.view_range.start.total_units() as f32
                    ..=self.view_range.end.total_units() as f32,
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
                        to_screen * pos2(self.view_range.end.total_units() as f32, 0.0);
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
            let label = if let Some(links) = self.control_router.control_links(self.uid) {
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
                self.control_router
                    .link_control(self.uid, Uid(234), ControlIndex(456));
            }
        }

        response
    }
}

/// Wraps a [LivePatternSequencerWidget] as a [Widget](eframe::egui::Widget).
pub fn live_pattern_sequencer_widget<'a>(
    sequencer: &'a mut LivePatternSequencer,
    view_range: &'a ViewRange,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| LivePatternSequencerWidget::new(sequencer, view_range).ui(ui)
}

/// An egui widget that draws a legend on the horizontal axis of the timeline
/// view.
#[derive(Debug)]
pub struct LivePatternSequencerWidget<'a> {
    sequencer: &'a mut LivePatternSequencer,
    view_range: ViewRange,
}
impl<'a> LivePatternSequencerWidget<'a> {
    fn new(sequencer: &'a mut LivePatternSequencer, view_range: &'a ViewRange) -> Self {
        Self {
            sequencer,
            view_range: view_range.clone(),
        }
    }

    fn shape_for_note(to_screen: &RectTransform, visuals: &WidgetVisuals, note: &Note) -> Shape {
        Shape::Rect(RectShape::new(
            Rect::from_two_pos(
                to_screen * pos2(note.range.start.total_units() as f32, note.key as f32),
                to_screen * pos2(note.range.end.total_units() as f32, note.key as f32),
            ),
            visuals.rounding,
            visuals.bg_fill,
            visuals.fg_stroke,
        ))
    }
}

impl<'a> Widget for LivePatternSequencerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.allocate_ui(vec2(ui.available_width(), 64.0), |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
            let x_range_f32 = self.view_range.start.total_units() as f32
                ..=self.view_range.end.total_units() as f32;
            let y_range = i8::MAX as f32..=u8::MIN as f32;
            let local_space_rect = Rect::from_x_y_ranges(x_range_f32, y_range);
            let to_screen = RectTransform::from_to(local_space_rect, response.rect);
            let from_screen = to_screen.inverse();

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
            let pattern_shapes: Vec<Shape> = self.sequencer.inner.patterns.iter().fold(
                Vec::default(),
                |mut v, (_channel, pattern)| {
                    pattern.notes().iter().for_each(|note| {
                        let note = Note {
                            key: note.key,
                            range: (note.range.start)..(note.range.end),
                        };
                        v.push(Self::shape_for_note(&to_screen, &visuals, &note));
                    });
                    v
                },
            );

            // Paint all the shapes
            //            painter.extend(note_shapes);
            painter.extend(pattern_shapes);

            response
        })
        .inner
    }
}

/// Wraps a [PatternSequencerWidget] as a [Widget](eframe::egui::Widget).
pub fn pattern_sequencer_widget<'a>(
    sequencer: &'a mut ensnare_core::entities::controllers::sequencers::PatternSequencer,
    view_range: &'a ViewRange,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| PatternSequencerWidget::new(sequencer, view_range).ui(ui)
}

/// An egui widget that draws a legend on the horizontal axis of the timeline
/// view.
#[derive(Debug)]
pub struct PatternSequencerWidget<'a> {
    sequencer: &'a mut ensnare_core::entities::controllers::sequencers::PatternSequencer,
    view_range: ViewRange,
}
impl<'a> PatternSequencerWidget<'a> {
    fn new(
        sequencer: &'a mut ensnare_core::entities::controllers::sequencers::PatternSequencer,
        view_range: &'a ViewRange,
    ) -> Self {
        Self {
            sequencer,
            view_range: view_range.clone(),
        }
    }
}
impl<'a> Widget for PatternSequencerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        todo!()
    }
}

/// Wraps a [NoteSequencerWidget] as a [Widget](eframe::egui::Widget).
pub fn note_sequencer_widget<'a>(
    sequencer: &'a mut ensnare_core::entities::controllers::sequencers::NoteSequencer,
    view_range: &'a ViewRange,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| NoteSequencerWidget::new(sequencer, view_range).ui(ui)
}

/// An egui widget that draws a legend on the horizontal axis of the timeline
/// view.
#[derive(Debug)]
pub struct NoteSequencerWidget<'a> {
    sequencer: &'a mut ensnare_core::entities::controllers::sequencers::NoteSequencer,
    view_range: ViewRange,
}
impl<'a> NoteSequencerWidget<'a> {
    fn new(
        sequencer: &'a mut ensnare_core::entities::controllers::sequencers::NoteSequencer,
        view_range: &'a ViewRange,
    ) -> Self {
        Self {
            sequencer,
            view_range: view_range.clone(),
        }
    }
}
impl<'a> Widget for NoteSequencerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        todo!()
    }
}

/// Wraps a [ArpeggiatorWidget] as a [Widget](eframe::egui::Widget).
pub fn arpeggiator<'a>(entity: &'a mut Arpeggiator) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| ArpeggiatorWidget::new(entity).ui(ui)
}

/// Renders [Arpeggiator] in egui.
#[derive(Debug)]
struct ArpeggiatorWidget<'a> {
    inner: &'a mut Arpeggiator,
}
impl<'a> ArpeggiatorWidget<'a> {
    fn new(entity: &'a mut Arpeggiator) -> Self {
        Self { inner: entity }
    }
}
impl<'a> Widget for ArpeggiatorWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut r = eframe::egui::ComboBox::from_label("Scale")
            .selected_text(self.inner.mode.to_string())
            .show_ui(ui, |ui| {
                let mut bool_response = false;
                for mode in ArpeggioMode::iter() {
                    let mode_str: &'static str = mode.into();
                    if ui
                        .selectable_value(&mut self.inner.mode, mode, mode_str)
                        .changed()
                    {
                        bool_response = true;
                    }
                }
                bool_response
            });
        if let Some(inner) = r.inner {
            if inner {
                r.response.mark_changed();
            }
        }
        r.response
    }
}

/// Wraps an [LfoControllerWidget] as a [Widget](eframe::egui::Widget).
pub fn lfo_controller<'a>(
    waveform: &'a mut Waveform,
    frequency: &'a mut FrequencyHz,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| LfoControllerWidget::new(waveform, frequency).ui(ui)
}

#[derive(Debug)]
struct LfoControllerWidget<'a> {
    waveform: &'a mut Waveform,
    frequency: &'a mut FrequencyHz,
}
impl<'a> Widget for LfoControllerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(waveform(self.waveform))
            | ui.add(frequency(FrequencyRange::Subaudible, self.frequency))
    }
}
impl<'a> LfoControllerWidget<'a> {
    pub fn new(waveform: &'a mut Waveform, frequency: &'a mut FrequencyHz) -> Self {
        Self {
            waveform,
            frequency,
        }
    }
}
