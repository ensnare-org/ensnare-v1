// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;

/// The [Projects] trait specifies the common behavior of an Ensnare project,
/// which is everything that makes up a single musical work -- the tempo, the
/// time signature, the musical notes, the tracks, the instrument layouts and
/// configurations, etc. [Projects] is a trait because we have different
/// implementations of project behavior, depending on the use case.
///
/// Incidentally, the name "Projects" sounds awkward, but I looked up the
/// etymology of the word "project," and it originally meant "to cause to move
/// forward" in the sense of making an idea transform into reality. So saying
/// that a project projects is not totally strange.
pub trait Projects {
    /// Creates a new track, optionally assigning the given [TrackUid]. Returns
    /// the [TrackUid] of the new track. Specified [TrackUid]s must not
    /// duplicate one that already exists in the project.
    fn create_track(&mut self, uid: Option<TrackUid>) -> anyhow::Result<TrackUid>;

    /// Deletes the given track. If the track owns anything, they're dropped.
    fn delete_track(&mut self, uid: TrackUid) -> anyhow::Result<()>;

    /// Adds an entity to a track and takes ownership of the entity. If the
    /// entity's [Uid] is [Uid::default()], generates a new one, setting the
    /// entity's [Uid] to match. Returns the entity's [Uid].
    fn add_entity(
        &mut self,
        track_uid: TrackUid,
        entity: Box<dyn EntityBounds>,
    ) -> anyhow::Result<Uid>;

    /// Deletes and discards an existing entity.
    fn delete_entity(&mut self, uid: Uid) -> anyhow::Result<()>;

    /// Removes an existing entity from the project and returns it to the
    /// caller.
    fn remove_entity(&mut self, uid: Uid) -> anyhow::Result<Box<dyn EntityBounds>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{entities::TestAudioSource, orchestration::TrackUidFactory};
    use anyhow::anyhow;
    use std::collections::{HashMap, HashSet};

    /// [TestProject] is a harness that helps make the [Projects] trait
    /// ergonomic.
    #[derive(Default)]
    struct TestProject {
        track_uid_factory: TrackUidFactory,
        track_uids: HashSet<TrackUid>,

        entity_uid_factory: EntityUidFactory,
        entity_uids_to_entities: HashMap<Uid, Box<dyn EntityBounds>>,
        entity_uids_to_track_uids: HashMap<Uid, TrackUid>,
        track_uids_to_entity_uids: HashMap<TrackUid, Vec<Uid>>,
    }
    impl Projects for TestProject {
        fn create_track(&mut self, uid: Option<TrackUid>) -> anyhow::Result<TrackUid> {
            let uid = if let Some(uid) = uid {
                if self.track_uids.contains(&uid) {
                    return Err(anyhow!("Duplicate TrackUid"));
                }
                uid
            } else {
                self.track_uid_factory.mint_next()
            };

            self.track_uids.insert(uid);
            Ok(uid)
        }

        fn delete_track(&mut self, uid: TrackUid) -> anyhow::Result<()> {
            self.track_uids.remove(&uid);
            Ok(())
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
                let uid = self.entity_uid_factory.mint_next();
                entity.set_uid(uid);
                uid
            };
            self.entity_uids_to_entities.insert(uid.clone(), entity);
            self.entity_uids_to_track_uids
                .insert(uid.clone(), track_uid);
            self.track_uids_to_entity_uids
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
            if let Some(track_uid) = self.entity_uids_to_track_uids.remove(&uid) {
                if let Some(entities) = self.track_uids_to_entity_uids.get_mut(&track_uid) {
                    entities.retain(|e| *e == uid);
                }
            }
            if let Some(entity) = self.entity_uids_to_entities.remove(&uid) {
                Ok(entity)
            } else {
                Err(anyhow!("No such entity {uid}"))
            }
        }
    }

    #[test]
    fn trait_track_lifetime() {
        let mut p = TestProject::default();
        let track_uid_1 = p.create_track(None).unwrap();
        let track_uid_2 = p.create_track(None).unwrap();
        assert_ne!(
            track_uid_1, track_uid_2,
            "create_track should generate unique IDs"
        );

        assert!(
            p.create_track(Some(track_uid_1)).is_err(),
            "create_track should disallow assignment of duplicate uids."
        );

        assert!(p.delete_track(track_uid_1).is_ok(), "delete_track succeeds");
        assert!(
            p.create_track(Some(track_uid_1)).is_ok(),
            "delete_track works (should be able to create a track having same TrackUid of a just-deleted track)."
        );
    }

    #[test]
    fn trait_entity_lifetime() {
        let mut p = TestProject::default();
        let track_uid_1 = p.create_track(None).unwrap();

        let e_uid_1 = p
            .add_entity(track_uid_1, Box::new(TestAudioSource::default()))
            .unwrap();
        let entity = p.remove_entity(e_uid_1).unwrap();
        assert_eq!(
            entity.uid(),
            e_uid_1,
            "remove_entity returns the same entity we added (and add_entity fixed up the uid)"
        );

        let e_uid_2 = p
            .add_entity(track_uid_1, Box::new(TestAudioSource::default()))
            .unwrap();
        assert!(p.delete_entity(e_uid_2).is_ok());
        assert!(
            p.remove_entity(e_uid_2).is_err(),
            "removing an entity after deleting it should fail"
        );
    }
}
