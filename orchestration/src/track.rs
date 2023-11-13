// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    control_router::ControlRouter, egui::title_bar, humidifier::Humidifier, midi_router::MidiRouter,
};
use anyhow::anyhow;
use eframe::{
    egui::{ImageButton, Widget},
    emath::RectTransform,
    epaint::{vec2, Color32, Rect, Vec2},
};
use ensnare_core::{
    controllers::ControlTrip,
    piano_roll::{PatternUid, PianoRoll},
    prelude::*,
    traits::{ControlProxyEventsFn, ControlsAsProxy, Sequences},
    types::TrackTitle,
};
use ensnare_cores::LivePatternSequencer;
use ensnare_cores_egui::{
    prelude::{live_pattern_sequencer_widget, trip},
    widgets::timeline::{cursor, grid},
};
use ensnare_drag_drop::{DragDropManager, DragSource, DropTarget};
use ensnare_entity::prelude::*;
use std::{
    collections::HashMap,
    ops::Range,
    option::Option,
    sync::{Arc, RwLock},
};
use strum_macros::Display;

#[derive(Debug)]
pub struct TrackBuffer(pub [StereoSample; Self::LEN]);
impl TrackBuffer {
    pub const LEN: usize = 64;
}
impl Default for TrackBuffer {
    fn default() -> Self {
        Self([StereoSample::default(); Self::LEN])
    }
}

#[derive(Debug, Default)]
pub struct TrackEphemerals {
    buffer: TrackBuffer,
    pub piano_roll: Arc<RwLock<PianoRoll>>,
    view_range: ViewRange,
    pub title_font_galley: Option<Arc<eframe::epaint::Galley>>,
}

/// A collection of instruments, effects, and controllers that combine to
/// produce a single source of audio.
#[derive(Debug, Default)]
pub struct Track {
    pub title: TrackTitle,
    pub entity_store: EntityStore,
    pub sequencer: LivePatternSequencer,
    pub midi_router: MidiRouter,
    pub control_router: ControlRouter,
    pub control_trips: HashMap<Uid, ControlTrip>,
    pub controllers: Vec<Uid>,
    pub instruments: Vec<Uid>,
    pub effects: Vec<Uid>,
    humidifier: Humidifier,

    pub e: TrackEphemerals,
}
impl Track {
    // TODO: for now the only way to add something new to a Track is to append
    // it.
    #[allow(missing_docs)]
    pub fn append_entity(&mut self, entity: Box<dyn Entity>, uid: Uid) -> anyhow::Result<()> {
        // Some entities are hybrids, so they can appear in multiple lists.
        // That's why we don't have if-else here.
        if entity.as_controller().is_some() {
            self.controllers.push(uid);
        }
        if entity.as_effect().is_some() {
            self.effects.push(uid);
        }
        if entity.as_instrument().is_some() {
            self.instruments.push(uid);
        }
        if entity.as_handles_midi().is_some() {
            // TODO: for now, everyone's on channel 0
            self.midi_router.connect(uid, MidiChannel::default());
        }
        self.entity_store.add(entity, uid)
    }

    #[allow(missing_docs)]
    pub fn remove_entity(&mut self, uid: &Uid) -> Option<Box<dyn Entity>> {
        if let Some(entity) = self.entity_store.remove(uid) {
            if entity.as_controller().is_some() {
                self.controllers.retain(|e| e != uid)
            }
            if entity.as_effect().is_some() {
                self.effects.retain(|e| e != uid);
            }
            if entity.as_instrument().is_some() {
                self.instruments.retain(|e| e != uid);
            }
            Some(entity)
        } else {
            None
        }
    }

    pub fn set_sequencer(&mut self, sequencer: LivePatternSequencer) {
        self.sequencer = sequencer;
    }

    /// Returns the [Entity] having the given [Uid], if it exists.
    pub fn entity(&self, uid: &Uid) -> Option<&Box<dyn Entity>> {
        self.entity_store.get(uid)
    }

    /// Returns the mutable [Entity] having the given [Uid], if it exists.
    pub fn entity_mut(&mut self, uid: &Uid) -> Option<&mut Box<dyn Entity>> {
        self.entity_store.get_mut(uid)
    }

    #[allow(missing_docs)]
    pub fn remove_selected_patterns(&mut self) {
        todo!()
        //        self.sequencer.remove_selected_arranged_patterns();
    }

    #[allow(missing_docs)]
    pub fn route_midi_message(&mut self, channel: MidiChannel, message: MidiMessage) {
        if let Err(e) = self
            .midi_router
            .route(&mut self.entity_store, channel, message)
        {
            eprintln!("While routing: {e}");
        }
    }

    #[allow(missing_docs)]
    pub fn route_control_change(&mut self, uid: Uid, value: ControlValue) {
        if let Err(e) = self.control_router.route(
            &mut |target_uid, index, value| {
                if let Some(e) = self.entity_store.get_mut(target_uid) {
                    if let Some(e) = e.as_controllable_mut() {
                        e.control_set_param_by_index(index, value);
                    }
                }
            },
            uid,
            value,
        ) {
            eprintln!("While routing control change: {e}")
        }
    }

    pub(crate) fn set_title(&mut self, title: TrackTitle) {
        self.title = title;
    }

    /// Sets the wet/dry of an effect in the chain.
    pub fn set_humidity(&mut self, effect_uid: Uid, humidity: Normal) -> anyhow::Result<()> {
        if let Some(entity) = self.entity(&effect_uid) {
            if entity.as_effect().is_some() {
                self.humidifier.set_humidity_by_uid(effect_uid, humidity);
                Ok(())
            } else {
                Err(anyhow!("{effect_uid} is not an effect"))
            }
        } else {
            Err(anyhow!("{effect_uid} not found"))
        }
    }

    /// Moves the indicated effect to a new position within the effects chain.
    /// Zero is the first position.
    pub fn move_effect(&mut self, uid: Uid, new_index: usize) -> anyhow::Result<()> {
        if new_index >= self.effects.len() {
            Err(anyhow!(
                "Can't move {uid} to {new_index} when we have only {} items",
                self.effects.len()
            ))
        } else if self.effects.contains(&uid) {
            self.effects.retain(|e| e != &uid);
            self.effects.insert(new_index, uid);
            Ok(())
        } else {
            Err(anyhow!("Effect {uid} not found"))
        }
    }

    /// Returns an immutable reference to the internal buffer.
    pub fn buffer(&self) -> &TrackBuffer {
        &self.e.buffer
    }

    /// Returns a writable version of the internal buffer.
    pub fn buffer_mut(&mut self) -> &mut TrackBuffer {
        &mut self.e.buffer
    }

    pub fn view_range(&self) -> &ViewRange {
        &self.e.view_range
    }

    pub(crate) fn add_pattern(
        &mut self,
        pattern_uid: &PatternUid,
        position: MusicalTime,
    ) -> Result<(), anyhow::Error> {
        self.sequencer
            .record(MidiChannel::default(), pattern_uid, position)
    }
}
impl GeneratesToInternalBuffer<StereoSample> for Track {
    fn generate_batch_values(&mut self, len: usize) -> usize {
        if len > self.e.buffer.0.len() {
            eprintln!(
                "requested {} samples but buffer is only len {}",
                len,
                self.e.buffer.0.len()
            );
            return 0;
        }

        for uid in self.instruments.iter() {
            if let Some(e) = self.entity_store.get_mut(uid) {
                if let Some(e) = e.as_instrument_mut() {
                    // Note that we're expecting everyone to ADD to the buffer,
                    // not to overwrite! TODO: convert all instruments to have
                    // internal buffers
                    e.generate_batch_values(&mut self.e.buffer.0);
                }
            }
        }

        for uid in self.effects.iter() {
            if let Some(e) = self.entity_store.get_mut(uid) {
                if let Some(e) = e.as_effect_mut() {
                    let humidity = self.humidifier.get_humidity_by_uid(uid);
                    if humidity == Normal::zero() {
                        continue;
                    }
                    self.humidifier
                        .transform_batch(humidity, e, &mut self.e.buffer.0);
                }
            }
        }

        // See #146 TODO - at this point we might want to gather any events
        // produced during the effects stage.

        self.e.buffer.0.len()
    }

    fn values(&self) -> &[StereoSample] {
        &self.e.buffer.0
    }
}
impl Ticks for Track {
    fn tick(&mut self, tick_count: usize) {
        self.entity_store.tick(tick_count);
    }
}
impl Configurable for Track {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.entity_store.update_sample_rate(sample_rate);
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.entity_store.update_tempo(tempo);
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.entity_store.update_time_signature(time_signature);
    }
}

// TODO: I think this is wrong and misguided. If MIDI messages are handled by
// Track, then each Track needs to record who's receiving on which channel, and
// messages can't be sent from a device on one track to one on a different
// track. While that could make parallelism easier, it doesn't seem intuitively
// correct, because in a real studio you'd be able to hook up MIDI cables
// independently of audio cables.
#[cfg(never)]
impl HandlesMidi for Track {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        messages_fn: &mut dyn FnMut(Uid, MidiChannel, MidiMessage),
    ) {
        for e in self.controllers.iter_mut() {
            e.handle_midi_message(channel, &message, messages_fn);
        }
        for e in self.instruments.iter_mut() {
            e.handle_midi_message(channel, &message, messages_fn);
        }
    }
}
impl Controls for Track {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.sequencer.update_time(range);
        self.control_trips
            .values_mut()
            .for_each(|ct| ct.update_time(range));
        self.entity_store.update_time(range);
    }

    fn work(&mut self, _: &mut ControlEventsFn) {
        unimplemented!("Look at ControlsAsProxy")
    }

    fn is_finished(&self) -> bool {
        self.sequencer.is_finished()
            && self.control_trips.values().all(|ct| ct.is_finished())
            && self.entity_store.is_finished()
    }

    fn play(&mut self) {
        self.sequencer.play();
        self.control_trips.values_mut().for_each(|ct| ct.play());
        self.entity_store.play();
    }

    fn stop(&mut self) {
        self.sequencer.stop();
        self.control_trips.values_mut().for_each(|ct| ct.stop());
        self.entity_store.stop();
    }

    fn skip_to_start(&mut self) {
        self.sequencer.skip_to_start();
        self.control_trips
            .values_mut()
            .for_each(|ct| ct.skip_to_start());
        self.entity_store.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.sequencer.is_performing()
            || self.control_trips.values().any(|ct| ct.is_performing())
            || self.entity_store.is_performing()
    }
}
impl ControlsAsProxy for Track {
    fn work_as_proxy(&mut self, control_events_fn: &mut ControlProxyEventsFn) {
        // Create a place to hold MIDI messages that we need to route.
        let mut midi_events = Vec::default();

        // Let everyone work and possibly generate messages.
        self.sequencer.work(&mut |event| {
            match event {
                // We need to route MIDI messages to all eligible Entities in
                // this Track, so we save them up.
                EntityEvent::Midi(channel, message) => {
                    midi_events.push((channel, message));
                }
                EntityEvent::Control(_) => {}
            }
            control_events_fn(Uid::default(), event);
        });

        self.control_trips.iter_mut().for_each(|(uid, ct)| {
            // Since ControlTrips don't know their own Uid, we have to provide
            // it for them.
            let mut uid_handler = |event| {
                control_events_fn(*uid, event);
            };
            ct.work(&mut uid_handler)
        });

        let mut handler = |uid, event| {
            match event {
                // We need to route MIDI messages to all eligible Entities in
                // this Track, so we save them up.
                EntityEvent::Midi(channel, message) => {
                    midi_events.push((channel, message));
                }
                EntityEvent::Control(_) => {}
            }
            control_events_fn(uid, event);
        };
        self.entity_store.work_as_proxy(&mut handler);

        // We've accumulated all the MIDI messages. Route them to our own
        // MidiRouter. They've already been forwarded to the caller via
        // control_events_fn.
        midi_events.into_iter().for_each(|(channel, message)| {
            let _ = self
                .midi_router
                .route(&mut self.entity_store, channel, message);
        });
    }
}
impl Serializable for Track {
    fn after_deser(&mut self) {
        // TODO: I think here is where we'd tell the sequencer about the piano
        // roll again (which will be hard).
        self.entity_store.after_deser();
        self.control_trips
            .values_mut()
            .for_each(|ct| ct.after_deser());
    }
}

/// Wraps a [TrackWidget] as a [Widget](eframe::egui::Widget).
pub fn track_widget<'a>(
    track_uid: TrackUid,
    track: &'a mut Track,
    is_selected: bool,
    cursor: Option<MusicalTime>,
    view_range: &'a ViewRange,
    action: &'a mut Option<TrackWidgetAction>,
) -> impl Widget + 'a {
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
impl<'a> Widget for TrackWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
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
                            self.track.control_trips.iter_mut().for_each(|(uid, t)| {
                                ui.add_enabled_ui(enabled, |ui| {
                                    ui.allocate_ui_at_rect(rect, |ui| {
                                        ui.add(trip(
                                            *uid,
                                            t,
                                            self.track.control_router.control_links(*uid),
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
                            ui.add(signal_chain(self.track_uid, self.track, &mut action));
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
    fn check_drag_source_for_timeline() -> bool {
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
) -> impl Widget + 'a {
    move |ui: &mut eframe::egui::Ui| SignalChainWidget::new(track_uid, track, action).ui(ui)
}

#[derive(Debug, Display)]
pub enum SignalChainWidgetAction {
    EntitySelected(Uid, String),
}

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
impl<'a> Widget for SignalChainWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
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
) -> impl Widget + 'a {
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
impl Widget for SignalChainItem {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ensnare_entities::{
        effects::TestEffect,
        instruments::{TestInstrument, TestInstrumentCountsMidiMessages},
    };
    use ensnare_entities_toy::prelude::ToyControllerAlwaysSendsMidiMessage;

    #[test]
    fn basic_track_operations() {
        let mut t = Track::default();
        assert!(t.controllers.is_empty());
        assert!(t.effects.is_empty());
        assert!(t.instruments.is_empty());

        // Create an instrument and add it to a track.
        let id1 = Uid(1);
        assert!(t
            .append_entity(Box::new(TestInstrument::default()), id1)
            .is_ok());

        // Add a second instrument to the track.
        let id2 = Uid(2);
        assert!(t
            .append_entity(Box::new(TestInstrument::default()), id2)
            .is_ok());

        assert_ne!(id1, id2, "Don't forget to assign unique IDs!");

        assert_eq!(
            t.instruments[0], id1,
            "first appended entity should be at index 0"
        );
        assert_eq!(
            t.instruments[1], id2,
            "second appended entity should be at index 1"
        );
        assert_eq!(
            t.instruments.len(),
            2,
            "there should be exactly as many entities as added"
        );

        let _ = t.remove_entity(&id1).unwrap();
        assert_eq!(t.instruments.len(), 1, "removed exactly one instrument");
        assert_eq!(
            t.instruments[0], id2,
            "the remaining instrument should be the one we left"
        );
        assert!(
            t.entity_store.get(&id1).is_none(),
            "it should be gone from the store"
        );

        let effect_id1 = Uid(3);
        assert!(t
            .append_entity(Box::new(TestEffect::default()), effect_id1)
            .is_ok());
        let effect_id2 = Uid(4);
        assert!(t
            .append_entity(Box::new(TestEffect::default()), effect_id2,)
            .is_ok());

        assert_eq!(t.effects[0], effect_id1);
        assert_eq!(t.effects[1], effect_id2);
        assert!(t.move_effect(effect_id1, 1).is_ok());
        assert_eq!(
            t.effects[0], effect_id2,
            "After moving effects, id2 should be first"
        );
        assert_eq!(t.effects[1], effect_id1);
    }

    // We expect that a MIDI message will be routed to the eligible Entities in
    // the same Track, and forwarded to the work() caller, presumably to decide
    // whether to send it to other destination(s) such as external MIDI
    // interfaces.
    #[test]
    fn midi_messages_sent_to_caller_and_sending_track_instruments() {
        let mut t = Track::default();

        assert!(t
            .append_entity(
                Box::new(ToyControllerAlwaysSendsMidiMessage::default()),
                Uid(2001),
            )
            .is_ok());

        let receiver = TestInstrumentCountsMidiMessages::default();
        let counter = Arc::clone(receiver.received_midi_message_count_mutex());
        assert!(t.append_entity(Box::new(receiver), Uid(2002)).is_ok());

        let mut external_midi_messages = 0;
        t.play();
        t.work_as_proxy(&mut |_, _| {
            external_midi_messages += 1;
        });

        if let Ok(c) = counter.lock() {
            assert_eq!(
                *c, 1,
                "The receiving instrument in the track should have received the message"
            );
        };

        assert_eq!(
            external_midi_messages, 1,
            "After one work(), one MIDI message should have emerged for external processing"
        );
    }
}
