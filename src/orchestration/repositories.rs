// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::TrackUidFactory;
use crate::prelude::*;
use anyhow::{anyhow, Result};
use core::fmt::Debug;
use delegate::delegate;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TrackRepository {
    pub(crate) uid_factory: TrackUidFactory,
    pub(crate) uids: Vec<TrackUid>,
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
#[serde(rename_all = "kebab-case")]
pub struct EntityRepository {
    pub(crate) uid_factory: EntityUidFactory,
    pub(crate) entities: FxHashMap<Uid, Box<dyn EntityBounds>>,
    pub(crate) uids_for_track: FxHashMap<TrackUid, Vec<Uid>>,
    pub(crate) track_for_uid: FxHashMap<Uid, TrackUid>,

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
        to self.uid_factory.0 {
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
            self.mint_entity_uid()
        };
        entity.set_uid(uid);
        entity.update_sample_rate(self.sample_rate);
        entity.update_time_signature(self.time_signature);
        entity.update_tempo(self.tempo);
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

    pub fn uids_for_track(&self) -> &FxHashMap<TrackUid, Vec<Uid>> {
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
            // To segregate MIDI events to the track in which they were
            // generated, we record the track Uid. But we don't do the lookup
            // until we have a MIDI event to route.
            let mut track_uid = None;

            // Call each entity's Controls::work(), processing any events it
            // generates.
            e.work(&mut |inner_event| match inner_event {
                WorkEvent::Midi(channel, message) => {
                    // We have a MIDI event. Do we know the entity's track Uid?
                    if track_uid.is_none() {
                        // We don't. Let's look it up and cache it for the rest
                        // of the block, because an entity can belong to only
                        // one track.
                        track_uid = self.track_for_uid.get(uid).copied();
                    }
                    control_events_fn(
                        (*uid).into(),
                        WorkEvent::MidiForTrack(track_uid.unwrap_or_default(), channel, message),
                    );
                }
                _ => {
                    // Route other event types without further processing.
                    control_events_fn((*uid).into(), inner_event)
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::TestInstrument;
    use more_asserts::assert_gt;

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
}
