// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    control::ControlRouter,
    controllers::{ControlTrip, LivePatternSequencer},
    humidifier::Humidifier,
    midi::prelude::*,
    midi_router::MidiRouter,
    piano_roll::{PatternUid, PianoRoll},
    prelude::*,
    traits::{prelude::*, Sequences},
    uid::{IsUid, UidFactory},
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    ops::Range,
    option::Option,
    sync::{atomic::AtomicUsize, Arc, RwLock},
};
use strum_macros::Display;

/// Identifies a [Track].
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TrackUid(pub usize);
impl Default for TrackUid {
    fn default() -> Self {
        Self(1)
    }
}
impl IsUid for TrackUid {}
impl From<usize> for TrackUid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for TrackUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
pub type TrackUidFactory = UidFactory<TrackUid>;
impl UidFactory<TrackUid> {
    pub const FIRST_UID: AtomicUsize = AtomicUsize::new(1);
}
impl Default for UidFactory<TrackUid> {
    fn default() -> Self {
        Self {
            next_uid_value: Self::FIRST_UID,
            _phantom: Default::default(),
        }
    }
}

/// A [TrackAction] represents any UI operation that happens to a [Track] but
/// that the [Track] can't perform itself.  
#[derive(Clone, Debug, Display)]
pub enum TrackAction {
    /// Using the [EntityFactory], create a new entity of type [EntityKey] and
    /// add it to the track. [Track]s can't do this themselves because they
    /// don't have access to [EntityFactory] (or at least we've decided they
    /// shouldn't).
    NewDevice(EntityKey),

    /// Establish a control link between the source and target uids for the
    /// given parameter.
    LinkControl(Uid, Uid, ControlIndex),

    /// An entity has been selected, and we should show its detail view.
    EntitySelected(Uid),
}
impl IsAction for TrackAction {}

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
impl From<&str> for TrackTitle {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Default)]
pub struct TrackEphemerals {
    buffer: TrackBuffer,
    pub piano_roll: Arc<RwLock<PianoRoll>>,
    pub action: Option<TrackAction>,
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
impl Acts for Track {
    type Action = TrackAction;

    fn set_action(&mut self, action: Self::Action) {
        debug_assert!(
            self.e.action.is_none(),
            "Uh-oh, tried to set to {action} but it was already set to {:?}",
            self.e.action
        );
        self.e.action = Some(action);
    }

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
        self.sequencer.work(&mut handler);
        self.control_trips
            .values_mut()
            .for_each(|ct| ct.work(&mut handler));
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
impl Displays for Track {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{
        factory::test_entities::{TestEffect, TestInstrument, TestInstrumentCountsMidiMessages},
        prelude::ToyControllerAlwaysSendsMidiMessage,
    };
    use ensnare_proc_macros::{Control, IsController, Metadata};

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

    #[derive(Default, Debug, Control, IsController, Metadata, Serialize, Deserialize)]
    struct TimelineDisplayer {
        uid: Uid,
    }
    impl Serializable for TimelineDisplayer {}
    impl Controls for TimelineDisplayer {}
    impl Configurable for TimelineDisplayer {}
    impl HandlesMidi for TimelineDisplayer {}
    impl Displays for TimelineDisplayer {}
}
