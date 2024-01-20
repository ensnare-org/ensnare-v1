// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{parts::EntityRepository, types::ControlLink};
use anyhow::{anyhow, Result};
use ensnare_core::{
    generators::{PathUid, SignalPath},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, option::Option};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use ensnare_proc_macros::{IsEntity, Metadata};
    use std::sync::{Arc, RwLock};

    #[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
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
}
