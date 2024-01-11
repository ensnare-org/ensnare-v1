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
            #[call(uids)]
            pub fn track_uids(&self) -> &[TrackUid];
            pub fn set_track_position(&mut self, uid: TrackUid, new_position: usize) -> Result<()>;
            pub fn delete_track(&mut self, uid: TrackUid) -> Result<()>;
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
        }
        to self.entity_repo.entities {
            #[call(get_mut)]
            pub fn get_entity_mut(&mut self, uid: &Uid) -> Option<&mut Box<(dyn EntityBounds)>>;
        }
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

    sample_rate: SampleRate,
    tempo: Tempo,
    time_signature: TimeSignature,
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

#[cfg(test)]
mod tests {
    use super::*;
    use ensnare_core::time::TransportBuilder;
    use ensnare_entities::instruments::{TestInstrument, TestInstrumentParams};
    use ensnare_proc_macros::{Control, IsEntity2, Metadata};
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
                Box::new(TestInstrument::new_with(
                    expected_uid,
                    &TestInstrumentParams::default(),
                )),
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
                Box::new(TestInstrument::new_with(
                    Uid(33333),
                    &TestInstrumentParams::default(),
                )),
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

    #[derive(Debug, Default, IsEntity2, Metadata, Serialize, Deserialize)]
    #[entity2(
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
        tracker: Arc<std::sync::RwLock<Vec<(Uid, ControlIndex, ControlValue)>>>,
    }
    impl TestControllable {
        pub fn new_with(
            uid: Uid,
            tracker: Arc<std::sync::RwLock<Vec<(Uid, ControlIndex, ControlValue)>>>,
        ) -> Self {
            Self { uid, tracker }
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

        let tracker = std::sync::Arc::new(std::sync::RwLock::new(Vec::default()));
        let controllable_1 =
            TestControllable::new_with(target_1_uid, std::sync::Arc::clone(&tracker));
        let controllable_2 =
            TestControllable::new_with(target_2_uid, std::sync::Arc::clone(&tracker));
        let track_uid = TrackUid(1);
        let mut repo = EntityRepository::default();
        let _ = repo.add_entity(track_uid, Box::new(controllable_1), Some(target_1_uid));
        let _ = repo.add_entity(track_uid, Box::new(controllable_2), Some(target_2_uid));

        // The closures are wooden and repetitive because we don't have access
        // to EntityStore in this crate, so we hardwired a simple version of it
        // here.
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

    #[derive(Debug, Control, Default, IsEntity2, Metadata, Serialize, Deserialize)]
    #[entity2(
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
}
