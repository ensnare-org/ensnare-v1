// Copyright (c) 2024 Mike Tsao. All rights reserved.

use anyhow::{anyhow, Result};
use delegate::delegate;
use ensnare_core::{
    prelude::*,
    traits::{ControlProxyEventsFn, ControlsAsProxy},
};
use ensnare_entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, option::Option};

use crate::types::ControlLink;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Orchestrator {
    pub track_repo: TrackRepository,
    pub entity_repo: EntityRepository,
}
impl Orchestrator {
    delegate! {
        to self.track_repo {
            pub fn create_track(&mut self, uid: Option<TrackUid>) -> Result<TrackUid>;
            pub fn set_track_position(&mut self, uid: TrackUid, new_position: usize) -> Result<()>;
            pub fn delete_track(&mut self, uid: TrackUid) -> Result<()>;
        }
    }
    pub fn track_uids(&self) -> &[TrackUid] {
        &self.track_repo.uids
    }
    delegate! {
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
        }
        to self.entity_repo.entities {
            #[call(get_mut)]
            pub fn get_entity_mut(&mut self, uid: &Uid) -> Option<&mut Box<(dyn EntityBounds)>>;
        }
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
        let track_buffers: Vec<Vec<StereoSample>> = self
            .track_repo
            .uids
            .iter()
            .map(|track_uid| {
                let mut track_buffer = Vec::default();
                track_buffer.resize(buffer_len, StereoSample::SILENCE);
                if let Some(entity_uids) = self.entity_repo.uids_for_track.get(track_uid) {
                    entity_uids.iter().for_each(|uid| {
                        if let Some(entity) = self.entity_repo.entities.get_mut(uid) {
                            entity.generate_batch_values(&mut track_buffer);
                        }
                    });
                }
                track_buffer
            })
            .collect();
        track_buffers.iter().for_each(|buffer| {
            for (dst, src) in values.iter_mut().zip(buffer) {
                *dst += *src;
            }
        });
    }
}
impl Ticks for Orchestrator {}
impl Configurable for Orchestrator {}
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
    pub uids: Vec<TrackUid>,
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
}
impl EntityRepository {
    pub fn add_entity(
        &mut self,
        track_uid: TrackUid,
        entity: Box<dyn EntityBounds>,
        uid: Option<Uid>,
    ) -> Result<Uid> {
        let uid = if let Some(uid) = uid {
            uid
        } else {
            self.uid_factory.mint_next()
        };
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

    delegate! {
        to self.uid_factory {
            #[call(mint_next)]
            pub fn mint_entity_uid(&self) -> Uid;
        }
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
        self.entities.values().all(|e| e.is_finished())
    }

    fn play(&mut self) {
        self.entities.values_mut().for_each(|e| e.play());
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
        self.entities
            .iter_mut()
            .for_each(|(uid, e)| e.work(&mut |inner_event| control_events_fn(*uid, inner_event)));
    }
}
impl Configurable for EntityRepository {}
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
}
impl Serializable for Automator {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {}
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
}
impl Serializable for MidiRouter {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {}
}
