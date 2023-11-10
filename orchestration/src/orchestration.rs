// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    bus_route::{BusRoute, BusStation},
    humidifier::Humidifier,
    main_mixer::MainMixer,
    midi_router::MidiRouter,
    track::{track_widget, Track, TrackAction, TrackBuffer, TrackWidgetAction},
    traits::Orchestrates,
};
use anyhow::anyhow;
use crossbeam_channel::Sender;
use derive_builder::Builder;
use ensnare_core::{
    control::{ControlIndex, ControlValue},
    controllers::ControlTripBuilder,
    midi::{MidiChannel, MidiMessage},
    piano_roll::{PatternUid, PianoRoll},
    selection_set::SelectionSet,
    time::{MusicalTime, SampleRate, Tempo, TimeSignature, Transport, TransportBuilder, ViewRange},
    traits::{
        Configurable, ControlEventsFn, Controllable, Controls, EntityEvent, Generates,
        GeneratesToInternalBuffer, HandlesMidi, HasMetadata, MidiMessagesFn, Serializable, Ticks,
    },
    types::{AudioQueue, Normal, Sample, StereoSample, TrackTitle},
    uid::{EntityUidFactory, TrackUid, TrackUidFactory, Uid},
};
use ensnare_cores::LivePatternSequencer;
use ensnare_egui::{
    control::ControlRouter,
    piano_roll::piano_roll,
    widgets::{
        timeline::{self, TimelineIconStripAction},
        track,
    },
};
use ensnare_entity::prelude::*;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use std::{
    collections::HashMap,
    fmt::Debug,
    ops::Range,
    path::PathBuf,
    sync::{Arc, RwLock},
    vec::Vec,
};
use strum_macros::Display;

/// Actions that [Orchestrator]'s UI might need the parent to perform.
#[derive(Clone, Debug, Display)]
pub enum OrchestratorAction {
    /// A [Track] was clicked in the UI.
    ClickTrack(TrackUid),
    /// A [Track] was double-clicked in the UI.
    DoubleClickTrack(TrackUid),
    /// A [Track] wants a new device of type [Key].
    NewDeviceForTrack(TrackUid, EntityKey),
}
impl IsAction for OrchestratorAction {}

/// A grouping mechanism to declare parts of [Orchestrator] that Serde shouldn't
/// be serializing. Exists so we don't have to spray #[serde(skip)] all over the
/// place.
#[derive(Debug, Default)]
pub struct OrchestratorEphemerals {
    range: ViewRange,
    events: Vec<(Uid, EntityEvent)>,
    is_finished: bool,
    is_performing: bool,
    pub action: Option<OrchestratorAction>,
    pub track_selection_set: SelectionSet<TrackUid>,
    pub sample_buffer_channel_sender: Option<Sender<[Sample; 64]>>,
    //    pub keyboard_controller: KeyboardController,
    pub is_piano_roll_open: bool, // TODO whether this should be serialized
    pub is_entity_detail_open: bool,
    pub entity_detail_title: String,
    pub selected_entity_uid: Option<Uid>,
}

/// Owns all entities (instruments, controllers, and effects), and manages the
/// relationships among them to create an audio performance.
#[derive(Debug, Builder)]
#[builder(setter(skip), default)]
#[builder_struct_attr(allow(missing_docs))]
#[deprecated]
pub struct OldOrchestrator {
    /// The user-supplied name of this project.
    #[builder(setter, default)]
    pub title: Option<String>,

    entity_uid_factory: EntityUidFactory,
    track_uid_factory: TrackUidFactory,

    pub transport: Transport,
    control_router: ControlRouter,

    /// An ordered list of [TrackUid]s in the order they appear in the UI.
    pub track_uids: Vec<TrackUid>,

    /// All [Track]s, indexed by their [TrackUid].
    pub tracks: HashMap<TrackUid, Track>,

    // This is the owned and serialized instance of PianoRoll. Because we're
    // using Arc<> in a struct that Serde serializes, we need to have the `rc`
    // feature enabled for Serde.
    pub piano_roll: Arc<RwLock<PianoRoll>>,

    bus_station: BusStation,

    // A cache for finding an [Entity]'s owning [Track].
    pub entity_uid_to_track_uid: HashMap<Uid, TrackUid>,

    // We do want this serialized, unlike many other entities that implement
    // DisplaysInTimeline, because this field determines the timeline view
    // range, and it's nicer to remember it when the project is loaded and
    // saved.
    pub view_range: ViewRange,

    main_mixer: MainMixer,

    //////////////////////////////////////////////////////
    // Nothing below this comment should be serialized. //
    //////////////////////////////////////////////////////
    //
    pub e: OrchestratorEphemerals,
}
impl Default for OldOrchestrator {
    fn default() -> Self {
        let transport = TransportBuilder::default().build().unwrap();
        let view_range =
            MusicalTime::START..MusicalTime::new_with_bars(&transport.time_signature(), 4);
        let piano_roll = Arc::new(RwLock::new(PianoRoll::default()));
        Self {
            title: Default::default(),
            entity_uid_factory: Default::default(),
            track_uid_factory: Default::default(),
            transport,
            control_router: Default::default(),
            tracks: Default::default(),
            track_uids: Default::default(),
            piano_roll,
            bus_station: Default::default(),
            entity_uid_to_track_uid: Default::default(),
            view_range,
            main_mixer: Default::default(),

            e: Default::default(),
        }
    }
}
impl OldOrchestrator {
    /// The expected size of any buffer provided for samples.
    //
    // TODO: how hard would it be to make this dynamic? Does adjustability
    // matter?
    pub const SAMPLE_BUFFER_SIZE: usize = 64;

    /// The fixed [Uid] for this Orchestrator.
    pub const ORCHESTRATOR_UID: Uid = Uid(1);

    /// The fixed [Uid] for the global transport.
    pub const TRANSPORT_UID: Uid = Uid(2);

    pub const ENTITY_NAME: &'static str = "Orchestrator";
    pub const ENTITY_KEY: &'static str = "orchestrator";

    /// Adds the pattern with the given [PatternUid] (in [PianoRoll]) at the
    /// specified position to the given track's sequencer.
    pub fn add_pattern_to_track(
        &mut self,
        track_uid: &TrackUid,
        pattern_uid: &PatternUid,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        if let Some(track) = self.tracks.get_mut(track_uid) {
            track.add_pattern(pattern_uid, position)
        } else {
            Err(anyhow!("Couldn't find track {track_uid}"))
        }
    }

    pub fn mint_entity_uid(&self) -> Uid {
        self.entity_uid_factory.mint_next()
    }

    fn mint_track_uid(&mut self) -> TrackUid {
        self.track_uid_factory.mint_next()
    }

    fn new_base_track(
        &mut self,
        post_creation: impl FnOnce(TrackUid, &mut Track),
    ) -> anyhow::Result<TrackUid> {
        let track_uid = self.create_track()?;
        if let Some(track) = self.tracks.get_mut(&track_uid) {
            post_creation(track_uid, track);
            Ok(track_uid)
        } else {
            Err(anyhow!("Couldn't find just-created track {track_uid}"))
        }
    }

    /// Adds a new MIDI track, which can contain controllers, instruments, and
    /// effects. Returns the new track's [TrackUid] if successful.
    pub fn new_midi_track(&mut self) -> anyhow::Result<TrackUid> {
        let sequencer = LivePatternSequencer::new_with(Arc::clone(&self.piano_roll));
        let trip_uid = self.mint_entity_uid();
        self.new_base_track(|track_uid, track| {
            track.title = TrackTitle(format!("MIDI {}", track_uid));
            track.set_sequencer(sequencer);
            track.control_trips.insert(
                trip_uid,
                ControlTripBuilder::default()
                    .random(MusicalTime::START)
                    .build()
                    .unwrap(),
            );
        })
    }

    /// Adds a new audio track, which can contain audio clips and effects.
    /// Returns the new track's [TrackUid] if successful.
    pub fn new_audio_track(&mut self) -> anyhow::Result<TrackUid> {
        self.new_base_track(|track_uid, track| {
            track.title = TrackTitle(format!("Audio {}", track_uid));
        })
    }

    /// Adds a new aux track, which contains only effects, and to which other
    /// tracks can *send* their output audio. Returns the new track's [TrackUid]
    /// if successful.
    pub fn new_aux_track(&mut self) -> anyhow::Result<TrackUid> {
        self.new_base_track(|track_uid, track| {
            track.title = TrackTitle(format!("Aux {}", track_uid));
        })
    }

    /// Adds a set of tracks that make sense for a new project.
    pub fn create_starter_tracks(&mut self) -> anyhow::Result<()> {
        if !self.track_uids.is_empty() {
            return Err(anyhow!("Must be invoked on an empty orchestrator."));
        }
        self.new_midi_track()?;
        self.new_midi_track()?;
        self.new_audio_track()?;
        self.new_aux_track()?;
        Ok(())
    }

    /// Sets a new title for the track.
    pub fn set_track_title(&mut self, uid: TrackUid, title: TrackTitle) {
        if let Some(track) = self.tracks.get_mut(&uid) {
            track.set_title(title);
        }
    }

    /// Renders the next set of samples into the provided buffer. This is the
    /// main event loop.
    pub fn render(
        &mut self,
        samples: &mut [StereoSample],
        control_events_fn: &mut ControlEventsFn,
    ) {
        // Note that advance() can return the same range twice, depending on
        // sample rate. TODO: we should decide whose responsibility it is to
        // handle that -- either we skip calling work() if the time range is the
        // same as prior, or everyone who gets called needs to detect the case
        // or be idempotent.
        let range = self.transport.advance(samples.len());
        self.update_time(&range);
        self.work(control_events_fn);
        self.generate_batch_values(samples);
    }

    /// A convenience method for callers who would have ignored any
    /// [EntityEvent]s produced by the render() method.
    pub fn render_and_ignore_events(&mut self, samples: &mut [StereoSample]) {
        self.render(samples, &mut |_, _| {});
    }

    /// Renders part of the project to audio, creating at least the requested
    /// number of [StereoSample]s and inserting them in the given [AudioQueue].
    /// Exceptions: the method operates only in [Self::SAMPLE_BUFFER_SIZE]
    /// chunks, and it won't generate a chunk unless there is enough room in the
    /// queue for it.
    ///
    /// This method expects to be called continuously, even when the project
    /// isn't actively playing. In such cases, it will provide a stream of
    /// silent samples.
    //
    // TODO: I don't think there's any reason why this must be limited to an
    // `AudioQueue` rather than a more general `Vec`-like interface.
    pub fn render_and_enqueue(
        &mut self,
        samples_requested: usize,
        queue: &AudioQueue,
        control_events_fn: &mut ControlEventsFn,
    ) {
        // Round up
        let buffers_requested =
            (samples_requested + Self::SAMPLE_BUFFER_SIZE - 1) / Self::SAMPLE_BUFFER_SIZE;
        for _ in 0..buffers_requested {
            // Generate a buffer only if there's enough room in the queue for
            // it.
            if queue.capacity() - queue.len() >= Self::SAMPLE_BUFFER_SIZE {
                let mut samples = [StereoSample::SILENCE; Self::SAMPLE_BUFFER_SIZE];
                if false {
                    self.render_debug(&mut samples);
                } else {
                    self.render(&mut samples, control_events_fn);
                }
                // No need to do the Arc deref each time through the loop. TODO:
                // is there a queue type that allows pushing a batch?
                let queue = queue.as_ref();
                let mut mono_samples = [Sample::SILENCE; Self::SAMPLE_BUFFER_SIZE];
                for (index, sample) in samples.into_iter().enumerate() {
                    let _ = queue.push(sample);
                    mono_samples[index] = Sample::from(sample);
                }

                // TODO: can we do this work outside this critical loop? And can
                // we have the recipient do the work of the stereo->mono
                // conversion?
                if let Some(sender) = &self.e.sample_buffer_channel_sender {
                    let _ = sender.send(mono_samples);
                }
            }
        }
    }

    /// Fills in the given sample buffer with something simple and audible.
    pub fn render_debug(&mut self, samples: &mut [StereoSample]) {
        let len = samples.len() as f64;
        for (i, s) in samples.iter_mut().enumerate() {
            s.0 = Sample::from(i as f64 / len);
            s.1 = Sample::from(i as f64 / -len);
        }
    }

    /// After loading a new Self from disk, we want to copy all the appropriate
    /// ephemeral state from this one to the next one.
    pub fn prepare_successor(&self, new: &mut OldOrchestrator) {
        // Copy over the current sample rate, whose validity shouldn't change
        // because we loaded a new project.
        new.update_sample_rate(self.sample_rate());
    }

    /// Returns all [Track] uids in UI order.
    pub fn track_uids(&self) -> &[TrackUid] {
        self.track_uids.as_ref()
    }

    fn calculate_is_finished(&self) -> bool {
        self.tracks.values().all(|t| t.is_finished())
    }

    // This method is called only for events generated internally (i.e., from
    // our own Entities). It is not called for external MIDI messages.
    fn dispatch_event(&mut self, uid: Uid, event: EntityEvent) {
        match event {
            EntityEvent::Midi(..) => {
                panic!("FATAL: we were asked to dispatch an EntityEvent::Midi, which should already have been handled")
            }
            EntityEvent::Control(value) => {
                self.route_control_change(uid, value);
            }
        }
    }

    fn route_midi_message(&mut self, channel: MidiChannel, message: MidiMessage) {
        for t in self.tracks.values_mut() {
            t.route_midi_message(channel, message);
        }
    }

    fn route_control_change(&mut self, source_uid: Uid, value: ControlValue) {
        let _ = self.control_router.route(
            &mut |target_uid, index, value| {
                if target_uid == &Self::TRANSPORT_UID {
                    self.transport.control_set_param_by_index(index, value);
                }
            },
            source_uid,
            value,
        );
        for t in self.tracks.values_mut() {
            t.route_control_change(source_uid, value);
        }
    }

    #[allow(missing_docs)]
    pub fn set_track_selection_set(&mut self, track_selection_set: SelectionSet<TrackUid>) {
        self.e.track_selection_set = track_selection_set;
    }

    pub fn handle_track_action(&mut self, uid: TrackUid, action: TrackAction) {
        match action {
            TrackAction::NewDevice(key) => {
                self.e.action = Some(OrchestratorAction::NewDeviceForTrack(uid, key))
            }
            TrackAction::LinkControl(source_uid, target_uid, control_index) => {
                if let Some(track) = self.tracks.get_mut(&uid) {
                    track
                        .control_router
                        .link_control(source_uid, target_uid, control_index);
                }
            }
            TrackAction::EntitySelected(uid) => self.e.selected_entity_uid = Some(uid),
        }
    }

    fn check_keyboard(&mut self) {
        let mut keyboard_events = Vec::default();

        // self.e.keyboard_controller.work(&mut |_, m| {
        //     if let EntityEvent::Midi(channel, message) = m {
        //         keyboard_events.push((channel, message));
        //     }
        // });
        for (channel, message) in keyboard_events.into_iter() {
            self.handle_midi_message(channel, message, &mut |_, _| {})
        }
    }
}
impl Orchestrates for OldOrchestrator {
    fn create_track(&mut self) -> anyhow::Result<TrackUid> {
        let track_uid = self.mint_track_uid();
        self.track_uids.push(track_uid);
        let mut track = Track::default();
        track.e.piano_roll = Arc::clone(&self.piano_roll);
        self.tracks.insert(track_uid, track);
        Ok(track_uid)
    }

    fn track_uids(&self) -> &[TrackUid] {
        &self.track_uids
    }

    fn set_track_position(
        &mut self,
        track_uid: TrackUid,
        new_position: usize,
    ) -> anyhow::Result<()> {
        if self.track_uids.contains(&track_uid) {
            self.track_uids.retain(|uid| *uid != track_uid);
            if new_position <= self.track_uids.len() {
                self.track_uids.insert(new_position, track_uid);
                Ok(())
            } else {
                Err(anyhow!(
                    "Given position {new_position} is beyond range 0..{}",
                    self.track_uids.len(),
                ))
            }
        } else {
            Err(anyhow!("Track {track_uid} not found"))
        }
    }

    fn delete_track(&mut self, track_uid: &TrackUid) {
        self.tracks.remove(&track_uid);
        self.track_uids.retain(|uid| uid != track_uid);
    }

    fn delete_tracks(&mut self, uids: &[TrackUid]) {
        uids.iter().for_each(|uid| {
            self.delete_track(uid);
        });
    }

    fn add_entity(&mut self, track_uid: &TrackUid, entity: Box<dyn Entity>) -> anyhow::Result<()> {
        if entity.uid() == Uid::default() {
            panic!("Attempted to add an entity without a valid Uid. Did you mean to call assign_uid_and_add_entity() instead?");
        }
        if let Some(track) = self.tracks.get_mut(&track_uid) {
            let uid = entity.uid();
            self.entity_uid_to_track_uid.insert(uid, *track_uid);
            track.append_entity(entity, uid)?;
            Ok(())
        } else {
            Err(anyhow!("Couldn't find track {track_uid}"))
        }
    }

    fn assign_uid_and_add_entity(
        &mut self,
        track_uid: &TrackUid,
        mut entity: Box<dyn Entity>,
    ) -> anyhow::Result<Uid> {
        if entity.uid() != Uid::default() {
            panic!(
                "Attempted to assign Uid to entity that already had one ({})",
                entity.uid()
            );
        }
        let uid = self.mint_entity_uid();
        entity.set_uid(uid);
        if let Some(track) = self.tracks.get_mut(&track_uid) {
            self.entity_uid_to_track_uid.insert(uid, *track_uid);
            track.append_entity(entity, uid)?;
            Ok(uid)
        } else {
            Err(anyhow!("Couldn't find track {track_uid}"))
        }
    }

    fn set_entity_track(&mut self, track_uid: &TrackUid, uid: &Uid) -> anyhow::Result<()> {
        if let Ok(entity) = self.remove_entity(uid) {
            match self.add_entity(track_uid, entity) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(anyhow!("Couldn't find track {track_uid}"))
        }
    }

    fn remove_entity(&mut self, uid: &Uid) -> anyhow::Result<Box<dyn Entity>> {
        if let Some(track_uid) = self.entity_uid_to_track_uid.get(&uid) {
            if let Some(track) = self.tracks.get_mut(track_uid) {
                if let Some(entity) = track.remove_entity(uid) {
                    Ok(entity)
                } else {
                    Err(anyhow!(
                        "Couldn't remove entity {uid} from track {track_uid}"
                    ))
                }
            } else {
                Err(anyhow!("Track {track_uid} not found"))
            }
        } else {
            Err(anyhow!("Track location of entity {uid} not found"))
        }
    }

    fn link_control(
        &mut self,
        source_uid: Uid,
        target_uid: Uid,
        control_index: ControlIndex,
    ) -> anyhow::Result<()> {
        if target_uid == Self::TRANSPORT_UID {
            self.control_router
                .link_control(source_uid, target_uid, control_index);
            Ok(())
        } else {
            if let Some(track_uid) = self.entity_uid_to_track_uid.get(&target_uid) {
                if let Some(track) = self.tracks.get_mut(track_uid) {
                    track
                        .control_router
                        .link_control(source_uid, target_uid, control_index);
                    Ok(())
                } else {
                    Err(anyhow!("Couldn't find track that owns entity {target_uid}"))
                }
            } else {
                Err(anyhow!(
                    "Couldn't find uid of track that owns entity {target_uid}"
                ))
            }
        }
    }

    fn unlink_control(&mut self, source_uid: Uid, target_uid: Uid, control_index: ControlIndex) {
        if target_uid == Self::TRANSPORT_UID {
            self.control_router
                .unlink_control(source_uid, target_uid, control_index);
        } else {
            if let Some(track_uid) = self.entity_uid_to_track_uid.get(&target_uid) {
                if let Some(track) = self.tracks.get_mut(track_uid) {
                    track
                        .control_router
                        .unlink_control(source_uid, target_uid, control_index);
                }
            }
        }
    }

    fn set_effect_humidity(&mut self, uid: Uid, humidity: Normal) -> anyhow::Result<()> {
        if let Some(track_uid) = self.entity_uid_to_track_uid.get(&uid) {
            if let Some(track) = self.tracks.get_mut(track_uid) {
                track.set_humidity(uid, humidity)
            } else {
                Err(anyhow!("Couldn't find track that owns entity {uid}"))
            }
        } else {
            Err(anyhow!("Couldn't find uid of track that owns entity {uid}"))
        }
    }

    fn set_effect_position(&mut self, uid: Uid, index: usize) -> anyhow::Result<()> {
        if let Some(track_uid) = self.entity_uid_to_track_uid.get(&uid) {
            if let Some(track) = self.tracks.get_mut(track_uid) {
                track.move_effect(uid, index)
            } else {
                Err(anyhow!("Couldn't find track that owns entity {uid}"))
            }
        } else {
            Err(anyhow!("Couldn't find uid of track that owns entity {uid}"))
        }
    }

    fn send(
        &mut self,
        send_track_uid: TrackUid,
        aux_track_uid: TrackUid,
        send_amount: Normal,
    ) -> anyhow::Result<()> {
        self.bus_station.add_send_route(
            send_track_uid,
            BusRoute {
                aux_track_uid,
                amount: send_amount,
            },
        )
    }

    fn remove_send(&mut self, send_track_uid: TrackUid, aux_track_uid: TrackUid) {
        self.bus_station
            .remove_send_route(&send_track_uid, &aux_track_uid);
    }

    fn set_track_output(&mut self, track_uid: TrackUid, output: Normal) {
        self.main_mixer.set_track_output(track_uid, output)
    }

    fn mute_track(&mut self, track_uid: TrackUid, muted: bool) {
        self.main_mixer.mute_track(track_uid, muted)
    }

    fn solo_track(&self) -> Option<TrackUid> {
        self.main_mixer.solo_track()
    }

    fn set_solo_track(&mut self, track_uid: TrackUid) {
        self.main_mixer.set_solo_track(track_uid)
    }

    fn end_solo(&mut self) {
        self.main_mixer.end_solo()
    }

    fn next_range(&mut self, frame_count: usize) -> std::ops::Range<MusicalTime> {
        self.transport.advance(frame_count)
    }

    fn connect_midi_receiver(&mut self, uid: Uid, channel: MidiChannel) -> anyhow::Result<()> {
        if let Some(track_uid) = self.entity_uid_to_track_uid.get(&uid) {
            if let Some(track) = self.tracks.get_mut(track_uid) {
                track.midi_router.connect(uid, channel);
            }
        }
        Ok(())
    }

    fn disconnect_midi_receiver(&mut self, uid: Uid, channel: MidiChannel) {
        if let Some(track_uid) = self.entity_uid_to_track_uid.get(&uid) {
            if let Some(track) = self.tracks.get_mut(track_uid) {
                track.midi_router.disconnect(uid, channel);
            }
        }
    }
}
impl Displays for OldOrchestrator {}
impl Acts for OldOrchestrator {
    type Action = OrchestratorAction;

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
impl HasMetadata for OldOrchestrator {
    fn uid(&self) -> Uid {
        Self::ORCHESTRATOR_UID
    }
    fn set_uid(&mut self, _: Uid) {
        panic!("Orchestrator's UID is reserved and should never change.")
    }
    fn name(&self) -> &'static str {
        Self::ENTITY_NAME
    }
    fn key(&self) -> &'static str {
        Self::ENTITY_KEY
    }
}
impl Generates<StereoSample> for OldOrchestrator {
    fn value(&self) -> StereoSample {
        StereoSample::SILENCE
    }

    // Note! It's the caller's job to prepare the buffer. This method will *add*
    // its results, rather than overwriting.
    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        let len = values.len();

        // Generate all normal tracks in parallel.
        //
        // TODO: I couldn't figure out how to filter() and then spawn the
        // parallel iteration on the results, so it looks like we're wasting
        // time spinning up threads on tracks that we know have no work to do.
        self.tracks.par_iter_mut().for_each(|(_, track)| {
            track.buffer_mut().0.fill(StereoSample::SILENCE);
            if track.instruments.is_empty() {
                // This looks like an aux track. We won't call it yet.
            } else {
                // This looks like a non-aux track. It's time for it to generate
                // its signal.
                track.generate_batch_values(len);
            }
        });

        // Send audio to aux tracks...
        for (track_uid, routes) in self.bus_station.send_routes() {
            // We need an extra buffer copy to satisfy the borrow checker.
            // HashMap::get_mut() grabs the entire HashMap, preventing us from
            // holding references to other elements in it. There are other
            // implementations of HashMap that allow get_many_mut(), which could
            // help. TODO
            let mut send_buffer = TrackBuffer::default();
            if let Some(send) = self.tracks.get(track_uid) {
                send_buffer.0.copy_from_slice(&send.buffer().0);
            } else {
                eprintln!("Warning: couldn't find send track {track_uid}");
                continue;
            }

            for route in routes {
                if let Some(aux) = self.tracks.get_mut(&route.aux_track_uid) {
                    let aux_buffer = aux.buffer_mut();
                    for (index, sample) in send_buffer.0.iter().enumerate() {
                        aux_buffer.0[index] += *sample * route.amount
                    }
                }
            }
        }

        // ... and then generate the aux tracks...
        //
        // We don't currently support an aux returning to another aux. It's just
        // regular tracks sending to aux, then aux returning to main. See #143
        self.tracks.par_iter_mut().for_each(|(_, track)| {
            if track.instruments.is_empty() {
                track.generate_batch_values(len);
            }
        });

        // ... and we get returns for free, because (for now) all tracks are
        // connected to the main mixer.

        // TODO: there must be a way to quickly sum same-sized arrays into a
        // final array. https://stackoverflow.com/questions/41207666/ seems to
        // address at least some of it, but I don't think it's any faster, if
        // more idiomatic.
        //
        // TODO even more: hmmmmmm, maybe I can use
        // https://doc.rust-lang.org/std/cell/struct.Cell.html so that we can
        // get back to the original Generates model of the caller providing the
        // buffer. And then hmmmm, once we know how things are laid out in
        // memory, maybe we can even sic some fast matrix code on it.
        self.tracks.values().for_each(|track| {
            let generator_values = track.values();
            let copy_len = len.min(generator_values.len());
            for i in 0..copy_len {
                values[i] += generator_values[i];
            }
        });
    }
}
impl Ticks for OldOrchestrator {
    fn tick(&mut self, _tick_count: usize) {
        panic!()
    }
}
impl Configurable for OldOrchestrator {
    fn sample_rate(&self) -> SampleRate {
        self.transport.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.transport.update_sample_rate(sample_rate);
        for track in self.tracks.values_mut() {
            track.update_sample_rate(sample_rate);
        }
    }

    fn tempo(&self) -> Tempo {
        self.transport.tempo()
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.transport.set_tempo(tempo);
        // TODO: how do we let the service know this changed?
    }

    fn time_signature(&self) -> TimeSignature {
        self.transport.time_signature()
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.transport.update_time_signature(time_signature);
    }
}
impl HandlesMidi for OldOrchestrator {
    /// Accepts a [MidiMessage] and handles it, usually by forwarding it to
    /// controllers and instruments on the given [MidiChannel]. We implement
    /// this trait only for external messages; for ones generated internally, we
    /// use [MidiRouter].
    ///
    /// REPEAT: this method is called only for MIDI messages from EXTERNAL MIDI
    /// INTERFACES!
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        self.route_midi_message(channel, message);
    }
}
impl Controls for OldOrchestrator {
    fn update_time(&mut self, range: &ViewRange) {
        self.e.range = range.clone();

        for track in self.tracks.values_mut() {
            track.update_time(&self.e.range);
        }
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.transport
            .work(&mut |_, m| self.e.events.push((Self::TRANSPORT_UID, m)));
        self.check_keyboard();

        for track in self.tracks.values_mut() {
            // By this point in the control_events_fn chain, `u` must be
            // correctly assigned, which is why we know we can safely .unwrap()
            // it. That's because each Track should know the Uid of the Entity
            // that gave it the event, so it should be substituting that Uid
            // into the `u` argument.
            track.work(&mut |u, m| self.e.events.push((u.unwrap(), m)));
        }
        while let Some((uid, event)) = self.e.events.pop() {
            if matches!(event, EntityEvent::Midi(_, _)) {
                // This MIDI message came from one of our internal Entities and
                // has bubbled all the way up here. We don't want to do anything
                // with it, and should instead pass it along to the caller, who
                // will forward it to external MIDI interfaces.
                //
                // This MIDI message came from a Track. The Track's
                // responsibility was to route the message to all the eligible
                // Entities that it owned. We don't want to route these messages
                // back to any Tracks; our only responsibility is to send them
                // to external MIDI interfaces.
                //
                // Eventually, we might allow one Track to send MIDI messages to
                // another Track. But today we don't. TODO?
                control_events_fn(None, event);
            } else {
                self.dispatch_event(uid, event);
            }
        }
        self.e.is_finished = self.calculate_is_finished();
        if self.is_performing() && self.is_finished() {
            self.stop();
        }
    }

    fn is_finished(&self) -> bool {
        self.e.is_finished
    }

    fn play(&mut self) {
        self.e.is_performing = true;
        self.transport.play();
        self.tracks.values_mut().for_each(|t| t.play());
        self.e.is_finished = self.calculate_is_finished();

        // This handles the case where there isn't anything to play because the
        // performance is zero-length. It stops the transport from advancing a
        // tiny bit and looking weird.
        if self.e.is_finished {
            self.transport.stop();
        }
    }

    fn stop(&mut self) {
        // If we were performing, stop. Otherwise, it's a stop-while-stopped
        // action, which means the user wants to rewind to the beginning.
        if self.e.is_performing {
            self.e.is_performing = false;
        } else {
            self.skip_to_start();
        }
        self.transport.stop();
        self.tracks.values_mut().for_each(|t| t.stop());
    }

    fn skip_to_start(&mut self) {
        self.transport.skip_to_start();
        self.tracks.values_mut().for_each(|t| t.skip_to_start());
    }

    fn is_performing(&self) -> bool {
        self.e.is_performing
    }
}
impl Serializable for OldOrchestrator {
    fn after_deser(&mut self) {
        self.tracks.values_mut().for_each(|t| {
            t.e.piano_roll = Arc::clone(&self.piano_roll);
            t.after_deser();
        });
    }
}

/// [NewOrchestrator] is a back-to-basics implementation that satisfies
/// [Orchestrates]. It takes [Entities](Entity) and invokes them appropriately
/// to produce an audio performance.
#[derive(Debug, Default)]
pub struct Orchestrator {
    entity_uid_factory: EntityUidFactory,
    track_uid_factory: TrackUidFactory,

    pub transport: Transport,

    track_uids: Vec<TrackUid>,
    track_for_entity: HashMap<Uid, TrackUid>,

    entity_store: EntityStore,
    controller_uids: HashMap<TrackUid, Vec<Uid>>,
    instrument_uids: HashMap<TrackUid, Vec<Uid>>,
    effect_uids: HashMap<TrackUid, Vec<Uid>>,

    control_router: ControlRouter,
    midi_router: MidiRouter,
    humidifier: Humidifier,
    bus_station: BusStation,
    main_mixer: MainMixer,
}
impl Orchestrator {
    /// The fixed [Uid] for the global transport.
    pub(crate) const TRANSPORT_UID: Uid = Uid(2);

    pub fn mint_entity_uid(&self) -> Uid {
        self.entity_uid_factory.mint_next()
    }

    fn mint_track_uid(&mut self) -> TrackUid {
        self.track_uid_factory.mint_next()
    }

    //////////////////////////////// RECONSIDER
    /// Adds the pattern with the given [PatternUid] (in [PianoRoll]) at the
    /// specified position to the given track's sequencer.
    pub fn add_pattern_to_track(
        &mut self,
        track_uid: &TrackUid,
        pattern_uid: &PatternUid,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        // if let Some(track) = self.tracks.get_mut(track_uid) {
        //     track.add_pattern(pattern_uid, position)
        // } else {
        Err(anyhow!("Couldn't find track {track_uid}"))
        // }
    }

    /// Adds a new MIDI track, which can contain controllers, instruments, and
    /// effects. Returns the new track's [TrackUid] if successful.
    pub fn new_midi_track(&mut self) -> anyhow::Result<TrackUid> {
        self.create_track()
    }

    /// Adds a new audio track, which can contain audio clips and effects.
    /// Returns the new track's [TrackUid] if successful.
    pub fn new_audio_track(&mut self) -> anyhow::Result<TrackUid> {
        self.create_track()
    }

    /// Adds a new aux track, which contains only effects, and to which other
    /// tracks can *send* their output audio. Returns the new track's [TrackUid]
    /// if successful.
    pub fn new_aux_track(&mut self) -> anyhow::Result<TrackUid> {
        self.create_track()
    }

    /// Adds a set of tracks that make sense for a new project.
    pub fn create_starter_tracks(&mut self) -> anyhow::Result<()> {
        if !self.track_uids.is_empty() {
            return Err(anyhow!("Must be invoked on an empty orchestrator."));
        }
        self.new_midi_track()?;
        self.new_midi_track()?;
        self.new_audio_track()?;
        self.new_aux_track()?;
        Ok(())
    }
    //////////////////////////////// RECONSIDER
}
impl Orchestrates for Orchestrator {
    fn create_track(&mut self) -> anyhow::Result<TrackUid> {
        let track_uid = self.mint_track_uid();
        self.track_uids.push(track_uid);
        Ok(track_uid)
    }

    fn track_uids(&self) -> &[TrackUid] {
        &self.track_uids
    }

    fn set_track_position(
        &mut self,
        track_uid: TrackUid,
        new_position: usize,
    ) -> anyhow::Result<()> {
        if !self.track_uids.contains(&track_uid) {
            return Err(anyhow!("Track Uid {track_uid} not found"));
        }
        self.track_uids.retain(|t| *t != track_uid);
        self.track_uids.insert(new_position, track_uid);
        Ok(())
    }

    fn delete_track(&mut self, track_uid: &TrackUid) {
        self.track_uids.retain(|t| *t != *track_uid);
        self.track_for_entity.retain(|_, v| *v != *track_uid);
    }

    fn delete_tracks(&mut self, uids: &[TrackUid]) {
        // TODO something something O(n^2) something
        self.track_uids.retain(|t| !uids.contains(t));
        self.track_for_entity.retain(|_, v| !uids.contains(v));
    }

    fn add_entity(&mut self, track_uid: &TrackUid, entity: Box<dyn Entity>) -> anyhow::Result<()> {
        let uid = entity.uid();
        if uid.0 == 0 {
            return Err(anyhow!("Entity has invalid Uid {}", uid));
        }
        self.track_for_entity.insert(uid, *track_uid);
        if entity.as_controller().is_some() {
            self.controller_uids
                .entry(*track_uid)
                .or_default()
                .push(uid);
        }
        if entity.as_instrument().is_some() {
            self.instrument_uids
                .entry(*track_uid)
                .or_default()
                .push(uid);
        }
        if entity.as_effect().is_some() {
            self.effect_uids.entry(*track_uid).or_default().push(uid);
        }
        self.entity_store.add(entity, uid)
    }

    fn assign_uid_and_add_entity(
        &mut self,
        track_uid: &TrackUid,
        mut entity: Box<dyn Entity>,
    ) -> anyhow::Result<Uid> {
        let uid = self.entity_uid_factory.mint_next();
        entity.set_uid(uid);
        self.add_entity(track_uid, entity)?;
        Ok(uid)
    }

    fn remove_entity(&mut self, uid: &Uid) -> anyhow::Result<Box<dyn Entity>> {
        if let Some(entity) = self.entity_store.remove(uid) {
            if let Some(track_uid) = self.track_for_entity.get(uid) {
                self.controller_uids
                    .entry(*track_uid)
                    .or_default()
                    .retain(|uid| *uid != entity.uid());
                self.instrument_uids
                    .entry(*track_uid)
                    .or_default()
                    .retain(|uid| *uid != entity.uid());
                self.effect_uids
                    .entry(*track_uid)
                    .or_default()
                    .retain(|uid| *uid != entity.uid());
            }
            Ok(entity)
        } else {
            Err(anyhow!("Entity Uid {uid} not found"))
        }
    }

    fn set_entity_track(&mut self, new_track_uid: &TrackUid, uid: &Uid) -> anyhow::Result<()> {
        if self.track_for_entity.get(uid).is_some() {
            self.track_for_entity.remove(uid);
            self.track_for_entity.insert(*uid, *new_track_uid);
            Ok(())
        } else {
            Err(anyhow!("Entity Uid {uid} not found"))
        }
    }

    fn link_control(
        &mut self,
        source_uid: Uid,
        target_uid: Uid,
        control_index: ControlIndex,
    ) -> anyhow::Result<()> {
        self.control_router
            .link_control(source_uid, target_uid, control_index);
        Ok(())
    }

    fn unlink_control(&mut self, source_uid: Uid, target_uid: Uid, control_index: ControlIndex) {
        self.control_router
            .unlink_control(source_uid, target_uid, control_index)
    }

    fn set_effect_humidity(&mut self, uid: Uid, humidity: Normal) -> anyhow::Result<()> {
        if let Some(entity) = self.entity_store.get(&uid) {
            if entity.as_effect().is_some() {
                self.humidifier.set_humidity_by_uid(uid, humidity);
                Ok(())
            } else {
                Err(anyhow!("Entity Uid {uid} does not implement as_effect()"))
            }
        } else {
            Err(anyhow!("Entity Uid {uid} not found"))
        }
    }

    fn set_effect_position(&mut self, uid: Uid, new_position: usize) -> anyhow::Result<()> {
        if let Some(track_uid) = self.track_for_entity.get(&uid) {
            let effect_uids = self.effect_uids.entry(*track_uid).or_default();
            if effect_uids.contains(&uid) {
                effect_uids.retain(|e| *e != uid);
                effect_uids.insert(new_position, uid);
                Ok(())
            } else {
                Err(anyhow!("Entity Uid {uid} not found in effects chain"))
            }
        } else {
            Err(anyhow!("Entity Uid {uid} not found"))
        }
    }

    fn send(
        &mut self,
        send_track_uid: TrackUid,
        aux_track_uid: TrackUid,
        send_amount: Normal,
    ) -> anyhow::Result<()> {
        self.bus_station.add_send_route(
            send_track_uid,
            BusRoute {
                aux_track_uid,
                amount: send_amount,
            },
        )
    }

    fn remove_send(&mut self, send_track_uid: TrackUid, aux_track_uid: TrackUid) {
        self.bus_station
            .remove_send_route(&send_track_uid, &aux_track_uid);
    }

    fn set_track_output(&mut self, track_uid: TrackUid, output: Normal) {
        self.main_mixer.set_track_output(track_uid, output)
    }

    fn mute_track(&mut self, track_uid: TrackUid, muted: bool) {
        self.main_mixer.mute_track(track_uid, muted)
    }

    fn solo_track(&self) -> Option<TrackUid> {
        self.main_mixer.solo_track()
    }

    fn set_solo_track(&mut self, track_uid: TrackUid) {
        self.main_mixer.set_solo_track(track_uid)
    }

    fn end_solo(&mut self) {
        self.main_mixer.end_solo()
    }

    fn next_range(&mut self, sample_count: usize) -> std::ops::Range<MusicalTime> {
        // Note that advance() can return the same range twice, depending on
        // sample rate. TODO: we should decide whose responsibility it is to
        // handle that -- either we skip calling work() if the time range is the
        // same as prior, or everyone who gets called needs to detect the case
        // or be idempotent.
        self.transport.advance(sample_count)
    }

    fn connect_midi_receiver(&mut self, uid: Uid, channel: MidiChannel) -> anyhow::Result<()> {
        self.midi_router.connect(uid, channel);
        Ok(())
    }

    fn disconnect_midi_receiver(&mut self, uid: Uid, channel: MidiChannel) {
        self.midi_router.disconnect(uid, channel);
    }
}
impl Configurable for Orchestrator {
    fn sample_rate(&self) -> SampleRate {
        self.transport.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.transport.update_sample_rate(sample_rate);
        self.entity_store.update_sample_rate(sample_rate);
    }

    fn tempo(&self) -> Tempo {
        self.transport.tempo()
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.transport.update_tempo(tempo);
        self.entity_store.update_tempo(tempo);
    }

    fn time_signature(&self) -> TimeSignature {
        self.transport.time_signature()
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.transport.update_time_signature(time_signature);
        self.entity_store.update_time_signature(time_signature);
    }
}
impl Controls for Orchestrator {
    fn update_time(&mut self, range: &ViewRange) {
        // We don't call self.transport.update_time() because self.transport is
        // the publisher of the current time, not a subscriber.
        self.entity_store.update_time(range);
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        let mut events = Vec::default();

        self.transport
            .work(&mut |_, m| events.push((Self::TRANSPORT_UID, m)));

        // By this point in the control_events_fn chain, `u` must be
        // correctly assigned, which is why we know we can safely .unwrap()
        // it. That's because each Track should know the Uid of the Entity
        // that gave it the event, so it should be substituting that Uid
        // into the `u` argument.
        self.entity_store
            .work(&mut |u, m| events.push((u.unwrap(), m)));

        // Dispatch all the events accumulated during work().
        while let Some((uid, event)) = events.pop() {
            match event {
                EntityEvent::Midi(channel, message) => {
                    // Let the caller forward the MIDI message to external interfaces.
                    control_events_fn(None, event);

                    let _ = self
                        .midi_router
                        .route(&mut self.entity_store, channel, message);
                }
                EntityEvent::Control(value) => {
                    let _ = self.control_router.route(
                        &mut |target_uid, index, value| {
                            if target_uid == &Self::TRANSPORT_UID {
                                self.transport.control_set_param_by_index(index, value);
                            } else {
                                if let Some(entity) =
                                    self.entity_store.as_controllable_mut(&target_uid)
                                {
                                    entity.control_set_param_by_index(index, value);
                                }
                            }
                        },
                        uid,
                        value,
                    );
                }
            }
        }

        if self.is_performing() && self.is_finished() {
            self.stop();
        }
    }

    fn is_finished(&self) -> bool {
        self.transport.is_finished() && self.entity_store.is_finished()
    }

    fn play(&mut self) {
        self.transport.play();
        self.entity_store.play();
    }

    fn stop(&mut self) {
        self.transport.stop();
        self.entity_store.stop();
    }

    fn skip_to_start(&mut self) {
        self.transport.skip_to_start();
        self.entity_store.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.transport.is_performing() || self.entity_store.is_performing()
    }
}
impl Ticks for Orchestrator {
    fn tick(&mut self, tick_count: usize) {
        self.entity_store.tick(tick_count);
    }
}
impl Generates<StereoSample> for Orchestrator {
    fn value(&self) -> StereoSample {
        <StereoSample>::default()
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        let solo_track_uid = self.main_mixer.solo_track;

        for track_uid in self.track_uids.iter() {
            // If we're soloing and this isn't the solo track, then skip.
            if let Some(solo_track_uid) = solo_track_uid {
                if *track_uid != solo_track_uid {
                    continue;
                }
            }

            // If this track is muted, skip.
            if self
                .main_mixer
                .track_mute
                .get(track_uid)
                .cloned()
                .unwrap_or_default()
            {
                continue;
            }

            // Start with a silent buffer.
            let mut buffer = Vec::default();
            buffer.resize(values.len(), StereoSample::SILENCE);

            // Play each instrument. They add to the current buffer, so we don't
            // have to worry about mixing.
            if let Some(uids) = self.instrument_uids.get(track_uid) {
                for uid in uids {
                    if let Some(entity) = self.entity_store.as_instrument_mut(uid) {
                        entity.generate_batch_values(&mut buffer);
                    }
                }
            }

            // Apply each effect.
            if let Some(uids) = self.effect_uids.get(track_uid) {
                for uid in uids {
                    let humidity = self.humidifier.get_humidity_by_uid(uid);
                    if humidity == Normal::zero() {
                        continue;
                    }
                    if let Some(entity) = self.entity_store.as_effect_mut(uid) {
                        self.humidifier
                            .transform_batch(humidity, entity, &mut buffer);
                    }
                }
            }

            // Look up the track volume.
            let output = self
                .main_mixer
                .track_output
                .get(track_uid)
                .cloned()
                .unwrap_or_default();
            if output != Normal::zero() {
                for (dst, src) in values.iter_mut().zip(buffer.iter()) {
                    *dst += *src * output;
                }
            }
        }
    }
}
impl HandlesMidi for Orchestrator {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        _midi_messages_fn: &mut MidiMessagesFn,
    ) {
        let _ = self
            .midi_router
            .route(&mut self.entity_store, channel, message);
    }
}

pub struct OrchestratorHelper<'a> {
    orchestrator: &'a mut dyn Orchestrates,
}
impl<'a> OrchestratorHelper<'a> {
    /// The expected size of any buffer provided for samples.
    //
    // TODO: how hard would it be to make this dynamic? Does adjustability
    // matter?
    pub const SAMPLE_BUFFER_SIZE: usize = 64;

    pub fn new_with(orchestrator: &'a mut dyn Orchestrates) -> Self {
        Self { orchestrator }
    }

    /// Returns the number of channels in the audio stream. For now, this is
    /// always 2 (stereo audio stream).
    pub fn channels(&self) -> u16 {
        2
    }

    /// Renders the next set of samples into the provided buffer. This is the
    /// main event loop.
    pub fn render(
        &mut self,
        range: Range<MusicalTime>,
        samples: &mut [StereoSample],
        control_events_fn: &mut ControlEventsFn,
    ) {
        self.orchestrator.update_time(&range);
        self.orchestrator.work(control_events_fn);
        self.orchestrator.generate_batch_values(samples);
    }

    /// A convenience method for callers who would have ignored any
    /// [EntityEvent]s produced by the render() method.
    pub fn render_and_ignore_events(
        &mut self,
        range: Range<MusicalTime>,
        samples: &mut [StereoSample],
    ) {
        self.render(range, samples, &mut |_, _| {});
    }

    /// Renders part of the project to audio, creating at least the requested
    /// number of [StereoSample]s and inserting them in the given [AudioQueue].
    /// Exceptions: the method operates only in [Self::SAMPLE_BUFFER_SIZE]
    /// chunks, and it won't generate a chunk unless there is enough room in the
    /// queue for it.
    ///
    /// This method expects to be called continuously, even when the project
    /// isn't actively playing. In such cases, it will provide a stream of
    /// silent samples.
    //
    // TODO: I don't think there's any reason why this must be limited to an
    // `AudioQueue` rather than a more general `Vec`-like interface.
    pub fn render_and_enqueue(
        &mut self,
        samples_requested: usize,
        queue: &AudioQueue,
        control_events_fn: &mut ControlEventsFn,
    ) {
        // Round up
        let buffers_requested =
            (samples_requested + Self::SAMPLE_BUFFER_SIZE - 1) / Self::SAMPLE_BUFFER_SIZE;
        for _ in 0..buffers_requested {
            // Generate a buffer only if there's enough room in the queue for
            // it.
            if queue.capacity() - queue.len() >= Self::SAMPLE_BUFFER_SIZE {
                let mut samples = [StereoSample::SILENCE; Self::SAMPLE_BUFFER_SIZE];
                if false {
                    self.render_debug(&mut samples);
                } else {
                    let range = self.orchestrator.next_range(samples.len());
                    self.render(range, &mut samples, control_events_fn);
                }
                // No need to do the Arc deref each time through the loop. TODO:
                // is there a queue type that allows pushing a batch?
                let queue = queue.as_ref();
                let mut mono_samples = [Sample::SILENCE; Self::SAMPLE_BUFFER_SIZE];
                for (index, sample) in samples.into_iter().enumerate() {
                    let _ = queue.push(sample);
                    mono_samples[index] = Sample::from(sample);
                }

                // TODO: can we do this work outside this critical loop? And can
                // we have the recipient do the work of the stereo->mono
                // conversion?
                // if let Some(sender) = &self.e.sample_buffer_channel_sender {
                //     let _ = sender.send(mono_samples);
                // }
            }
        }
    }

    /// Fills in the given sample buffer with something simple and audible.
    pub fn render_debug(&mut self, samples: &mut [StereoSample]) {
        let len = samples.len() as f64;
        for (i, s) in samples.iter_mut().enumerate() {
            s.0 = Sample::from(i as f64 / len);
            s.1 = Sample::from(i as f64 / -len);
        }
    }

    // TODO: this could be a feature so that we don't always need the hound
    // dependency. Probably not important either way.
    /// Writes the current performance to a WAV file. Intended for integration
    /// tests only (for now).
    pub fn write_to_file(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        let spec = hound::WavSpec {
            channels: self.channels(),
            sample_rate: self.orchestrator.sample_rate().into(),
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();

        self.orchestrator.play();
        let mut samples = [StereoSample::SILENCE; 64];

        loop {
            if self.orchestrator.is_finished() {
                break;
            }
            samples.fill(StereoSample::SILENCE);
            let range = self.orchestrator.next_range(samples.len());
            self.render_and_ignore_events(range, &mut samples);
            for sample in samples {
                let (left, right) = sample.into_i16();
                let _ = writer.write_sample(left);
                let _ = writer.write_sample(right);
            }
        }

        Ok(())
    }
}

/// Wraps an [OrchestratorEgui] as a [Widget](eframe::egui::Widget).
pub fn orchestrator<'a>(orchestrator: &'a mut Orchestrator) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| OrchestratorEgui::new(orchestrator).ui(ui)
}

/// An egui component that draws an [Orchestrator].
#[derive(Debug)]
struct OrchestratorEgui<'a> {
    orchestrator: &'a mut Orchestrator,
}
impl<'a> Displays for OrchestratorEgui<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(format!(
            "There are {} tracks",
            self.orchestrator.track_uids().len()
        ));
        let add_track_button_response = ui.button("Add Track");
        if add_track_button_response.clicked() {
            let _ = self.orchestrator.create_track();
        }
        add_track_button_response
    }
}
impl<'a> OrchestratorEgui<'a> {
    pub fn new(orchestrator: &'a mut Orchestrator) -> Self {
        Self { orchestrator }
    }
}

/// Wraps an [OldOrchestratorEgui] as a [Widget](eframe::egui::Widget).
pub fn old_orchestrator<'a>(
    orchestrator: &'a mut OldOrchestrator,
    view_range: &'a mut ViewRange,
    is_piano_roll_visible: &'a mut bool,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        OldOrchestratorEgui::new(orchestrator, view_range, is_piano_roll_visible).ui(ui)
    }
}

/// An egui component that draws an [Orchestrator].
#[derive(Debug)]
struct OldOrchestratorEgui<'a> {
    orchestrator: &'a mut OldOrchestrator,
    view_range: &'a mut ViewRange,
    is_piano_roll_visible: &'a mut bool,
}
impl<'a> Displays for OldOrchestratorEgui<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        eframe::egui::Window::new("Piano Roll")
            .open(self.is_piano_roll_visible)
            .default_width(ui.available_width())
            .anchor(
                eframe::emath::Align2::LEFT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                let mut inner = self.orchestrator.piano_roll.write().unwrap();
                let inner_piano_roll: &mut PianoRoll = &mut inner;
                ui.add(piano_roll(inner_piano_roll))
            });

        eframe::egui::Window::new(&self.orchestrator.e.entity_detail_title)
            .id(eframe::egui::Id::from("Entity Detail"))
            .open(&mut self.orchestrator.e.is_entity_detail_open)
            .anchor(
                eframe::emath::Align2::RIGHT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                if let Some(uid) = &self.orchestrator.e.selected_entity_uid {
                    if let Some(track_uid) = self.orchestrator.entity_uid_to_track_uid.get(uid) {
                        if let Some(track) = self.orchestrator.tracks.get_mut(track_uid) {
                            if let Some(entity) = track.entity_mut(uid) {
                                entity.ui(ui);
                            }
                        }
                    }
                }
            });

        eframe::egui::CentralPanel::default()
            .show(ui.ctx(), |ui| {
                let mut action = None;
                ui.add(timeline::timeline_icon_strip(&mut action));
                if let Some(action) = action {
                    match action {
                        TimelineIconStripAction::NextTimelineView => {
                            panic!("get rid of this")
                        }
                        TimelineIconStripAction::ShowPianoRoll => {
                            *self.is_piano_roll_visible = !*self.is_piano_roll_visible;
                        }
                    }
                }

                // The timeline needs to be aligned with the track content, so
                // we create an empty track title bar to match with the real
                // ones.
                ui.horizontal(|ui| {
                    ui.add_enabled(false, track::title_bar(None));
                    ui.add(timeline::legend(self.view_range));
                });

                // Create a scrolling area for all the tracks.
                eframe::egui::ScrollArea::vertical()
                    .id_source("orchestrator-scroller")
                    .show(ui, |ui| {
                        let mut track_action = None;
                        let mut track_action_track_uid = None;
                        for track_uid in self.orchestrator.track_uids.iter() {
                            if let Some(track) = self.orchestrator.tracks.get_mut(track_uid) {
                                let is_selected =
                                    self.orchestrator.e.track_selection_set.contains(track_uid);
                                let cursor = if self.orchestrator.transport.is_performing() {
                                    Some(self.orchestrator.transport.current_time())
                                } else {
                                    None
                                };
                                //                                track.update_font_galley(ui);
                                let mut track_widget_action = None;
                                let response = ui.add(track_widget(
                                    *track_uid,
                                    track,
                                    is_selected,
                                    cursor,
                                    &self.view_range,
                                    &mut track_widget_action,
                                ));
                                if let Some(track_widget_action) = track_widget_action {
                                    match track_widget_action {
                                        TrackWidgetAction::EntitySelected(uid, name) => {
                                            self.orchestrator.e.selected_entity_uid = Some(uid);
                                            self.orchestrator.e.is_entity_detail_open = true;
                                            self.orchestrator.e.entity_detail_title = name;
                                        }
                                    }
                                }
                                if response.double_clicked() {
                                    self.orchestrator.e.action =
                                        Some(OrchestratorAction::DoubleClickTrack(*track_uid));
                                } else if response.clicked() {
                                    self.orchestrator.e.action =
                                        Some(OrchestratorAction::ClickTrack(*track_uid));
                                }

                                if let Some(action) = track.take_action() {
                                    track_action = Some(action);
                                    track_action_track_uid = Some(*track_uid);
                                }
                            }
                        }
                        if let Some(action) = track_action {
                            if let Some(track_uid) = track_action_track_uid {
                                self.orchestrator.handle_track_action(track_uid, action);
                            }
                        }
                    });
            })
            .response
    }
}
impl<'a> OldOrchestratorEgui<'a> {
    pub fn new(
        orchestrator: &'a mut OldOrchestrator,
        view_range: &'a mut ViewRange,
        is_piano_roll_visible: &'a mut bool,
    ) -> Self {
        Self {
            orchestrator,
            view_range,
            is_piano_roll_visible,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::tests::validate_orchestrates_trait;

    use super::*;
    use ensnare_proc_macros::{IsController, Metadata};

    #[test]
    fn basic_operations() {
        let mut o = Orchestrator::default();

        assert!(
            o.sample_rate().0 != 0,
            "Default sample rate should be reasonable"
        );
        let new_sample_rate = SampleRate(3);
        o.update_sample_rate(new_sample_rate);
        assert_eq!(
            o.sample_rate(),
            new_sample_rate,
            "Sample rate should be settable"
        );

        assert!(o.tempo().0 > 0.0, "Default tempo should be reasonable");
        let new_tempo = Tempo(64.0);
        o.update_tempo(new_tempo);
        assert_eq!(o.tempo(), new_tempo, "Tempo should be settable");
    }

    #[cfg(obsolete)]
    #[test]
    fn exposes_traits_ergonomically() {
        let mut o = Orchestrator::default();
        let tuid = o.new_midi_track().unwrap();
        let track = o.tracks.get_mut(&tuid).unwrap();

        const TIMER_DURATION: MusicalTime = MusicalTime::new_with_beats(1);
        let _ = track.append_entity(Box::new(Timer::new_with(TIMER_DURATION)), Uid(459));

        o.play();
        let mut _prior_start_time = MusicalTime::TIME_ZERO;
        loop {
            if o.is_finished() {
                break;
            }
            _prior_start_time = o.transport.current_time();
            let mut samples = [StereoSample::SILENCE; 1];
            o.render_and_ignore_events(&mut samples);
        }

        // TODO: this section is confusing me. It used to say
        // `prior_start_time..o.transport().current_time()`, but that failed
        // just now. I am not sure why it would have ever passed. Consider
        // bisecting to see how it did.
        let prior_range = MusicalTime::TIME_ZERO..o.transport.current_time();
        assert!(
            prior_range.contains(&TIMER_DURATION),
            "Expected the covered range {:?} to include the duration {:?}",
            prior_range,
            TIMER_DURATION
        );
    }

    #[cfg(obsolete)]
    #[test]
    fn starter_tracks() {
        let mut o = Orchestrator::default();
        assert!(o.track_uids.is_empty());
        assert!(o.create_starter_tracks().is_ok());
        assert!(!o.track_uids.is_empty());
        assert!(o.create_starter_tracks().is_err());

        assert_eq!(o.track_uids().len(), 4,
            "we should have two MIDI tracks, one audio track, and one aux track after create_starter_tracks().");
    }

    #[cfg(obsolete)]
    #[test]
    fn track_discovery() {
        let mut o = Orchestrator::default();
        assert!(o.create_starter_tracks().is_ok());
        let track_count = o.track_uids().len();

        // Make sure we can call this and that nothing explodes.
        let mut count = 0;
        o.tracks.values_mut().for_each(|t| {
            t.play();
            count += 1;
        });
        assert_eq!(count, track_count);
    }

    #[cfg(obsolete)]
    #[test]
    fn track_crud() {
        let mut o = Orchestrator::default();
        assert_eq!(o.track_uids().len(), 0);
        let track_uid = o.new_midi_track().unwrap();
        assert_eq!(o.track_uids().len(), 1);

        assert!(o.track_uids()[0] == track_uid);

        o.delete_track(&track_uid);
        assert!(o.track_uids().is_empty());

        // Do it one way
        {
            assert!(o.create_starter_tracks().is_ok());
            assert!(!o.track_uids().is_empty());

            o.delete_tracks(&Vec::from(o.track_uids()));
            assert!(o.track_uids().is_empty());
        }

        // Do it another way
        {
            assert!(o.create_starter_tracks().is_ok());
            assert!(!o.track_uids().is_empty());

            let mut selection_set: HashSet<TrackUid> = HashSet::default();
            for uid in o.track_uids() {
                selection_set.insert(*uid);
            }
            o.delete_tracks(&Vec::from_iter(selection_set.iter().copied()));
            assert!(o.track_uids().is_empty());
        }
    }

    #[test]
    fn zero_length_performance_ends_immediately() {
        let mut o = Orchestrator::default();

        // Controls::is_finished() is undefined before play(), so no fair
        // calling it before play().

        o.play();
        assert!(o.is_finished());
    }

    #[cfg(obsolete)]
    #[test]
    fn sends_send() {
        const EXPECTED_LEVEL: ParameterType = TestAudioSource::MEDIUM;
        let mut o = Orchestrator::default();
        let midi_track_uid = o.new_midi_track().unwrap();
        let aux_track_uid = o.new_aux_track().unwrap();

        {
            let new_uid = o.mint_entity_uid();
            let track = o.tracks.get_mut(&midi_track_uid).unwrap();
            assert!(track
                .append_entity(
                    Box::new(TestAudioSource::new_with(&TestAudioSourceParams {
                        level: EXPECTED_LEVEL,
                    })),
                    new_uid
                )
                .is_ok());
        }
        let mut samples = [StereoSample::SILENCE; TrackBuffer::LEN];
        o.render_and_ignore_events(&mut samples);
        let expected_sample = StereoSample::from(EXPECTED_LEVEL);
        assert!(
            samples.iter().all(|s| *s == expected_sample),
            "Without a send, original signal should pass through unchanged."
        );

        assert!(o
            .send(midi_track_uid, aux_track_uid, Normal::from(0.5))
            .is_ok());
        let mut samples = [StereoSample::SILENCE; TrackBuffer::LEN];
        o.render_and_ignore_events(&mut samples);
        let expected_sample = StereoSample::from(0.75);
        samples.iter().enumerate().for_each(|(index, s)| {
            assert_eq!(*s, expected_sample, "With a 50% send to an aux track with no effects, we should see the original MEDIUM=0.5 plus 50% of it = 0.75, but at sample #{index} we got {:?}", s);
        });

        // Add an effect to the aux track.
        {
            let track = o.tracks.get_mut(&aux_track_uid).unwrap();
            assert!(track
                .append_entity(Box::new(TestEffectNegatesInput::default()), Uid(405))
                .is_ok());
        }
        let mut samples = [StereoSample::SILENCE; TrackBuffer::LEN];
        o.render_and_ignore_events(&mut samples);
        let expected_sample = StereoSample::from(0.5 + 0.5 * 0.5 * -1.0);
        assert!(
            samples.iter().all(|s| *s == expected_sample),
            "With a 50% send to an aux with a negating effect, we should see the original 0.5 plus a negation of 50% of 0.5 = 0.250"
        );
    }

    #[cfg(obsolete)]
    #[test]
    fn midi_routing_from_external_reaches_instruments() {
        let mut o = Orchestrator::default();
        let track_uid = o.new_midi_track().unwrap();

        let track = o.tracks.get_mut(&track_uid).unwrap();
        let instrument = TestInstrumentCountsMidiMessages::default();
        let midi_messages_received = Arc::clone(instrument.received_midi_message_count_mutex());
        let _ = track.append_entity(Box::new(instrument), Uid(345));

        let test_message = MidiMessage::NoteOn {
            key: 7.into(),
            vel: 13.into(),
        };
        if let Ok(received) = midi_messages_received.lock() {
            assert_eq!(
                *received, 0,
                "Before sending an external MIDI message to Orchestrator, count should be zero"
            );
        };
        o.handle_midi_message(
            MidiChannel::default(),
            test_message,
            &mut |channel, message| panic!("Didn't expect {channel:?} {message:?}",),
        );
        if let Ok(received) = midi_messages_received.lock() {
            assert_eq!(
                *received, 1,
                "Count should update after sending an external MIDI message to Orchestrator"
            );
        };
    }

    #[cfg(obsolete)]
    #[test]
    fn midi_messages_from_track_a_do_not_reach_track_b() {
        let mut o = Orchestrator::default();
        let track_a_uid = o.new_midi_track().unwrap();
        let track_b_uid = o.new_midi_track().unwrap();

        // On Track 1, put a sender and receiver.
        let _ = o.tracks.get_mut(&track_a_uid).unwrap().append_entity(
            Box::new(ToyControllerAlwaysSendsMidiMessage::default()),
            Uid(10001),
        );
        let receiver_1 = TestInstrumentCountsMidiMessages::default();
        let counter_1 = Arc::clone(receiver_1.received_midi_message_count_mutex());
        let _ = o
            .tracks
            .get_mut(&track_a_uid)
            .unwrap()
            .append_entity(Box::new(receiver_1), Uid(10002));

        // On Track 2, put another receiver.
        let receiver_2 = TestInstrumentCountsMidiMessages::default();
        let counter_2 = Arc::clone(receiver_2.received_midi_message_count_mutex());
        let _ = o
            .tracks
            .get_mut(&track_b_uid)
            .unwrap()
            .append_entity(Box::new(receiver_2), Uid(20001));

        // Fire everything up.
        o.play();
        o.work(&mut |_, _| {});

        // Sender should have sent a message that receiver #1 should receive,
        // because they're both in the same Track.
        if let Ok(c) = counter_1.lock() {
            assert_eq!(1, *c);
        }
        // But Receiver #2 shouldn't see that message, because it's in a
        // different Track.
        if let Ok(c) = counter_2.lock() {
            assert_eq!(0, *c);
        };
    }

    #[test]
    fn orchestrator_orchestrates() {
        let mut orchestrator = Orchestrator::default();
        validate_orchestrates_trait(&mut orchestrator);
    }

    #[test]
    fn new_orchestrator_orchestrates() {
        let mut orchestrator = Orchestrator::default();
        validate_orchestrates_trait(&mut orchestrator);
    }

    /// An [IsController] that sends one Control event each time work() is called.
    #[derive(Debug, Default, IsController, Metadata)]
    pub struct TestControllerSendsOneEvent {
        uid: Uid,
    }
    impl Displays for TestControllerSendsOneEvent {}
    impl HandlesMidi for TestControllerSendsOneEvent {}
    impl Controls for TestControllerSendsOneEvent {
        fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
            control_events_fn(None, EntityEvent::Control(ControlValue::MAX));
        }
    }
    impl Configurable for TestControllerSendsOneEvent {}
    impl Serializable for TestControllerSendsOneEvent {}

    #[test]
    fn orchestrator_handles_transport_control() {
        let mut orchestrator = Orchestrator::default();
        let track_uid = orchestrator.create_track().unwrap();
        let uid = orchestrator
            .assign_uid_and_add_entity(&track_uid, Box::new(TestControllerSendsOneEvent::default()))
            .unwrap();

        const TEMPO_INDEX: ControlIndex = ControlIndex(0);
        assert!(orchestrator
            .link_control(uid, Orchestrator::TRANSPORT_UID, TEMPO_INDEX)
            .is_ok());

        assert_eq!(orchestrator.tempo(), Tempo::default());
        orchestrator.work(&mut |_, _| {});
        assert_eq!(orchestrator.tempo(), Tempo::from(Tempo::MAX_VALUE));
    }

    #[test]
    fn transport_is_automatable() {
        let mut t = TransportBuilder::default().build().unwrap();

        assert_eq!(t.tempo(), Tempo::default());

        assert_eq!(
            t.control_index_count(),
            1,
            "Transport should have one automatable parameter"
        );
        const TEMPO_INDEX: ControlIndex = ControlIndex(0);
        assert_eq!(
            t.control_name_for_index(TEMPO_INDEX),
            Some("tempo".to_string()),
            "Transport's parameter name should be 'tempo'"
        );
        t.control_set_param_by_index(TEMPO_INDEX, ControlValue::MAX);
        assert_eq!(t.tempo(), Tempo::from(Tempo::MAX_VALUE));
        t.control_set_param_by_index(TEMPO_INDEX, ControlValue::MIN);
        assert_eq!(t.tempo(), Tempo::from(Tempo::MIN_VALUE));
    }
}
