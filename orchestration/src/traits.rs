// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    control::ControlIndex,
    midi::MidiChannel,
    time::MusicalTime,
    traits::{Configurable, Controls, Generates},
    types::{Normal, StereoSample},
    uid::{TrackUid, Uid},
};
use ensnare_entity::traits::Entity;

/// Manages relationships among [Entities](Entity) to produce a song.
pub trait Orchestrates: Configurable + Controls + Generates<StereoSample> {
    /// Creates a new track, returning its [TrackUid] if successful. A track is
    /// a group of musical instruments that together produce a single sample for
    /// every frame of audio. Each track's frame sample is then merged into a
    /// single sample for the audio frame.
    ///
    /// The [TrackUid] should be appended to the internal list of [TrackUid]s.
    fn create_track(&mut self) -> anyhow::Result<TrackUid>;

    /// Returns an ordered list of [TrackUid]s. The ordering of tracks
    /// determines how tracks are presented in a visual rendering of the
    /// project, but it shouldn't affect how the project sounds.
    ///
    /// [TrackUid]s are generally appended to this list as they are created.
    fn track_uids(&self) -> &[TrackUid];

    /// Moves the specified [TrackUid] to the given position. Later [TrackUid]s
    /// are shifted to make room, if needed.
    fn set_track_position(
        &mut self,
        track_uid: TrackUid,
        new_position: usize,
    ) -> anyhow::Result<()>;

    /// Deletes the specified track, disposing of any [Entities](Entity) that it
    /// owns.
    fn delete_track(&mut self, track_uid: &TrackUid);

    /// Deletes the specified tracks. As with [Orchestrates::delete_track()],
    /// disposes of any owned [Entities](Entity).
    fn delete_tracks(&mut self, uids: &[TrackUid]);

    /// Adds the given [Entity] to the end of the specified track. The [Entity]
    /// must have a valid [Uid].
    fn add_entity(&mut self, track_uid: &TrackUid, entity: Box<dyn Entity>) -> anyhow::Result<()>;

    /// Assigns a new [Uid] to the given [Entity] and adds it to the end of the
    /// specified track.
    fn assign_uid_and_add_entity(
        &mut self,
        track_uid: &TrackUid,
        entity: Box<dyn Entity>,
    ) -> anyhow::Result<Uid>;

    /// Removes the specified [Entity], returning ownership (if successful) to
    /// the caller.
    fn remove_entity(&mut self, uid: &Uid) -> anyhow::Result<Box<dyn Entity>>;

    /// Moves the specified [Entity] to the end of the specified track.
    fn set_entity_track(&mut self, new_track_uid: &TrackUid, uid: &Uid) -> anyhow::Result<()>;

    /// Establishes a control link between the source [Entity]'s output and the
    /// given parameter of the target [Entity]'s.
    ///
    /// The global transport has a special [Uid] of 1, and its tempo parameter's
    /// index is zero. Therefore, it's possible to automate the global tempo by
    /// linking something with target Uid 1, control_index zero. Tempo ranges
    /// linearly from 0..=Tempo::MAX_VALUE (currently 1024), so a ControlValue
    /// of 0.125 corresponds to a Tempo of 128 BPM.
    fn link_control(
        &mut self,
        source_uid: Uid,
        target_uid: Uid,
        control_index: ControlIndex,
    ) -> anyhow::Result<()>;

    /// Removes the specified control link, if it exists.
    fn unlink_control(&mut self, source_uid: Uid, target_uid: Uid, control_index: ControlIndex);

    /// Sets the specified effect's wet/dry mix. A humidity of 1.0 is 100%
    /// effect, and 0.0 is 100% unprocessed input. Returns an error if the
    /// entity is not an effect.
    fn set_effect_humidity(&mut self, uid: Uid, humidity: Normal) -> anyhow::Result<()>;

    /// Repositions the specified effect in the track's effects chain.
    ///
    /// Note that ordering matters only for effects, not controllers or
    /// instruments. During a time slice, all controllers perform their work
    /// simultaneously, and all instruments generate signals simultaneously. But
    /// effects operate sequentially. Thus, the first effect operates on the
    /// output of the mixed instruments, and the second effect operates on the
    /// output of the first effect, and so on.
    fn set_effect_position(&mut self, uid: Uid, index: usize) -> anyhow::Result<()>;

    /// Configures a send from the given track to the given aux track. The
    /// `send_amount` parameter indicates how much signal attenuation should
    /// happen before reaching the aux: 1.0 means the full signal should reach
    /// it, and 0.0 means that none of it should.
    ///
    /// Note that send_to_aux(1, 2, Normal(0.0)) can be implemented as
    /// remove_send_to_aux(1, 2), because the behavior is identical.
    fn send(
        &mut self,
        send_track_uid: TrackUid,
        aux_track_uid: TrackUid,
        send_amount: Normal,
    ) -> anyhow::Result<()>;

    /// Removes a send configuration.
    fn remove_send(&mut self, send_track_uid: TrackUid, aux_track_uid: TrackUid);

    /// Sets the level of audio from the given track that reaches the main
    /// mixer.
    fn set_track_output(&mut self, track_uid: TrackUid, output: Normal);

    /// Sets whether the given track is muted.
    fn mute_track(&mut self, track_uid: TrackUid, muted: bool);

    /// Returns which track, if any, is soloing.
    fn solo_track(&self) -> Option<TrackUid>;

    /// Sets the current track that is soloing.
    fn set_solo_track(&mut self, track_uid: TrackUid);

    /// Ends any soloing.
    fn end_solo(&mut self);

    /// Given the number of digital audio samples that we want, returns the next
    /// slice of musical time that we should render in this performance.
    fn next_range(&mut self, sample_count: usize) -> std::ops::Range<MusicalTime>;

    /// Connect the specified Entity to the given MIDI channel.
    fn connect_midi_receiver(&mut self, uid: Uid, channel: MidiChannel) -> anyhow::Result<()>;

    /// Disconnect the specified Entity from the given MIDI channel.
    fn disconnect_midi_receiver(&mut self, uid: Uid, channel: MidiChannel);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use ensnare_entity::test_entities::TestInstrument;
    use more_asserts::assert_gt;
    use std::collections::HashSet;

    pub(crate) fn validate_orchestrates_trait(orchestrates: &mut dyn Orchestrates) {
        assert!(
            orchestrates.track_uids().is_empty(),
            "Initial impl should have no tracks"
        );
        let track_1_uid = orchestrates.create_track().unwrap();
        assert_gt!(track_1_uid.0, 0, "new track's uid should be nonzero");
        assert_eq!(
            orchestrates.track_uids().len(),
            1,
            "should be one track after creating one"
        );

        let track_2_uid = orchestrates.create_track().unwrap();
        assert_eq!(
            orchestrates.track_uids().len(),
            2,
            "should be two tracks after creating second"
        );
        assert!(orchestrates.set_track_position(track_2_uid, 0).is_ok());
        assert_eq!(
            orchestrates.track_uids(),
            vec![track_2_uid, track_1_uid],
            "order of track uids should be as expected after move"
        );
        orchestrates.delete_track(&track_2_uid);

        let target_uid = orchestrates
            .assign_uid_and_add_entity(&track_1_uid, Box::new(TestInstrument::default()))
            .unwrap();
        assert!(
            orchestrates
                .link_control(Uid(123), target_uid, ControlIndex(7))
                .is_ok(),
            "Linking control to a known target Uid should work"
        );
        orchestrates.unlink_control(Uid(234), Uid(345), ControlIndex(8));

        orchestrates.delete_track(&TrackUid(99999));
        assert_eq!(
            orchestrates.track_uids().len(),
            1,
            "Deleting nonexistent track shouldn't change anything"
        );

        let mut ids: HashSet<Uid> = HashSet::default();
        for _ in 0..64 {
            let e = Box::new(TestInstrument::default());
            let uid = orchestrates
                .assign_uid_and_add_entity(&track_1_uid, e)
                .unwrap();
            assert!(
                !ids.contains(&uid),
                "added entities should be assigned unique IDs"
            );
            ids.insert(uid);
        }

        orchestrates.delete_track(&track_1_uid);
        assert!(
            orchestrates.track_uids().is_empty(),
            "Deleting track should change track count"
        );

        assert!(
            orchestrates.solo_track().is_none(),
            "No track should be soloing at first"
        );
        orchestrates.set_solo_track(track_1_uid);
        assert_eq!(
            orchestrates.solo_track(),
            Some(track_1_uid),
            "set_solo_track() should work"
        );
        orchestrates.end_solo();
        assert!(
            orchestrates.solo_track().is_none(),
            "No track should be soloing after end_solo()"
        );
    }
}
