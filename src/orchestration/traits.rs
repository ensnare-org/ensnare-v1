// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;

/// The [Projects] trait specifies the common behavior of an Ensnare project,
/// which is everything that makes up a single musical piece, such as the tempo,
/// the time signature, the musical notes, the tracks, and instrument layouts
/// and configurations. [Projects] is a trait because we have different
/// implementations of project behavior, depending on the use case.
///
/// Incidentally, the name "Projects" sounds awkward, but I looked up the
/// etymology of the word "project," and it originally meant "to cause to move
/// forward" in the sense of making an idea transform into reality. So saying
/// that a project projects is not totally strange.
pub trait Projects: Configurable {
    /// Generates a new, unique [TrackUid]. Uniqueness is guaranteed only within
    /// this project.
    fn mint_track_uid(&self) -> TrackUid;

    /// Creates a new track, optionally assigning the given [TrackUid]. Returns
    /// the [TrackUid] of the new track. Specified [TrackUid]s must not
    /// duplicate one that already exists in the project.
    fn create_track(&mut self, track_uid: Option<TrackUid>) -> anyhow::Result<TrackUid>;

    /// Deletes the given track. If the track owns anything, they're dropped.
    fn delete_track(&mut self, track_uid: TrackUid) -> anyhow::Result<()>;

    /// Returns an ordered list of [TrackUid]s.
    fn track_uids(&self) -> &[TrackUid];

    /// Moves the given track to the new position, shifting later tracks to make
    /// room if needed.
    fn set_track_position(
        &mut self,
        track_uid: TrackUid,
        new_position: usize,
    ) -> anyhow::Result<()>;

    /// Generates a new, unique [TrackUid].
    fn mint_entity_uid(&self) -> Uid;

    /// Adds an entity to a track and takes ownership of the entity. If the
    /// entity's [Uid] is [Uid::default()], generates a new one, setting the
    /// entity's [Uid] to match. Returns the entity's [Uid].
    fn add_entity(
        &mut self,
        track_uid: TrackUid,
        entity: Box<dyn EntityBounds>,
    ) -> anyhow::Result<Uid>;

    /// Deletes and discards an existing entity.
    fn delete_entity(&mut self, entity_uid: Uid) -> anyhow::Result<()>;

    /// Removes an existing entity from the project and returns it to the
    /// caller.
    fn remove_entity(&mut self, entity_uid: Uid) -> anyhow::Result<Box<dyn EntityBounds>>;

    /// Returns an ordered list of entity uids for the specified track.
    fn entity_uids(&self, track_uid: TrackUid) -> Option<&[Uid]>;

    /// Returns the [TrackUid] for the specified entity.
    fn track_for_entity(&self, uid: Uid) -> Option<TrackUid>;

    /// Moves the given entity to a new track and/or position within that track.
    fn move_entity(
        &mut self,
        uid: Uid,
        new_track_uid: Option<TrackUid>,
        new_position: Option<usize>,
    ) -> anyhow::Result<()>;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{entities::TestAudioSource, orchestration::TrackUidFactory};
    use anyhow::anyhow;
    use std::collections::HashMap;

    pub(crate) fn test_trait_projects(mut p: impl Projects) {
        test_projects_uids(&mut p);
        test_projects_track_lifetime(&mut p);
        test_projects_entity_lifetime(&mut p);
    }

    fn test_projects_uids(p: &mut impl Projects) {
        assert_ne!(
            p.mint_track_uid(),
            p.mint_track_uid(),
            "Minted TrackUids should be unique"
        );
        assert_ne!(
            p.mint_entity_uid(),
            p.mint_entity_uid(),
            "Minted Uids should be unique"
        );
    }

    fn test_projects_track_lifetime(p: &mut impl Projects) {
        assert_eq!(
            p.track_uids().len(),
            0,
            "supplied impl Projects should be clean"
        );

        let track_uid_1 = p.create_track(None).unwrap();
        let track_uid_2 = p.create_track(None).unwrap();
        assert_ne!(
            track_uid_1, track_uid_2,
            "create_track should generate unique IDs"
        );
        assert_eq!(p.track_uids().len(), 2);
        assert_eq!(
            p.track_uids(),
            &vec![track_uid_1, track_uid_2],
            "track ordering is same order as track creation"
        );

        assert!(
            p.create_track(Some(track_uid_2)).is_err(),
            "create_track should disallow assignment of duplicate uids."
        );

        assert!(p.delete_track(track_uid_2).is_ok(), "delete_track succeeds");
        assert_eq!(p.track_uids().len(), 1);
        assert!(
            p.create_track(Some(track_uid_2)).is_ok(),
            "delete_track works (should be able to create a track having same TrackUid of a just-deleted track)."
        );
        assert_eq!(p.track_uids().len(), 2);

        assert!(
            p.set_track_position(track_uid_1, 999).is_err(),
            "set_track_position should disallow invalid positions"
        );
        assert!(
            p.set_track_position(track_uid_1, 1).is_ok(),
            "set_track_position should allow valid positions"
        );
        assert_eq!(
            p.track_uids(),
            &vec![track_uid_2, track_uid_1],
            "set_track_position should work"
        );

        // Clean up
        let _ = p.delete_track(track_uid_1);
        let _ = p.delete_track(track_uid_2);
    }

    fn test_projects_entity_lifetime(p: &mut impl Projects) {
        assert_eq!(
            p.track_uids().len(),
            0,
            "supplied impl Projects should be clean"
        );

        let track_uid_1 = p.create_track(None).unwrap();

        let e_uid_1 = p
            .add_entity(track_uid_1, Box::new(TestAudioSource::default()))
            .unwrap();
        assert_eq!(p.track_for_entity(e_uid_1).unwrap(), track_uid_1);
        let entity = p.remove_entity(e_uid_1).unwrap();
        assert_eq!(
            entity.uid(),
            e_uid_1,
            "remove_entity returns the same entity we added (and add_entity fixed up the uid)"
        );
        assert!(p.track_for_entity(e_uid_1).is_none());
        assert_eq!(p.entity_uids(track_uid_1).unwrap().len(), 0);

        let e_uid_2 = p
            .add_entity(track_uid_1, Box::new(TestAudioSource::default()))
            .unwrap();
        assert!(p.delete_entity(e_uid_2).is_ok());
        assert!(
            p.remove_entity(e_uid_2).is_err(),
            "removing an entity after deleting it should fail"
        );
        assert_eq!(p.entity_uids(track_uid_1).unwrap().len(), 0);

        let e_uid_3 = p
            .add_entity(track_uid_1, Box::new(TestAudioSource::default()))
            .unwrap();
        let e_uid_4 = p
            .add_entity(track_uid_1, Box::new(TestAudioSource::default()))
            .unwrap();
        assert_eq!(p.entity_uids(track_uid_1).unwrap().len(), 2);

        assert_eq!(
            p.entity_uids(track_uid_1).unwrap(),
            &vec![e_uid_3, e_uid_4],
            "add_entity adds in order"
        );
        assert!(
            p.move_entity(e_uid_3, None, Some(999)).is_err(),
            "out of bounds move_entity fails"
        );
        p.move_entity(e_uid_3, None, Some(1)).unwrap();
        assert_eq!(
            p.entity_uids(track_uid_1).unwrap(),
            &vec![e_uid_4, e_uid_3],
            "move_entity works"
        );

        // Clean up
        let _ = p.delete_track(track_uid_1);
    }

    /// [TestProject] is a harness that helps make the [Projects] trait
    /// ergonomic.
    #[derive(Default)]
    struct TestProject {
        track_uid_factory: TrackUidFactory,
        track_uids: Vec<TrackUid>,

        entity_uid_factory: EntityUidFactory,
        entity_uid_to_entity: HashMap<Uid, Box<dyn EntityBounds>>,
        entity_uid_to_track_uid: HashMap<Uid, TrackUid>,
        track_uid_to_entity_uids: HashMap<TrackUid, Vec<Uid>>,
    }
    impl Projects for TestProject {
        fn create_track(&mut self, track_uid: Option<TrackUid>) -> anyhow::Result<TrackUid> {
            let track_uid = if let Some(track_uid) = track_uid {
                if self.track_uids.contains(&track_uid) {
                    return Err(anyhow!("Duplicate TrackUid"));
                }
                track_uid
            } else {
                self.mint_track_uid()
            };

            self.track_uids.push(track_uid);
            Ok(track_uid)
        }

        fn delete_track(&mut self, track_uid: TrackUid) -> anyhow::Result<()> {
            if let Some(uids) = self.track_uid_to_entity_uids.get(&track_uid) {
                for uid in uids.clone() {
                    let _ = self.delete_entity(uid);
                }
            }
            let _ = self.track_uid_to_entity_uids.remove(&track_uid);
            self.track_uids.retain(|tuid| &track_uid != tuid);
            Ok(())
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
                if new_position <= self.track_uids.len() {
                    self.delete_track(track_uid)?;
                    self.track_uids.insert(new_position, track_uid);
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Invalid track position {new_position} for {track_uid}"
                    ))
                }
            } else {
                Err(anyhow!("Track {track_uid} not found"))
            }
        }

        fn add_entity(
            &mut self,
            track_uid: TrackUid,
            mut entity: Box<dyn EntityBounds>,
        ) -> anyhow::Result<Uid> {
            if !self.track_uids.contains(&track_uid) {
                return Err(anyhow!("Nonexistent track {track_uid}"));
            }
            let uid = if entity.uid() != Uid::default() {
                entity.uid()
            } else {
                let uid = self.mint_entity_uid();
                entity.set_uid(uid);
                uid
            };
            self.entity_uid_to_entity.insert(uid.clone(), entity);
            self.entity_uid_to_track_uid.insert(uid.clone(), track_uid);
            self.track_uid_to_entity_uids
                .entry(track_uid)
                .or_default()
                .push(uid.clone());
            Ok(uid)
        }

        fn delete_entity(&mut self, uid: Uid) -> anyhow::Result<()> {
            let _ = self.remove_entity(uid);
            Ok(())
        }

        fn remove_entity(&mut self, uid: Uid) -> anyhow::Result<Box<dyn EntityBounds>> {
            if let Some(track_uid) = self.entity_uid_to_track_uid.remove(&uid) {
                if let Some(entities) = self.track_uid_to_entity_uids.get_mut(&track_uid) {
                    entities.retain(|e| *e != uid);
                }
            }
            if let Some(entity) = self.entity_uid_to_entity.remove(&uid) {
                Ok(entity)
            } else {
                Err(anyhow!("No such entity {uid}"))
            }
        }

        fn mint_track_uid(&self) -> TrackUid {
            self.track_uid_factory.mint_next()
        }

        fn mint_entity_uid(&self) -> Uid {
            self.entity_uid_factory.mint_next()
        }

        fn move_entity(
            &mut self,
            uid: Uid,
            new_track_uid: Option<TrackUid>,
            new_position: Option<usize>,
        ) -> anyhow::Result<()> {
            if !self.entity_uid_to_track_uid.contains_key(&uid) {
                return Err(anyhow!("Entity {uid} not found"));
            }
            if let Some(new_track_uid) = new_track_uid {
                if let Some(old_track_uid) = self.entity_uid_to_track_uid.get(&uid) {
                    if *old_track_uid != new_track_uid {
                        if let Some(uids) = self.track_uid_to_entity_uids.get_mut(old_track_uid) {
                            uids.retain(|u| *u != uid);
                            self.track_uid_to_entity_uids
                                .entry(new_track_uid)
                                .or_default()
                                .push(uid);
                        }
                    }
                }
                self.entity_uid_to_track_uid.insert(uid, new_track_uid);
            }
            if let Some(new_position) = new_position {
                if let Some(track_uid) = self.entity_uid_to_track_uid.get(&uid) {
                    let uids = self.track_uid_to_entity_uids.entry(*track_uid).or_default();
                    if new_position <= uids.len() {
                        uids.retain(|u| *u != uid);
                        uids.insert(new_position, uid);
                    } else {
                        return Err(anyhow!("new position {new_position} is out of bounds"));
                    }
                }
            }
            Ok(())
        }

        fn entity_uids(&self, track_uid: TrackUid) -> Option<&[Uid]> {
            if let Some(uids) = self.track_uid_to_entity_uids.get(&track_uid) {
                let uids: &[Uid] = uids;
                Some(uids)
            } else {
                None
            }
        }

        fn track_for_entity(&self, uid: Uid) -> Option<TrackUid> {
            self.entity_uid_to_track_uid.get(&uid).copied()
        }
    }
    impl Configurable for TestProject {}

    #[test]
    fn trait_tests() {
        test_trait_projects(TestProject::default());
    }
}
