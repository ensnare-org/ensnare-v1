// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::repositories::EntityRepository;
use anyhow::{anyhow, Result};
use ensnare_core::prelude::*;
use ensnare_entity::Uid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, option::Option};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
    use ensnare_proc_macros::{Control, IsEntity, Metadata};
    use std::sync::{Arc, RwLock};

    #[derive(Debug, Control, Default, IsEntity, Metadata, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
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
        fn generate(&mut self, values: &mut [StereoSample]) {
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
}
