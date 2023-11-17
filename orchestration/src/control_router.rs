// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;

/// Routes automation control events.
///
/// An [Entity] that produces control events can be linked to one or more
/// surfaces of other entities. An example of an event producer is an LFO that
/// generates an audio signal, and an example of an event consumer is a synth
/// that exposes its low-pass filter cutoff as a controllable parameter. Linking
/// them means that the cutoff should follow the LFO. When the LFO's value
/// changes, the synth receives a notification of the new [ControlValue] and
/// responds by updating its cutoff.
#[derive(Debug, Default)]
pub struct ControlRouter {
    uid_to_control: std::collections::HashMap<Uid, Vec<(Uid, ControlIndex)>>,
}
impl ControlRouter {
    /// Registers a link between a source [Entity] and a controllable surface on
    /// a target [Entity].
    pub fn link_control(&mut self, source_uid: Uid, target_uid: Uid, control_index: ControlIndex) {
        self.uid_to_control
            .entry(source_uid)
            .or_default()
            .push((target_uid, control_index));
    }

    /// Removes a control link matching the source/target [Uid] and
    /// [ControlIndex].
    pub fn unlink_control(
        &mut self,
        source_uid: Uid,
        target_uid: Uid,
        control_index: ControlIndex,
    ) {
        self.uid_to_control
            .entry(source_uid)
            .or_default()
            .retain(|(uid, index)| !(*uid == target_uid && *index == control_index));
    }

    /// Returns all the control links for a given [Entity].
    pub fn control_links(&self, source_uid: Uid) -> Option<&[(Uid, ControlIndex)]> {
        match self.uid_to_control.get(&source_uid) {
            Some(r) => Some(r),
            None => None,
        }
    }

    /// Given a control event consisting of a source [Entity] and a
    /// [ControlValue], routes that event to the control surfaces linked to it.
    pub fn route(
        &self,
        entity_store_fn: &mut dyn FnMut(&Uid, ControlIndex, ControlValue),
        source_uid: Uid,
        value: ControlValue,
    ) -> anyhow::Result<()> {
        if let Some(control_links) = self.control_links(source_uid) {
            control_links.iter().for_each(|(target_uid, index)| {
                entity_store_fn(target_uid, *index, value);
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Debug, Default)]
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
    fn crud_works() {
        let mut cr = ControlRouter::default();
        assert!(
            cr.uid_to_control.is_empty(),
            "new ControlRouter should be empty"
        );

        let source_1_uid = Uid(1);
        let source_2_uid = Uid(2);
        let target_1_uid = Uid(3);
        let target_2_uid = Uid(4);

        cr.link_control(source_1_uid, target_1_uid, ControlIndex(0));
        assert_eq!(
            cr.uid_to_control.len(),
            1,
            "there should be one vec after inserting one link"
        );
        cr.link_control(source_1_uid, target_2_uid, ControlIndex(1));
        assert_eq!(
            cr.uid_to_control.len(),
            1,
            "there should still be one vec after inserting a second link for same source_uid"
        );
        cr.link_control(source_2_uid, target_1_uid, ControlIndex(0));
        assert_eq!(
            cr.uid_to_control.len(),
            2,
            "there should be two vecs after inserting one link for a second Uid"
        );

        assert_eq!(
            cr.control_links(source_1_uid).unwrap().len(),
            2,
            "the first source's vec should have two entries"
        );
        assert_eq!(
            cr.control_links(source_2_uid).unwrap().len(),
            1,
            "the second source's vec should have one entry"
        );

        let tracker = std::sync::Arc::new(std::sync::RwLock::new(Vec::default()));
        let mut controllable_1 =
            TestControllable::new_with(target_1_uid, std::sync::Arc::clone(&tracker));
        let mut controllable_2 =
            TestControllable::new_with(target_2_uid, std::sync::Arc::clone(&tracker));

        // The closures are wooden and repetitive because we don't have access
        // to EntityStore in this crate, so we hardwired a simple version of it
        // here.
        let _ = cr.route(
            &mut |target_uid, index, value| match *target_uid {
                Uid(3) => {
                    controllable_1.control_set_param_by_index(index, value);
                }
                Uid(4) => {
                    controllable_2.control_set_param_by_index(index, value);
                }
                _ => panic!("Shouldn't have received target_uid {target_uid}"),
            },
            source_1_uid,
            ControlValue(0.5),
        );
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
        cr.unlink_control(source_1_uid, target_1_uid, ControlIndex(99));
        let _ = cr.route(
            &mut |target_uid, index, value| match *target_uid {
                Uid(3) => {
                    controllable_1.control_set_param_by_index(index, value);
                }
                Uid(4) => {
                    controllable_2.control_set_param_by_index(index, value);
                }
                _ => panic!("Shouldn't have received target_uid {target_uid}"),
            },
            source_1_uid,
            ControlValue(0.5),
        );
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
        cr.unlink_control(source_1_uid, target_1_uid, ControlIndex(0));
        let _ = cr.route(
            &mut |target_uid, index, value| match *target_uid {
                Uid(3) => {
                    controllable_1.control_set_param_by_index(index, value);
                }
                Uid(4) => {
                    controllable_2.control_set_param_by_index(index, value);
                }
                _ => panic!("Shouldn't have received target_uid {target_uid}"),
            },
            source_1_uid,
            ControlValue(0.5),
        );
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
}
