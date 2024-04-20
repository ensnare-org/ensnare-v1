// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{orchestration::EntityRepository, prelude::*};
use anyhow::{anyhow, Result};
use core::fmt::Debug;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Handles the virtual connections between MIDI senders and receivers.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MidiRouter {
    /// For each channel, lists the entities that are listening on that channel.
    pub midi_receivers: FxHashMap<MidiChannel, Vec<Uid>>,

    /// This router's preferred MIDI channel. This lets tracks have a MIDI
    /// channel.
    #[serde(default)]
    midi_channel: MidiChannel,

    /// Maps each entity to a [MidiChannel].
    #[serde(skip)]
    pub uid_to_channel: FxHashMap<Uid, MidiChannel>,
}
impl MidiRouter {
    /// Causes an entity to listen on the given MIDI channel, or clears all the
    /// channels it's listening to.
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
            self.uid_to_channel.insert(entity_uid, channel);
        } else {
            self.midi_receivers
                .values_mut()
                .for_each(|receivers| receivers.retain(|receiver_uid| *receiver_uid != entity_uid));
            self.uid_to_channel.remove(&entity_uid);
        }
        Ok(())
    }

    /// Sends the given message to all the entities that are listening on the
    /// given channel.
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

    /// Sends CC 123 to every instrument, which is supposed to shut all notes
    /// off.
    pub fn all_notes_off(&mut self, entity_repo: &mut EntityRepository) {
        for channel in MidiChannel::MIN_VALUE..=MidiChannel::MAX_VALUE {
            let _ = self.route(
                entity_repo,
                MidiChannel(channel),
                MidiMessage::Controller {
                    controller: 123.into(),
                    value: 0.into(),
                },
            );
        }
    }

    /// Returns the preferred MIDI channel for this router.
    pub fn midi_channel(&self) -> MidiChannel {
        self.midi_channel
    }

    /// Sets the preferred MIDI channel for this router.
    pub fn set_midi_channel(&mut self, midi_channel: MidiChannel) {
        self.midi_channel = midi_channel;
    }
}
impl Serializable for MidiRouter {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.uid_to_channel.clear();
        for (channel, uids) in self.midi_receivers.iter() {
            for uid in uids {
                self.uid_to_channel.insert(*uid, *channel);
            }
        }
    }
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
    impl Generates<StereoSample> for TestHandlesMidi {}

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

        let m = MidiUtils::new_note_on(1, 1);

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

        let m = MidiUtils::new_note_on(1, 1);

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
        let m = MidiUtils::new_note_on(2, 3);
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

        let m = MidiUtils::new_note_on(1, 1);

        assert!(r.route(&mut repo, MidiChannel(1), m).is_err());
    }
}
