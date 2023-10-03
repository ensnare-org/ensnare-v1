// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    control::ControlRouter,
    controllers::ControlAtlas,
    drag_drop::{DragDropManager, DragDropSource},
    entities::prelude::*,
    even_smaller_sequencer::ESSequencer,
    humidifier::Humidifier,
    midi::prelude::*,
    midi_router::MidiRouter,
    piano_roll::PianoRoll,
    prelude::*,
    traits::{prelude::*, Acts},
    uid::IsUid,
    widgets::{track::make_title_bar_galley, UiSize},
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::Range,
    option::Option,
    sync::{Arc, RwLock},
};

/// Identifies a [Track].
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TrackUid(pub usize);
impl Default for TrackUid {
    fn default() -> Self {
        Self(1)
    }
}
impl IsUid for TrackUid {
    fn increment(&mut self) -> &Self {
        self.0 += 1;
        self
    }
}
impl Display for TrackUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug)]
pub enum TrackElementAction {
    MoveDeviceLeft(usize),
    MoveDeviceRight(usize),
    RemoveDevice(usize),
}

#[derive(Debug)]
pub enum TrackDetailAction {}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum TrackAction {
    SetTitle(TrackTitle),
    ToggleDisclosure,
    NewDevice(TrackUid, EntityKey),
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub enum TrackType {
    #[default]
    Midi,
    Audio,
    Aux,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TrackFactory {
    next_uid: TrackUid,
}
impl TrackFactory {
    fn next_uid(&mut self) -> TrackUid {
        let uid = self.next_uid;
        self.next_uid.increment();
        uid
    }

    pub fn midi(&mut self, _piano_roll: &Arc<RwLock<PianoRoll>>) -> Track {
        let uid = self.next_uid();
        let title = TrackTitle(format!("MIDI {}", uid));

        let t = Track {
            uid,
            title,
            ty: TrackType::Midi,
            ..Default::default()
        };
        //        t.sequencer_mut().set_piano_roll(Arc::clone(piano_roll));

        t
    }

    pub fn audio(&mut self) -> Track {
        let uid = self.next_uid();
        let title = TrackTitle(format!("Audio {}", uid));
        Track {
            uid,
            title,
            ty: TrackType::Audio,
            ..Default::default()
        }
    }

    pub fn aux(&mut self) -> Track {
        let uid = self.next_uid();
        let title = TrackTitle(format!("Aux {}", uid));
        Track {
            uid,
            title,
            ty: TrackType::Aux,
            ..Default::default()
        }
    }
}
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

/// Newtype for track title string.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackTitle(pub String);
impl Default for TrackTitle {
    fn default() -> Self {
        Self("Untitled".to_string())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub enum TrackUiState {
    #[default]
    Collapsed,
    Expanded,
}

#[derive(Debug, Default)]
pub struct TrackEphemerals {
    buffer: TrackBuffer,
    piano_roll: Arc<RwLock<PianoRoll>>,
    pub(crate) action: Option<TrackAction>,
    view_range: std::ops::Range<MusicalTime>,
    pub(crate) title_font_galley: Option<Arc<eframe::epaint::Galley>>,
}

/// A collection of instruments, effects, and controllers that combine to
/// produce a single source of audio.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Track {
    uid: TrackUid,
    title: TrackTitle,
    ty: TrackType,

    pub(crate) entity_store: EntityStore,

    #[deprecated]
    pub(crate) sequencer: ESSequencer,
    midi_router: MidiRouter,

    /// [ControlAtlas] manages the sources of Control events. It generates
    /// events but does not handle their routing.
    #[deprecated]
    pub(crate) control_atlas: ControlAtlas,
    /// [ControlRouter] manages the destinations of Control events. It does not
    /// generate events, but when events are generated, it knows where to route
    /// them.
    pub(crate) control_router: ControlRouter,

    pub(crate) controllers: Vec<Uid>,
    pub(crate) instruments: Vec<Uid>,
    pub(crate) effects: Vec<Uid>,

    humidifier: Humidifier,

    #[serde(skip)]
    pub(crate) e: TrackEphemerals,
}
impl Track {
    #[allow(missing_docs)]
    pub fn is_aux(&self) -> bool {
        matches!(self.ty, TrackType::Aux)
    }

    // TODO: for now the only way to add something new to a Track is to append it.
    #[allow(missing_docs)]
    pub fn append_entity(&mut self, entity: Box<dyn Entity>) -> anyhow::Result<Uid> {
        let uid = entity.uid();

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
            self.midi_router.connect(uid, MidiChannel(0));
        }

        self.entity_store.add(entity)
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

    /// Returns the [Entity] having the given [Uid], if it exists.
    pub fn entity(&self, uid: &Uid) -> Option<&Box<dyn Entity>> {
        self.entity_store.get(uid)
    }

    /// Returns the mutable [Entity] having the given [Uid], if it exists.
    pub fn entity_mut(&mut self, uid: &Uid) -> Option<&mut Box<dyn Entity>> {
        self.entity_store.get_mut(uid)
    }

    pub(crate) fn track_view_height(track_type: TrackType, ui_state: TrackUiState) -> f32 {
        if matches!(track_type, TrackType::Aux) {
            Self::device_view_height(ui_state)
        } else {
            Self::timeline_view_height(ui_state) + Self::device_view_height(ui_state)
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

    #[allow(missing_docs)]
    pub fn remove_selected_patterns(&mut self) {
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

    #[allow(missing_docs)]
    pub fn uid(&self) -> TrackUid {
        self.uid
    }

    pub(crate) fn ty(&self) -> TrackType {
        self.ty
    }

    #[allow(missing_docs)]
    pub fn set_piano_roll(&mut self, piano_roll: Arc<RwLock<PianoRoll>>) {
        self.e.piano_roll = Arc::clone(&piano_roll);
        //self.sequencer.set_piano_roll(piano_roll);
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

    pub(crate) fn calculate_max_entity_uid(&self) -> Option<Uid> {
        self.entity_store.calculate_max_entity_uid()
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

    /// Returns the [ControlRouter].
    pub fn control_router_mut(&mut self) -> &mut ControlRouter {
        &mut self.control_router
    }

    /// Returns an immutable reference to the internal buffer.
    pub fn buffer(&self) -> &TrackBuffer {
        &self.e.buffer
    }

    /// Returns a writable version of the internal buffer.
    pub fn buffer_mut(&mut self) -> &mut TrackBuffer {
        &mut self.e.buffer
    }

    pub fn title_mut(&mut self) -> &mut TrackTitle {
        &mut self.title
    }

    pub fn view_range(&self) -> &std::ops::Range<MusicalTime> {
        &self.e.view_range
    }

    /// The [TitleBar] widget needs a Galley so that it can display the title
    /// sideways. But widgets live for only a frame, so it can't cache anything.
    /// Caller to the rescue! We generate the Galley and save it.
    ///
    /// TODO: when we allow title editing, we should set the galley to None so
    /// it can be rebuilt on the next frame.
    pub(crate) fn update_font_galley(&mut self, ui: &mut eframe::egui::Ui) {
        if self.e.title_font_galley.is_none() && !self.title.0.is_empty() {
            self.e.title_font_galley = Some(make_title_bar_galley(ui, &self.title));
        }
    }
}
impl Acts for Track {
    type Action = TrackAction;

    fn take_action(&mut self) -> Option<Self::Action> {
        self.e.action.take()
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

        if !self.is_aux() {
            // We're a regular track. Start with a fresh buffer and let each
            // instrument do its thing.
            self.e.buffer.0.fill(StereoSample::SILENCE);
        } else {
            // We're an aux track. We leave the internal buffer as-is, with the
            // expectation that the caller has already filled it with the signal
            // we should be processing.
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

        // TODO: change this trait to operate on batches.
        for uid in self.effects.iter() {
            if let Some(e) = self.entity_store.get_mut(uid) {
                if let Some(e) = e.as_effect_mut() {
                    let humidity = self.humidifier.get_humidity_by_uid(uid);
                    if humidity == Normal::zero() {
                        continue;
                    }
                    for sample in self.e.buffer.0.iter_mut() {
                        *sample = self.humidifier.transform_audio(
                            humidity,
                            *sample,
                            e.transform_audio(*sample),
                        );
                    }
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
        self.entity_store.update_time(range);
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        // Create a place to hold MIDI messages that we need to route.
        let mut midi_events = Vec::default();

        // Peek at incoming events before handing them to control_events_fn.
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

        // Let everyone work and possibly generate messages.
        self.entity_store.work(&mut handler);

        // We've accumulated all the MIDI messages. Route them to our own
        // MidiRouter. They've already been forwarded to the caller via
        // control_events_fn.
        midi_events.into_iter().for_each(|(channel, message)| {
            let _ = self
                .midi_router
                .route(&mut self.entity_store, channel, message);
        });
    }

    fn is_finished(&self) -> bool {
        self.entity_store.is_finished()
    }

    fn play(&mut self) {
        self.entity_store.play();
    }

    fn stop(&mut self) {
        self.entity_store.stop();
    }

    fn skip_to_start(&mut self) {
        self.entity_store.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.entity_store.is_performing()
    }
}
impl Serializable for Track {
    fn after_deser(&mut self) {
        self.entity_store.after_deser();
    }
}
impl DisplaysInTimeline for Track {
    fn set_view_range(&mut self, view_range: &Range<MusicalTime>) {
        self.e.view_range = view_range.clone();
    }
}
impl Displays for Track {}

#[derive(Debug)]
pub enum DeviceChainAction {
    NewDevice(EntityKey),
    LinkControl(Uid, Uid, ControlIndex),
}

#[derive(Debug)]
pub struct DeviceChain<'a> {
    #[allow(dead_code)]
    track_uid: TrackUid,
    store: &'a mut EntityStore,
    controllers: &'a mut Vec<Uid>,
    instruments: &'a mut Vec<Uid>,
    effects: &'a mut Vec<Uid>,

    action: &'a mut Option<DeviceChainAction>,

    ui_size: UiSize,
}
impl<'a> DeviceChain<'a> {
    pub fn new(
        track_uid: TrackUid,
        store: &'a mut EntityStore,
        controllers: &'a mut Vec<Uid>,
        instruments: &'a mut Vec<Uid>,
        effects: &'a mut Vec<Uid>,
        action: &'a mut Option<DeviceChainAction>,
    ) -> Self {
        Self {
            track_uid,
            store,
            controllers,
            instruments,
            effects,
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
                *self.action = Some(DeviceChainAction::NewDevice(key))
            }
        }
    }
}
impl<'a> Displays for DeviceChain<'a> {
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
                        self.controllers
                            .iter()
                            .chain(self.instruments.iter().chain(self.effects.iter()))
                            .for_each(|uid| {
                                if let Some(entity) = self.store.get_mut(uid) {
                                    eframe::egui::CollapsingHeader::new(entity.name())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::factory::test_entities::{
        TestEffect, TestInstrument, TestInstrumentCountsMidiMessages,
    };

    #[test]
    fn basic_track_operations() {
        let mut t = Track::default();
        assert!(t.controllers.is_empty());
        assert!(t.effects.is_empty());
        assert!(t.instruments.is_empty());

        // Create an instrument and add it to a track.
        let mut instrument = TestInstrument::default();
        instrument.set_uid(Uid(1));
        let id1 = t.append_entity(Box::new(instrument)).unwrap();

        // Add a second instrument to the track.
        let mut instrument = TestInstrument::default();
        instrument.set_uid(Uid(2));
        let id2 = t.append_entity(Box::new(instrument)).unwrap();

        assert_ne!(id1, id2, "Don't forget to assign UIDs!");

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

        let instrument = t.remove_entity(&id1).unwrap();
        assert_eq!(instrument.uid(), id1, "removed the right instrument");
        assert_eq!(t.instruments.len(), 1, "removed exactly one instrument");
        assert_eq!(
            t.instruments[0], id2,
            "the remaining instrument should be the one we left"
        );
        assert!(
            t.entity_store.get(&id1).is_none(),
            "it should be gone from the store"
        );

        let mut effect = TestEffect::default();
        effect.set_uid(Uid(3));
        let effect_id1 = t.append_entity(Box::new(effect)).unwrap();
        let mut effect = TestEffect::default();
        effect.set_uid(Uid(4));
        let effect_id2 = t.append_entity(Box::new(effect)).unwrap();

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

        let mut sender = ToyControllerAlwaysSendsMidiMessage::default();
        sender.set_uid(Uid(2001));
        let _sender_id = t.append_entity(Box::new(sender)).unwrap();

        let mut receiver = TestInstrumentCountsMidiMessages::default();
        receiver.set_uid(Uid(2002));
        let counter = Arc::clone(receiver.received_midi_message_count_mutex());
        let _receiver_id = t.append_entity(Box::new(receiver)).unwrap();

        let mut external_midi_messages = 0;
        t.play();
        t.work(&mut |_uid, _event| {
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
