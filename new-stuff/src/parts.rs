// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::types::ControlLink;
use anyhow::{anyhow, Result};
use delegate::delegate;
use ensnare_core::{
    generators::{PathUid, SignalPath},
    prelude::*,
    traits::{ControlProxyEventsFn, ControlsAsProxy},
};
use ensnare_entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, option::Option};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Orchestrator {
    pub track_repo: TrackRepository,
    pub entity_repo: EntityRepository,

    pub aux_track_uids: Vec<TrackUid>,
    pub bus_station: BusStation,
    pub humidifier: Humidifier,
    pub mixer: Mixer,
}
impl Orchestrator {
    delegate! {
        to self.track_repo {
            pub fn create_track(&mut self, uid: Option<TrackUid>) -> Result<TrackUid>;
            #[call(uids)]
            pub fn track_uids(&self) -> &[TrackUid];
            pub fn set_track_position(&mut self, uid: TrackUid, new_position: usize) -> Result<()>;
            pub fn mint_track_uid(&self) -> TrackUid;
        }
        to self.entity_repo {
            pub fn add_entity(
                &mut self,
                track_uid: TrackUid,
                entity: Box<dyn EntityBounds>,
                uid: Option<Uid>,
            ) -> Result<Uid>;
            pub fn move_entity(
                &mut self,
                uid: Uid,
                new_track_uid: Option<TrackUid>,
                new_position: Option<usize>,
            ) -> Result<()>;
            pub fn delete_entity(&mut self, uid: Uid) -> Result<()>;
            pub fn remove_entity(&mut self, uid: Uid) -> Result<Box<dyn EntityBounds>>;
            pub fn mint_entity_uid(&self) -> Uid;
        }
        to self.entity_repo.entities {
            #[call(get_mut)]
            pub fn get_entity_mut(&mut self, uid: &Uid) -> Option<&mut Box<(dyn EntityBounds)>>;
        }
        to self.bus_station {
            pub fn add_send(&mut self, src_uid: TrackUid, dst_uid: TrackUid, amount: Normal) -> anyhow::Result<()>;
            pub fn remove_send(&mut self, send_track_uid: TrackUid, aux_track_uid: TrackUid);
        }
        to self.humidifier {
            pub fn get_humidity(&self, uid: &Uid) -> Normal;
            pub fn set_humidity(&mut self, uid: Uid, humidity: Normal);
            pub fn transform_batch(
                &mut self,
                humidity: Normal,
                effect: &mut Box<dyn EntityBounds>,
                samples: &mut [StereoSample],
            );
        }
        to self.mixer {
            pub fn track_output(&mut self, track_uid: TrackUid) -> Normal;
            pub fn set_track_output(&mut self, track_uid: TrackUid, output: Normal);
            pub fn mute_track(&mut self, track_uid: TrackUid, muted: bool);
            pub fn is_track_muted(&mut self, track_uid: TrackUid) -> bool;
            pub fn solo_track(&self) -> Option<TrackUid>;
            pub fn set_solo_track(&mut self, track_uid: Option<TrackUid>);
        }
    }

    pub fn delete_track(&mut self, uid: TrackUid) -> Result<()> {
        self.bus_station.remove_sends_for_track(uid);
        self.track_repo.delete_track(uid)
    }

    pub fn entities_for_track(&self, uid: TrackUid) -> Option<&Vec<Uid>> {
        self.entity_repo.uids_for_track.get(&uid)
    }

    pub fn track_for_entity(&self, uid: Uid) -> Option<TrackUid> {
        self.entity_repo.track_for_uid.get(&uid).copied()
    }
}
impl Controls for Orchestrator {
    fn time_range(&self) -> Option<TimeRange> {
        self.entity_repo.time_range()
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.entity_repo.update_time_range(time_range)
    }

    fn is_finished(&self) -> bool {
        self.entity_repo.is_finished()
    }

    fn play(&mut self) {
        self.entity_repo.play();
    }

    fn stop(&mut self) {
        self.entity_repo.stop();
    }

    fn skip_to_start(&mut self) {
        self.entity_repo.skip_to_start()
    }

    fn is_performing(&self) -> bool {
        self.entity_repo.is_performing()
    }
}
impl ControlsAsProxy for Orchestrator {
    fn work_as_proxy(&mut self, control_events_fn: &mut ControlProxyEventsFn) {
        self.entity_repo.work_as_proxy(control_events_fn)
    }
}
impl Generates<StereoSample> for Orchestrator {
    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        let buffer_len = values.len();
        let solo_track_uid = self.solo_track();

        // First handle all non-aux tracks. As a side effect, we also create empty buffers for the aux tracks.
        let (track_buffers, mut aux_track_buffers): (
            HashMap<TrackUid, Vec<StereoSample>>,
            HashMap<TrackUid, Vec<StereoSample>>,
        ) = self.track_repo.uids.iter().fold(
            (HashMap::default(), HashMap::default()),
            |(mut h, mut aux_h), track_uid| {
                let mut track_buffer = Vec::default();
                track_buffer.resize(buffer_len, StereoSample::SILENCE);
                if self.aux_track_uids.contains(track_uid) {
                    aux_h.insert(*track_uid, track_buffer);
                } else {
                    let should_work = !self.mixer.is_track_muted(*track_uid)
                        && (solo_track_uid.is_none() || solo_track_uid == Some(*track_uid));
                    if should_work {
                        if let Some(entity_uids) = self.entity_repo.uids_for_track.get(track_uid) {
                            entity_uids.iter().for_each(|uid| {
                                if let Some(entity) = self.entity_repo.entities.get_mut(uid) {
                                    entity.generate_batch_values(&mut track_buffer);
                                    let humidity = self.humidifier.get_humidity(uid);
                                    if humidity != Normal::zero() {
                                        self.humidifier.transform_batch(
                                            humidity,
                                            entity,
                                            &mut track_buffer,
                                        );
                                    }
                                }
                            });
                        }
                    }
                    h.insert(*track_uid, track_buffer);
                }
                (h, aux_h)
            },
        );

        // Then send audio to the aux tracks.
        for (track_uid, routes) in self.bus_station.sends() {
            // We have a source track_uid and the aux tracks that should receive it.
            if let Some(source_track_buffer) = track_buffers.get(track_uid) {
                // Mix the source into the destination aux track.
                for route in routes {
                    if let Some(aux) = aux_track_buffers.get_mut(&route.aux_track_uid) {
                        for (src, dst) in source_track_buffer.iter().zip(aux.iter_mut()) {
                            *dst += *src * route.amount;
                        }
                    }
                }
            }
        }

        // Let the aux tracks do their processing.
        aux_track_buffers
            .iter_mut()
            .for_each(|(track_uid, track_buffer)| {
                let should_work = !self.mixer.is_track_muted(*track_uid)
                    && (solo_track_uid.is_none() || solo_track_uid == Some(*track_uid));
                if should_work {
                    if let Some(entity_uids) = self.entity_repo.uids_for_track.get(track_uid) {
                        entity_uids.iter().for_each(|uid| {
                            if let Some(entity) = self.entity_repo.entities.get_mut(uid) {
                                entity.transform_batch(track_buffer);
                            }
                        });
                    }
                }
            });

        // Mix all the tracks into the final buffer.
        track_buffers
            .iter()
            .chain(aux_track_buffers.iter())
            .for_each(|(track_uid, buffer)| {
                let should_mix = !self.mixer.is_track_muted(*track_uid)
                    && (solo_track_uid.is_none() || solo_track_uid == Some(*track_uid));
                if should_mix {
                    let output = self.track_output(*track_uid);
                    for (dst, src) in values.iter_mut().zip(buffer) {
                        *dst += *src * output;
                    }
                }
            });
    }
}
impl Ticks for Orchestrator {}
impl Configurable for Orchestrator {
    fn sample_rate(&self) -> SampleRate {
        self.entity_repo.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.entity_repo.update_sample_rate(sample_rate)
    }

    fn tempo(&self) -> Tempo {
        self.entity_repo.tempo()
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.entity_repo.update_tempo(tempo)
    }

    fn time_signature(&self) -> TimeSignature {
        self.entity_repo.time_signature()
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.entity_repo.update_time_signature(time_signature)
    }
}
impl Serializable for Orchestrator {
    fn before_ser(&mut self) {
        self.track_repo.before_ser();
        self.entity_repo.before_ser();
    }

    fn after_deser(&mut self) {
        self.track_repo.after_deser();
        self.entity_repo.after_deser();
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrackRepository {
    uid_factory: UidFactory<TrackUid>,
    uids: Vec<TrackUid>,
}
impl TrackRepository {
    pub fn create_track(&mut self, uid: Option<TrackUid>) -> Result<TrackUid> {
        let uid = if let Some(uid) = uid {
            uid
        } else {
            self.uid_factory.mint_next()
        };
        self.uids.push(uid);
        Ok(uid)
    }

    pub fn set_track_position(&mut self, uid: TrackUid, new_position: usize) -> Result<()> {
        if self.uids.contains(&uid) {
            self.delete_track(uid)?;
            self.uids.insert(new_position, uid);
            Ok(())
        } else {
            Err(anyhow!("Track {uid} not found"))
        }
    }

    pub fn delete_track(&mut self, uid: TrackUid) -> Result<()> {
        self.uids.retain(|tuid| *tuid != uid);
        Ok(())
    }

    delegate! {
        to self.uid_factory {
            #[call(mint_next)]
            pub fn mint_track_uid(&self) -> TrackUid;
        }
    }

    pub fn uids(&self) -> &[TrackUid] {
        self.uids.as_ref()
    }
}
impl Serializable for TrackRepository {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {}
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EntityRepository {
    uid_factory: UidFactory<Uid>,
    pub entities: HashMap<Uid, Box<dyn EntityBounds>>,
    pub uids_for_track: HashMap<TrackUid, Vec<Uid>>,
    pub track_for_uid: HashMap<Uid, TrackUid>,

    #[serde(skip)]
    sample_rate: SampleRate,
    #[serde(skip)]
    tempo: Tempo,
    #[serde(skip)]
    time_signature: TimeSignature,

    #[serde(skip)]
    is_finished: bool,
}
impl EntityRepository {
    delegate! {
        to self.uid_factory {
            #[call(mint_next)]
            pub fn mint_entity_uid(&self) -> Uid;
        }
    }

    /// Adds the provided [Entity] to the repository.
    ///
    /// The uid is determined using ordered rules.
    ///
    /// 1. If the optional uid parameter is present, then it is used.
    /// 2. If the entity has a non-default Uid, then it is used.
    /// 3. The repository generates a new Uid.
    ///
    /// In any case, the repo sets the entity Uid to match.
    pub fn add_entity(
        &mut self,
        track_uid: TrackUid,
        mut entity: Box<dyn EntityBounds>,
        uid: Option<Uid>,
    ) -> Result<Uid> {
        let uid = if let Some(uid) = uid {
            uid
        } else if entity.uid() != Uid::default() {
            entity.uid()
        } else {
            self.uid_factory.mint_next()
        };
        entity.set_uid(uid);
        self.entities.insert(uid, entity);
        self.uids_for_track
            .entry(track_uid.clone())
            .or_default()
            .push(uid);
        self.track_for_uid.insert(uid, track_uid.clone());
        Ok(uid)
    }

    pub fn move_entity(
        &mut self,
        uid: Uid,
        new_track_uid: Option<TrackUid>,
        new_position: Option<usize>,
    ) -> Result<()> {
        if !self.entities.contains_key(&uid) {
            return Err(anyhow!("Entity {uid} not found"));
        }
        if let Some(new_track_uid) = new_track_uid {
            if let Some(old_track_uid) = self.track_for_uid.get(&uid) {
                if *old_track_uid != new_track_uid {
                    if let Some(uids) = self.uids_for_track.get_mut(old_track_uid) {
                        uids.retain(|u| *u != uid);
                        self.uids_for_track
                            .entry(new_track_uid)
                            .or_default()
                            .push(uid);
                    }
                }
            }
            self.track_for_uid.insert(uid, new_track_uid);
        }
        if let Some(new_position) = new_position {
            if let Some(track_uid) = self.track_for_uid.get(&uid) {
                let uids = self.uids_for_track.entry(*track_uid).or_default();
                uids.retain(|u| *u != uid);
                uids.insert(new_position, uid);
            }
        }
        Ok(())
    }

    pub fn delete_entity(&mut self, uid: Uid) -> Result<()> {
        let _ = self.remove_entity(uid)?;
        Ok(())
    }

    pub fn remove_entity(&mut self, uid: Uid) -> Result<Box<dyn EntityBounds>> {
        if let Some(track_uid) = self.track_for_uid.get(&uid) {
            self.uids_for_track
                .entry(*track_uid)
                .or_default()
                .retain(|u| *u != uid);
            self.track_for_uid.remove(&uid);
            if let Some(entity) = self.entities.remove(&uid) {
                return Ok(entity);
            }
        }
        Err(anyhow!("Entity {uid} not found"))
    }

    pub fn entity(&self, uid: Uid) -> Option<&Box<dyn EntityBounds>> {
        self.entities.get(&uid)
    }

    pub fn entity_mut(&mut self, uid: Uid) -> Option<&mut Box<dyn EntityBounds>> {
        self.entities.get_mut(&uid)
    }

    pub fn uids_for_track(&self) -> &HashMap<TrackUid, Vec<Uid>> {
        &self.uids_for_track
    }

    fn update_is_finished(&mut self) {
        self.is_finished = self.entities.values().all(|e| e.is_finished());
    }
}
impl Controls for EntityRepository {
    fn time_range(&self) -> Option<TimeRange> {
        None
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.entities
            .values_mut()
            .for_each(|e| e.update_time_range(time_range));
    }

    fn work(&mut self, _: &mut ControlEventsFn) {
        panic!("call work_as_proxy() instead")
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn play(&mut self) {
        self.entities.values_mut().for_each(|e| e.play());
        self.update_is_finished();
    }

    fn stop(&mut self) {
        self.entities.values_mut().for_each(|e| {
            e.stop();
        });
    }

    fn skip_to_start(&mut self) {
        self.entities.values_mut().for_each(|e| {
            e.skip_to_start();
        });
    }

    fn is_performing(&self) -> bool {
        false
    }
}
impl ControlsAsProxy for EntityRepository {
    fn work_as_proxy(&mut self, control_events_fn: &mut ControlProxyEventsFn) {
        self.entities.iter_mut().for_each(|(uid, e)| {
            let mut track_uid = None;
            e.work(&mut |inner_event| match inner_event {
                WorkEvent::Midi(channel, message) => {
                    if track_uid.is_none() {
                        track_uid = self.track_for_uid.get(uid).copied();
                    }
                    control_events_fn(
                        *uid,
                        WorkEvent::MidiForTrack(track_uid.unwrap_or_default(), channel, message),
                    );
                }
                _ => control_events_fn(*uid, inner_event),
            })
        });
        self.update_is_finished();
    }
}
impl Configurable for EntityRepository {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.entities
            .values_mut()
            .for_each(|e| e.update_sample_rate(sample_rate));
    }

    fn tempo(&self) -> Tempo {
        self.tempo
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo;
        self.entities
            .values_mut()
            .for_each(|e| e.update_tempo(tempo))
    }

    fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
        self.entities
            .values_mut()
            .for_each(|e| e.update_time_signature(time_signature))
    }
}
impl Ticks for EntityRepository {
    fn tick(&mut self, tick_count: usize) {
        self.entities.values_mut().for_each(|e| e.tick(tick_count));
    }
}
impl Serializable for EntityRepository {
    fn before_ser(&mut self) {
        self.entities.values_mut().for_each(|e| e.before_ser());
    }

    fn after_deser(&mut self) {
        self.entities.values_mut().for_each(|e| e.after_deser());
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Automator {
    pub controllables: HashMap<Uid, Vec<ControlLink>>,

    uid_factory: UidFactory<PathUid>,
    pub paths: HashMap<PathUid, SignalPath>,
    pub path_links: HashMap<PathUid, Vec<ControlLink>>,

    #[serde(skip)]
    is_finished: bool,
    #[serde(skip)]
    time_range: TimeRange,
}
impl Automator {
    pub fn link(&mut self, source: Uid, target: Uid, param: ControlIndex) -> Result<()> {
        self.controllables
            .entry(source)
            .or_default()
            .push(ControlLink { uid: target, param });
        Ok(())
    }

    pub fn unlink(&mut self, source: Uid, target: Uid, param: ControlIndex) {
        if let Some(controllables) = self.controllables.get_mut(&source) {
            controllables.retain(|rlink| (ControlLink { uid: target, param }) != *rlink);
        }
    }

    pub fn control_links(&self, uid: Uid) -> Option<&Vec<ControlLink>> {
        self.controllables.get(&uid)
    }

    pub fn route(
        &mut self,
        entity_repo: &mut EntityRepository,
        mut not_found_fn: Option<&mut dyn FnMut(&ControlLink)>,
        uid: Uid,
        value: ControlValue,
    ) {
        if let Some(controllables) = self.controllables.get(&uid) {
            controllables.iter().for_each(|link| {
                if let Some(entity) = entity_repo.entity_mut(link.uid) {
                    entity.control_set_param_by_index(link.param, value);
                } else {
                    if let Some(not_found_fn) = not_found_fn.as_mut() {
                        not_found_fn(link);
                    }
                }
            });
        }
    }

    pub fn add_path(&mut self, path: SignalPath) -> Result<PathUid> {
        let path_uid = self.uid_factory.mint_next();
        self.paths.insert(path_uid, path);
        Ok(path_uid)
    }

    pub fn remove_path(&mut self, path_uid: PathUid) -> Option<SignalPath> {
        self.paths.remove(&path_uid)
    }

    pub fn link_path(
        &mut self,
        path_uid: PathUid,
        target_uid: Uid,
        param: ControlIndex,
    ) -> Result<()> {
        if self.paths.contains_key(&path_uid) {
            self.path_links
                .entry(path_uid)
                .or_default()
                .push(ControlLink {
                    uid: target_uid,
                    param,
                });
            Ok(())
        } else {
            Err(anyhow!("Couldn't find path {path_uid}"))
        }
    }

    pub fn unlink_path(&mut self, path_uid: PathUid) {
        self.path_links.entry(path_uid).or_default().clear();
    }
}
impl Serializable for Automator {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {}
}
impl Controls for Automator {
    fn time_range(&self) -> Option<TimeRange> {
        Some(self.time_range.clone())
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.time_range = time_range.clone();
    }

    fn work(&mut self, _control_events_fn: &mut ControlEventsFn) {
        self.is_finished = true;
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MidiRouter {
    pub midi_receivers: HashMap<MidiChannel, Vec<Uid>>,
}
impl MidiRouter {
    pub fn set_midi_receiver_channel(
        &mut self,
        entity_uid: Uid,
        channel: Option<MidiChannel>,
    ) -> Result<()> {
        if let Some(channel) = channel {
            self.midi_receivers
                .entry(channel)
                .or_default()
                .push(entity_uid);
        } else {
            self.midi_receivers
                .values_mut()
                .for_each(|receivers| receivers.retain(|receiver_uid| *receiver_uid != entity_uid));
        }
        Ok(())
    }

    pub fn route(
        &self,
        entity_repo: &mut EntityRepository,
        channel: MidiChannel,
        message: MidiMessage,
    ) -> anyhow::Result<()> {
        let mut loop_detected = false;
        let mut v = Vec::default();
        v.push((channel, message));
        while let Some((channel, message)) = v.pop() {
            if let Some(receivers) = self.midi_receivers.get(&channel) {
                receivers.iter().for_each(|receiver_uid| {
                if let Some(entity) = entity_repo.entity_mut(*receiver_uid) {
                    entity.handle_midi_message(channel, message, &mut |c, m| {
                        if channel != c {
                            v.push((c, m));
                        } else if !loop_detected {
                            loop_detected = true;
                            eprintln!("Warning: loop detected; while sending to channel {channel}, received request to send {:#?} to same channel", &m);
                        }
                    });
                }
            });
            }
        }
        if loop_detected {
            Err(anyhow!("Device attempted to send MIDI message to itself"))
        } else {
            Ok(())
        }
    }
}
impl Serializable for MidiRouter {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {}
}

/// A [BusRoute] represents a signal connection between two tracks.
#[derive(Debug, Serialize, Deserialize)]
pub struct BusRoute {
    /// The [TrackUid] of the receiving track.
    pub aux_track_uid: TrackUid,
    /// How much gain should be applied to this connection.
    pub amount: Normal,
}

/// A [BusStation] manages how signals move between tracks and aux tracks. These
/// collections of signals are sometimes called buses.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BusStation {
    routes: HashMap<TrackUid, Vec<BusRoute>>,
}

impl BusStation {
    pub(crate) fn add_send(
        &mut self,
        track_uid: TrackUid,
        dst_uid: TrackUid,
        amount: Normal,
    ) -> anyhow::Result<()> {
        self.routes.entry(track_uid).or_default().push(BusRoute {
            aux_track_uid: dst_uid,
            amount,
        });
        Ok(())
    }

    pub(crate) fn remove_send(&mut self, track_uid: TrackUid, aux_track_uid: TrackUid) {
        if let Some(routes) = self.routes.get_mut(&track_uid) {
            routes.retain(|route| route.aux_track_uid != aux_track_uid);
        }
    }

    pub(crate) fn sends(&self) -> impl Iterator<Item = (&TrackUid, &Vec<BusRoute>)> {
        self.routes.iter()
    }

    // If we want this method to be immutable and cheap, then we can't guarantee
    // that it will return a Vec. Such is life.
    #[allow(dead_code)]
    pub(crate) fn sends_for_track(&self, track_uid: &TrackUid) -> Option<&Vec<BusRoute>> {
        self.routes.get(track_uid)
    }

    pub(crate) fn remove_sends_for_track(&mut self, track_uid: TrackUid) {
        self.routes.remove(&track_uid);
        self.routes
            .values_mut()
            .for_each(|routes| routes.retain(|route| route.aux_track_uid != track_uid));
    }
}

/// Controls the wet/dry mix of arranged effects.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Humidifier {
    uid_to_humidity: HashMap<Uid, Normal>,
}
impl Humidifier {
    pub fn get_humidity(&self, uid: &Uid) -> Normal {
        self.uid_to_humidity.get(uid).cloned().unwrap_or_default()
    }

    pub fn set_humidity(&mut self, uid: Uid, humidity: Normal) {
        self.uid_to_humidity.insert(uid, humidity);
    }

    pub fn transform_batch(
        &mut self,
        humidity: Normal,
        effect: &mut Box<dyn EntityBounds>,
        samples: &mut [StereoSample],
    ) {
        for sample in samples {
            *sample = self.transform_audio(humidity, *sample, effect.transform_audio(*sample));
        }
    }

    pub fn transform_audio(
        &mut self,
        humidity: Normal,
        pre_effect: StereoSample,
        post_effect: StereoSample,
    ) -> StereoSample {
        StereoSample(
            self.transform_channel(humidity, 0, pre_effect.0, post_effect.0),
            self.transform_channel(humidity, 1, pre_effect.1, post_effect.1),
        )
    }

    fn transform_channel(
        &mut self,
        humidity: Normal,
        _: usize,
        pre_effect: Sample,
        post_effect: Sample,
    ) -> Sample {
        let humidity: f64 = humidity.into();
        let aridity = 1.0 - humidity;
        post_effect * humidity + pre_effect * aridity
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mixer {
    track_output: HashMap<TrackUid, Normal>,
    track_mute: HashMap<TrackUid, bool>,
    pub solo_track: Option<TrackUid>,
}
impl Mixer {
    pub fn track_output(&mut self, track_uid: TrackUid) -> Normal {
        self.track_output
            .get(&track_uid)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_track_output(&mut self, track_uid: TrackUid, output: Normal) {
        self.track_output.insert(track_uid, output);
    }

    pub fn mute_track(&mut self, track_uid: TrackUid, muted: bool) {
        self.track_mute.insert(track_uid, muted);
    }

    pub fn is_track_muted(&mut self, track_uid: TrackUid) -> bool {
        self.track_mute.get(&track_uid).copied().unwrap_or_default()
    }

    pub fn solo_track(&self) -> Option<TrackUid> {
        self.solo_track
    }

    pub fn set_solo_track(&mut self, track_uid: Option<TrackUid>) {
        self.solo_track = track_uid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ensnare_core::time::TransportBuilder;
    use ensnare_cores::TestEffectNegatesInput;
    use ensnare_entities::instruments::TestInstrument;
    use ensnare_proc_macros::{Control, IsEntity, Metadata};
    use more_asserts::assert_gt;
    use std::sync::{Arc, RwLock};

    #[test]
    fn track_repo_mainline() {
        let mut repo = TrackRepository::default();

        assert!(repo.uids.is_empty(), "Default should have no tracks");

        let track_1_uid = repo.create_track(None).unwrap();
        assert_gt!(track_1_uid.0, 0, "new track's uid should be nonzero");
        assert_eq!(repo.uids.len(), 1, "should be one track after creating one");

        let track_2_uid = repo.create_track(None).unwrap();
        assert_eq!(
            repo.uids.len(),
            2,
            "should be two tracks after creating second"
        );
        assert!(repo.set_track_position(track_2_uid, 0).is_ok());
        assert_eq!(
            repo.uids,
            vec![track_2_uid, track_1_uid],
            "order of track uids should be as expected after move"
        );
        assert!(repo.delete_track(track_2_uid).is_ok());

        assert_ne!(
            repo.mint_track_uid(),
            repo.mint_track_uid(),
            "Two consecutively minted Uids should be different."
        );
    }

    #[test]
    fn entity_repo_mainline() {
        let mut repo = EntityRepository::default();
        assert!(repo.entities.is_empty(), "Initial repo should be empty");

        let track_uid = TrackUid(1);
        let uid = repo
            .add_entity(track_uid, Box::new(TestInstrument::default()), None)
            .unwrap();
        let entity = repo.remove_entity(uid).unwrap();
        assert_ne!(
            entity.uid(),
            Uid::default(),
            "add_entity(..., None) with an entity having a default Uid should assign an autogen Uid"
        );
        assert!(
            repo.entities.is_empty(),
            "Repo should be empty after removing inserted entities"
        );

        let expected_uid = Uid(998877);
        let uid = repo
            .add_entity(
                track_uid,
                Box::new(TestInstrument::new_with(expected_uid)),
                None,
            )
            .unwrap();
        let entity = repo.remove_entity(uid).unwrap();
        assert_eq!(
            entity.uid(),
            expected_uid,
            "add_entity(..., None) with an entity having a Uid should use that Uid"
        );
        assert!(
            repo.entities.is_empty(),
            "Repo should be empty after removing inserted entities"
        );

        let expected_uid = Uid(998877);
        let uid = repo
            .add_entity(
                track_uid,
                Box::new(TestInstrument::new_with(Uid(33333))),
                Some(expected_uid),
            )
            .unwrap();
        let entity = repo.remove_entity(uid).unwrap();
        assert_eq!(
            entity.uid(),
            expected_uid,
            "add_entity(..., Some) with an entity having a Uid should use the Uid provided as the parameter"
        );
        assert!(
            repo.entities.is_empty(),
            "Repo should be empty after removing inserted entities"
        );
    }

    #[test]
    fn orchestrator_mainline() {
        let mut orchestrator = Orchestrator::default();

        let nonexistent_track_uid = TrackUid(12345);
        assert!(
            orchestrator
                .entities_for_track(nonexistent_track_uid)
                .is_none(),
            "Getting track entities for nonexistent track should return None"
        );

        let track_uid = orchestrator.create_track(None).unwrap();
        assert!(
            orchestrator.entities_for_track(track_uid).is_none(),
            "Getting track entries for a track that exists but is empty should return None"
        );
        let target_uid = orchestrator
            .add_entity(track_uid, Box::new(TestInstrument::default()), None)
            .unwrap();
        assert_eq!(
            orchestrator.track_for_entity(target_uid).unwrap(),
            track_uid,
            "Added entity's track uid should be retrievable"
        );
        let track_entities = orchestrator.entities_for_track(track_uid).unwrap();
        assert_eq!(track_entities.len(), 1);
        assert!(track_entities.contains(&target_uid));

        assert!(
            orchestrator.get_entity_mut(&Uid(99999)).is_none(),
            "getting nonexistent entity should return None"
        );
        assert!(
            orchestrator.get_entity_mut(&target_uid).is_some(),
            "getting an entity should return it"
        );
    }

    #[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
    #[entity(
        Configurable,
        Controls,
        Displays,
        GeneratesStereoSample,
        HandlesMidi,
        Serializable,
        SkipInner,
        Ticks,
        TransformsAudio
    )]
    pub struct TestControllable {
        uid: Uid,
        #[serde(skip)]
        tracker: Arc<RwLock<Vec<(Uid, ControlIndex, ControlValue)>>>,
    }
    impl TestControllable {
        pub fn new_with(tracker: Arc<RwLock<Vec<(Uid, ControlIndex, ControlValue)>>>) -> Self {
            Self {
                uid: Default::default(),
                tracker,
            }
        }
    }
    impl Controllable for TestControllable {
        fn control_set_param_by_index(&mut self, index: ControlIndex, value: ControlValue) {
            if let Ok(mut tracker) = self.tracker.write() {
                tracker.push((self.uid, index, value));
            }
        }
    }

    #[test]
    fn automator_mainline() {
        let mut automator = Automator::default();

        assert!(
            automator.controllables.is_empty(),
            "new Automator should be empty"
        );

        let source_1_uid = Uid(1);
        let source_2_uid = Uid(2);
        let target_1_uid = Uid(3);
        let target_2_uid = Uid(4);

        assert!(automator
            .link(source_1_uid, target_1_uid, ControlIndex(0))
            .is_ok());
        assert_eq!(
            automator.controllables.len(),
            1,
            "there should be one vec after inserting one link"
        );
        assert!(automator
            .link(source_1_uid, target_2_uid, ControlIndex(1))
            .is_ok());
        assert_eq!(
            automator.controllables.len(),
            1,
            "there should still be one vec after inserting a second link for same source_uid"
        );
        assert!(automator
            .link(source_2_uid, target_1_uid, ControlIndex(0))
            .is_ok());
        assert_eq!(
            automator.controllables.len(),
            2,
            "there should be two vecs after inserting one link for a second Uid"
        );

        assert_eq!(
            automator.control_links(source_1_uid).unwrap().len(),
            2,
            "the first source's vec should have two entries"
        );
        assert_eq!(
            automator.control_links(source_2_uid).unwrap().len(),
            1,
            "the second source's vec should have one entry"
        );

        let tracker = Arc::new(RwLock::new(Vec::default()));
        let controllable_1 = TestControllable::new_with(Arc::clone(&tracker));
        let controllable_2 = TestControllable::new_with(Arc::clone(&tracker));
        let track_uid = TrackUid(1);
        let mut repo = EntityRepository::default();
        let _ = repo.add_entity(track_uid, Box::new(controllable_1), Some(target_1_uid));
        let _ = repo.add_entity(track_uid, Box::new(controllable_2), Some(target_2_uid));

        let _ = automator.route(&mut repo, None, source_1_uid, ControlValue(0.5));
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                2,
                "there should be expected number of control events after the route {:#?}",
                t
            );
            assert_eq!(t[0], (target_1_uid, ControlIndex(0), ControlValue(0.5)));
            assert_eq!(t[1], (target_2_uid, ControlIndex(1), ControlValue(0.5)));
        };

        // Try removing links. Start with nonexistent link
        if let Ok(mut t) = tracker.write() {
            t.clear();
        }
        automator.unlink(source_1_uid, target_1_uid, ControlIndex(99));
        let _ = automator.route(&mut repo, None, source_1_uid, ControlValue(0.5));
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                2,
                "route results shouldn't change when removing nonexistent link {:#?}",
                t
            );
        };

        if let Ok(mut t) = tracker.write() {
            t.clear();
        }
        automator.unlink(source_1_uid, target_1_uid, ControlIndex(0));
        let _ = automator.route(&mut repo, None, source_1_uid, ControlValue(0.5));
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                1,
                "removing a link should continue routing to remaining ones {:#?}",
                t
            );
            assert_eq!(t[0], (target_2_uid, ControlIndex(1), ControlValue(0.5)));
        };
    }

    #[test]
    fn automator_paths_mainline() {
        let mut automator = Automator::default();
        assert!(automator.paths.is_empty());
        assert!(automator.path_links.is_empty());

        let path_uid = automator.add_path(SignalPath::default()).unwrap();
        assert_eq!(automator.paths.len(), 1);
        assert!(automator.path_links.is_empty());

        let target_uid = Uid(1024);
        let _ = automator.link_path(path_uid, target_uid, ControlIndex(123));

        automator.update_time_range(&TimeRange::new_with_start_and_duration(
            MusicalTime::START,
            MusicalTime::DURATION_SIXTEENTH,
        ));
        automator.work(&mut |event| {
            todo!("got {event:?}");
        });

        // TODO: finish this
    }

    #[derive(Debug, Control, Default, IsEntity, Metadata, Serialize, Deserialize)]
    #[entity(
        Configurable,
        Controls,
        Displays,
        Serializable,
        SkipInner,
        TransformsAudio
    )]
    struct TestHandlesMidi {
        uid: Uid,
        rebroadcast_to: Option<MidiChannel>,
        #[serde(skip)]
        tracker: Arc<RwLock<Vec<(Uid, MidiChannel, MidiMessage)>>>,
    }
    impl TestHandlesMidi {
        fn new_with(
            uid: Uid,
            rebroadcast_to: Option<MidiChannel>,
            tracker: Arc<RwLock<Vec<(Uid, MidiChannel, MidiMessage)>>>,
        ) -> Self {
            Self {
                uid,
                rebroadcast_to,
                tracker,
            }
        }
    }
    impl HandlesMidi for TestHandlesMidi {
        fn handle_midi_message(
            &mut self,
            channel: MidiChannel,
            message: MidiMessage,
            midi_messages_fn: &mut MidiMessagesFn,
        ) {
            if let Ok(mut tracker) = self.tracker.write() {
                tracker.push((self.uid, channel, message))
            };
            if let Some(rebroadcast_channel) = self.rebroadcast_to {
                midi_messages_fn(rebroadcast_channel, message);
            }
        }
    }
    impl Generates<StereoSample> for TestHandlesMidi {
        fn value(&self) -> StereoSample {
            todo!()
        }

        #[allow(unused_variables)]
        fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
            todo!()
        }
    }
    impl Ticks for TestHandlesMidi {
        #[allow(unused_variables)]
        fn tick(&mut self, tick_count: usize) {
            todo!()
        }
    }

    #[test]
    fn midi_router_routes_to_correct_channels() {
        let tracker = Arc::new(RwLock::new(Vec::default()));
        let mut repo = EntityRepository::default();
        let entity = Box::new(TestHandlesMidi::new_with(
            Uid(1),
            None,
            Arc::clone(&tracker),
        ));
        let _ = repo.add_entity(TrackUid(1), entity, None);
        let entity = Box::new(TestHandlesMidi::new_with(
            Uid(2),
            None,
            Arc::clone(&tracker),
        ));
        let _ = repo.add_entity(TrackUid(1), entity, None);

        let mut router = MidiRouter::default();
        let _ = router.set_midi_receiver_channel(Uid(1), Some(MidiChannel(1)));
        let _ = router.set_midi_receiver_channel(Uid(2), Some(MidiChannel(2)));

        let m = new_note_on(1, 1);

        assert!(router.route(&mut repo, MidiChannel(99), m).is_ok());
        if let Ok(t) = tracker.read() {
            assert!(
                t.is_empty(),
                "no messages received after routing to nonexistent MIDI channel"
            );
        }
        assert!(router.route(&mut repo, MidiChannel(1), m).is_ok());
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                1,
                "after routing to channel 1, only one listener should receive"
            );
            assert_eq!(
                t[0],
                (Uid(1), MidiChannel(1), m),
                "after routing to channel 1, only channel 1 listener should receive"
            );
        };
        assert!(router.route(&mut repo, MidiChannel(2), m).is_ok());
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                2,
                "after routing to channel 2, only one listener should receive"
            );
            assert_eq!(
                t[1],
                (Uid(2), MidiChannel(2), m),
                "after routing to channel 2, only channel 2 listener should receive"
            );
        };
    }

    #[test]
    fn midi_router_also_routes_produced_messages() {
        let tracker = Arc::new(RwLock::new(Vec::default()));
        let mut repo = EntityRepository::default();
        let entity = Box::new(TestHandlesMidi::new_with(
            Uid(1),
            Some(MidiChannel(2)),
            Arc::clone(&tracker),
        ));
        let _ = repo.add_entity(TrackUid(1), entity, None);
        let entity = Box::new(TestHandlesMidi::new_with(
            Uid(2),
            None,
            Arc::clone(&tracker),
        ));
        let _ = repo.add_entity(TrackUid(1), entity, None);

        let mut r = MidiRouter::default();
        let _ = r.set_midi_receiver_channel(Uid(1), Some(MidiChannel(1)));
        let _ = r.set_midi_receiver_channel(Uid(2), Some(MidiChannel(2)));

        let m = new_note_on(1, 1);

        assert!(r.route(&mut repo, MidiChannel(1), m).is_ok());
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                2,
                "routing to a producing receiver should produce and route a second message"
            );
            assert_eq!(
                t[0],
                (Uid(1), MidiChannel(1), m),
                "original message should be received"
            );
            assert_eq!(
                t[1],
                (Uid(2), MidiChannel(2), m),
                "produced message should be received"
            );
        };
        let m = new_note_on(2, 3);
        assert!(r.route(&mut repo, MidiChannel(2), m).is_ok());
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                3,
                "routing to a non-producing receiver shouldn't produce anything"
            );
            assert_eq!(
                t[2],
                (Uid(2), MidiChannel(2), m),
                "after routing to channel 2, only channel 2 listener should receive"
            );
        };
    }

    #[test]
    fn midi_router_detects_loops() {
        let tracker = Arc::new(RwLock::new(Vec::default()));
        let mut repo = EntityRepository::default();
        let entity = Box::new(TestHandlesMidi::new_with(
            Uid(1),
            Some(MidiChannel(1)),
            Arc::clone(&tracker),
        ));
        let _ = repo.add_entity(TrackUid(1), entity, None);

        let mut r = MidiRouter::default();
        let _ = r.set_midi_receiver_channel(Uid(1), Some(MidiChannel(1)));

        let m = new_note_on(1, 1);

        assert!(r.route(&mut repo, MidiChannel(1), m).is_err());
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

    #[test]
    fn bus_station_mainline() {
        let mut station = BusStation::default();
        assert!(station.routes.is_empty());

        assert!(station
            .add_send(TrackUid(7), TrackUid(13), Normal::from(0.8))
            .is_ok());
        assert_eq!(station.routes.len(), 1);

        assert!(station
            .add_send(TrackUid(7), TrackUid(13), Normal::from(0.7))
            .is_ok());
        assert_eq!(
            station.routes.len(),
            1,
            "Adding a new send route with a new amount should replace the prior one"
        );

        station.remove_send(TrackUid(7), TrackUid(13));
        assert_eq!(
            station.routes.len(),
            1,
            "Removing route should still leave a (possibly empty) Vec"
        );
        assert!(
            station.sends_for_track(&TrackUid(7)).unwrap().is_empty(),
            "Removing route should work"
        );

        // Removing nonexistent route is a no-op
        station.remove_send(TrackUid(7), TrackUid(13));

        assert!(station
            .add_send(TrackUid(7), TrackUid(13), Normal::from(0.8))
            .is_ok());
        assert!(station
            .add_send(TrackUid(7), TrackUid(14), Normal::from(0.8))
            .is_ok());
        assert_eq!(
            station.routes.len(),
            1,
            "Adding two sends to a track should not create an extra Vec"
        );
        assert_eq!(
            station.sends_for_track(&TrackUid(7)).unwrap().len(),
            2,
            "Adding two sends to a track should work"
        );

        // Empty can be either None or Vec::default(). Don't care.
        station.remove_sends_for_track(TrackUid(7));
        if let Some(sends) = station.sends_for_track(&TrackUid(7)) {
            assert!(sends.is_empty(), "Removing all a track's sends should work");
        }
    }

    #[test]
    fn humidifier_lookups_work() {
        let mut wd = Humidifier::default();
        assert_eq!(
            wd.get_humidity(&Uid(1)),
            Normal::maximum(),
            "a missing Uid should return default humidity 1.0"
        );

        let uid = Uid(1);
        wd.set_humidity(uid, Normal::from(0.5));
        assert_eq!(
            wd.get_humidity(&Uid(1)),
            Normal::from(0.5),
            "a non-missing Uid should return the humidity that we set"
        );
    }

    #[test]
    fn humidifier_mainline() {
        let mut humidifier = Humidifier::default();

        let mut effect = TestEffectNegatesInput::default();
        assert_eq!(
            effect.transform_channel(0, Sample::MAX),
            Sample::MIN,
            "we expected ToyEffect to negate the input"
        );

        let pre_effect = Sample::MAX;
        assert_eq!(
            humidifier.transform_channel(
                Normal::maximum(),
                0,
                pre_effect,
                effect.transform_channel(0, pre_effect),
            ),
            Sample::MIN,
            "Wetness 1.0 means full effect, zero pre-effect"
        );
        assert_eq!(
            humidifier.transform_channel(
                Normal::from_percentage(50.0),
                0,
                pre_effect,
                effect.transform_channel(0, pre_effect),
            ),
            Sample::from(0.0),
            "Wetness 0.5 means even parts effect and pre-effect"
        );
        assert_eq!(
            humidifier.transform_channel(
                Normal::zero(),
                0,
                pre_effect,
                effect.transform_channel(0, pre_effect),
            ),
            pre_effect,
            "Wetness 0.0 means no change from pre-effect to post"
        );
    }

    #[test]
    fn mixer_mainline() {
        let mut mixer = Mixer::default();
        assert!(mixer.track_output.is_empty());
        assert!(mixer.track_mute.is_empty());

        let track_1 = TrackUid(1);
        let track_2 = TrackUid(2);

        assert!(!mixer.is_track_muted(track_1));
        assert!(!mixer.is_track_muted(track_2));
        assert!(mixer.solo_track().is_none());

        mixer.set_solo_track(Some(track_1));
        assert_eq!(mixer.solo_track().unwrap(), track_1);
        mixer.set_solo_track(None);
        assert_eq!(mixer.solo_track(), None);

        assert_eq!(mixer.track_output(track_1), Normal::maximum());
        assert_eq!(mixer.track_output(track_2), Normal::maximum());

        mixer.set_track_output(track_2, Normal::from(0.5));
        assert_eq!(mixer.track_output(track_1), Normal::maximum());
        assert_eq!(mixer.track_output(track_2), Normal::from(0.5));
    }
}
