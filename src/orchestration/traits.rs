// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use derivative::Derivative;
use ensnare::prelude::*;

/// The [Projects] trait specifies the common behavior of an Ensnare project,
/// which is everything that makes up a single musical piece, such as the tempo,
/// the time signature, the musical notes, the tracks, and instrument layouts
/// and configurations. [Projects] is a trait because we have different project
/// implementations, depending on the use case.
///
/// Incidentally, the name "Projects" sounds awkward, but I looked up the
/// etymology of the word "project," and it originally meant "to cause to move
/// forward" in the sense of making an idea transform into reality. So saying
/// that a project projects is not totally strange.
pub trait Projects: Configurable + Controls + Sized {
    /// Generates a new [TrackUid] that is unique within this project.
    fn mint_track_uid(&self) -> TrackUid;

    /// Creates a new track and appends it to the list of tracks. Returns the
    /// generated [TrackUid] of the new track.
    fn create_track(&mut self) -> anyhow::Result<TrackUid>;

    /// Deletes the given track. Anything the track owns is dropped.
    fn delete_track(&mut self, track_uid: TrackUid) -> anyhow::Result<()>;

    /// Returns an ordered list of [TrackUid]s.
    fn track_uids(&self) -> &[TrackUid];

    /// Moves the given track to the new position in the track list. Shifts
    /// later tracks to make room if needed.
    fn set_track_position(
        &mut self,
        track_uid: TrackUid,
        new_position: usize,
    ) -> anyhow::Result<()>;

    /// Indicates whether a track is currently muted.
    fn is_track_muted(&mut self, track_uid: TrackUid) -> bool;
    /// Mutes or unmutes a track.
    fn mute_track(&mut self, track_uid: TrackUid, should_mute: bool);

    /// Returns which track is currently soloing, or None.
    ///
    /// Soloing means all but one track is muted. The solo state is independent
    /// of mute state; when soloing is enabled, muting is ignored, and when
    /// soloing is disabled, any prior mute state is reactivated.
    fn solo_track(&self) -> Option<TrackUid>;
    /// Sets or clears the single solo track.
    fn set_solo_track(&mut self, track_uid: Option<TrackUid>);

    /// Generates a new [Uid] that is unique within this project.
    fn mint_entity_uid(&self) -> Uid;

    /// Adds an entity to the end of a track and takes ownership of the entity.
    /// If the entity's [Uid] is [Uid::default()], generates a new one, setting
    /// the entity's [Uid] to match. Returns the entity's [Uid].
    ///
    /// Note that entity ordering is more complicated than a single linear list.
    /// This is because entities have different roles, and certain roles precede
    /// other roles. Some entities are controllers, others are instruments,
    /// others are effects, and some are hybrids. Instruments must generate
    /// sounds before effects process those sounds, and controllers must issue
    /// MIDI messages for instruments to generate sounds. Thus, even though a
    /// controller might be later in the entity list than an instrument, and an
    /// instrument might be later than an effect, the controller will do its
    /// work first, and then the instrument will play, and then the effect will
    /// process.
    ///
    /// However, in the case of a single role, entities are processed in the
    /// order in which they appear in the entity list. If a reverb is earlier
    /// than a delay, for example, then the reverb will be applied before the
    /// delay.
    fn add_entity(&mut self, track_uid: TrackUid, entity: Box<dyn Entity>) -> anyhow::Result<Uid>;

    /// Deletes and discards an existing entity.
    fn delete_entity(&mut self, entity_uid: Uid) -> anyhow::Result<()>;

    /// Removes an existing entity from the project and returns it to the
    /// caller.
    fn remove_entity(&mut self, entity_uid: Uid) -> anyhow::Result<Box<dyn Entity>>;

    /// Returns an ordered list of entity uids for the specified track.
    fn entity_uids(&self, track_uid: TrackUid) -> Option<&[Uid]>;

    /// Returns the [TrackUid] for the specified entity.
    fn track_for_entity(&self, uid: Uid) -> Option<TrackUid>;

    /// Moves the given entity to a new track and/or position within that track.
    /// Fails if the track doesn't exist or the position is out of bounds.
    fn move_entity(
        &mut self,
        uid: Uid,
        new_track_uid: Option<TrackUid>,
        new_position: Option<usize>,
    ) -> anyhow::Result<()>;

    /// Returns an [Iterator] that renders the project as [StereoSample]s from
    /// start to finish.
    fn render(&mut self) -> impl Iterator<Item = StereoSample> {
        self.play();
        ProjectsRenderer::new_with(self)
    }

    /// Fills the supplied buffer with [StereoSample]s that represent a portion
    /// of the project performance. Renders as of the current position set in
    /// [Controls] and advances the position appropriately. If the performance
    /// ends midway, the remainder of the buffer will be untouched.
    fn generate_audio(
        &mut self,
        frames: &mut [StereoSample],
        midi_events_fn: Option<&mut MidiMessagesFn>,
    );

    /// Generates the specified number of frames of the project and sends them
    /// to the preconfigured destination. The destination depends on the
    /// implementation of the [Projects] trait.
    fn generate_and_dispatch_audio(
        &mut self,
        count: usize,
        midi_events_fn: Option<&mut MidiMessagesFn>,
    );
}

/// Renders a [Projects] start-to-finish as a sequence of [StereoSample]s.
#[derive(Debug, Derivative)]
struct ProjectsRenderer<'a, P: Projects> {
    project: &'a mut P,
    samples: GenerationBuffer<StereoSample>,
    sample_pointer: usize,
}
impl<'a, P: Projects> ProjectsRenderer<'a, P> {
    fn new_with(project: &'a mut P) -> Self {
        Self {
            project,
            samples: GenerationBuffer::new_with(64),
            sample_pointer: usize::MAX,
        }
    }
}
impl<'a, P: Projects> Iterator for ProjectsRenderer<'a, P> {
    type Item = StereoSample;

    fn next(&mut self) -> Option<Self::Item> {
        // This catches the case where the project is zero-length. is_finished()
        // was updated during play(), and here is where we test it.
        if self.project.is_finished() {
            return None;
        }

        if self.sample_pointer >= self.samples.buffer_size() {
            self.samples.clear();
            self.project.generate_audio(self.samples.buffer_mut(), None);

            // End rendering if performance is over and silence is detected.
            if !self.project.is_performing() {
                if self.samples.buffer().iter().all(|s| s.almost_silent()) {
                    return None;
                }
            }
            self.sample_pointer = 0;
        }
        let r = self.samples.buffer()[self.sample_pointer];
        self.sample_pointer += 1;
        Some(r)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        entities::{TestAudioSource, Timer},
        orchestration::TrackUidFactory,
    };
    use anyhow::anyhow;
    use delegate::delegate;
    use std::collections::{HashMap, HashSet};

    pub(crate) fn test_trait_projects(mut p: impl Projects) {
        test_projects_uids(&mut p);
        test_projects_track_lifetime(&mut p);
        test_projects_track_signal_flow(&mut p);
        test_projects_entity_lifetime(&mut p);
        test_projects_rendering(&mut p);
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

        let track_uid_1 = p.create_track().unwrap();
        let track_uid_2 = p.create_track().unwrap();
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

        assert!(p.delete_track(track_uid_2).is_ok(), "delete_track succeeds");
        assert_eq!(p.track_uids().len(), 1);
        let track_uid_3 = p.create_track().unwrap();
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
            &vec![track_uid_3, track_uid_1],
            "set_track_position should work"
        );

        // Clean up
        let _ = p.delete_track(track_uid_1);
        let _ = p.delete_track(track_uid_3);
    }

    fn test_projects_track_signal_flow(p: &mut impl Projects) {
        assert_eq!(
            p.track_uids().len(),
            0,
            "supplied impl Projects should be clean"
        );

        let track_uid_1 = p.create_track().unwrap();
        let track_uid_2 = p.create_track().unwrap();

        assert!(
            !p.is_track_muted(track_uid_1),
            "Initial mute state should be empty"
        );
        assert!(
            !p.is_track_muted(track_uid_2),
            "Initial mute state should be empty"
        );
        assert!(
            p.solo_track().is_none(),
            "Initial solo state should be empty"
        );

        p.mute_track(track_uid_1, true);
        assert!(p.is_track_muted(track_uid_1));
        assert!(!p.is_track_muted(track_uid_2));
        assert!(
            p.solo_track().is_none(),
            "Muting shouldn't affect solo state"
        );

        p.set_solo_track(Some(track_uid_2));
        assert_eq!(p.solo_track(), Some(track_uid_2), "Soloing should work");
        assert!(
            p.is_track_muted(track_uid_1),
            "Soloing shouldn't change mute state"
        );
        assert!(
            !p.is_track_muted(track_uid_2),
            "Soloing shouldn't change mute state"
        );

        p.set_solo_track(Some(track_uid_1));
        assert_eq!(
            p.solo_track(),
            Some(track_uid_1),
            "Changing solo track should work"
        );

        p.mute_track(track_uid_1, false);
        p.mute_track(track_uid_2, true);
        assert_eq!(
            p.solo_track(),
            Some(track_uid_1),
            "Muting shouldn't change solo state"
        );

        p.set_solo_track(None);
        assert!(p.solo_track().is_none());
        assert!(
            !p.is_track_muted(track_uid_1),
            "Ending solo should return to prior mute state"
        );
        assert!(
            p.is_track_muted(track_uid_2),
            "Ending solo should return to prior mute state"
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

        let track_uid_1 = p.create_track().unwrap();

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

    fn test_projects_rendering(p: &mut impl Projects) {
        // Because rounding errors are annoying for purposes of this test, we
        // pick a sample rate that's an even multiple of the 64-byte buffer we
        // know we're using. TODO: be more precise later on.
        let prior_sample_rate = p.sample_rate();
        p.update_sample_rate(SampleRate(32768));

        assert!(
            p.render().next().is_none(),
            "A default project should render nothing"
        );

        assert!(
            p.render().next().is_none(),
            "A project should be able to render twice without exploding"
        );

        p.skip_to_start();
        assert!(
            p.render().next().is_none(),
            "A project should be able to render after seeking"
        );

        let track_uid_1 = p.create_track().unwrap();
        let _timer_uid = p
            .add_entity(
                track_uid_1,
                Box::new(Timer::new_with(Uid::default(), MusicalTime::ONE_BEAT)),
            )
            .unwrap();

        p.update_tempo(Tempo(60.0));
        let sample_rate = p.sample_rate().0;

        // Scope renderer so we can keep working with the project later in this
        // function.
        {
            let mut renderer = p.render();
            for i in 0..sample_rate {
                assert!(renderer.next().is_some(),
                        "A one-beat-long project with tempo 60 should render exactly a second of samples, but this one ended early at sample #{i}"
                );
            }
            assert!(renderer.next().is_none(),
                    "A one-beat-long project with tempo 60 should render exactly a second of samples, but this one kept going"
            );
        }

        // The trait behavior of `generate_and_dispatch_audio()` isn't
        // sufficiently specified for us to test anything. We'll call it to make
        // sure it doesn't blow up.
        p.skip_to_start();
        p.play();
        p.generate_and_dispatch_audio(64, None);

        // Restore prior sample rate
        p.update_sample_rate(prior_sample_rate);
    }

    /// [TestProject] is a harness that helps [Projects] trait development.
    #[derive(Default)]
    struct TestProject {
        track_uid_factory: TrackUidFactory,
        track_uids: Vec<TrackUid>,
        track_mute_state: HashSet<TrackUid>,
        track_solo_state: Option<TrackUid>,

        entity_uid_factory: EntityUidFactory,
        entity_uid_to_entity: HashMap<Uid, Box<dyn Entity>>,
        entity_uid_to_track_uid: HashMap<Uid, TrackUid>,
        track_uid_to_entity_uids: HashMap<TrackUid, Vec<Uid>>,

        transport: Transport,
    }
    impl Projects for TestProject {
        fn mint_track_uid(&self) -> TrackUid {
            self.track_uid_factory.mint_next()
        }

        fn create_track(&mut self) -> anyhow::Result<TrackUid> {
            let track_uid = self.mint_track_uid();
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

        fn is_track_muted(&mut self, track_uid: TrackUid) -> bool {
            self.track_mute_state.contains(&track_uid)
        }

        fn mute_track(&mut self, track_uid: TrackUid, should_mute: bool) {
            if should_mute {
                self.track_mute_state.insert(track_uid);
            } else {
                self.track_mute_state.remove(&track_uid);
            }
        }

        fn solo_track(&self) -> Option<TrackUid> {
            self.track_solo_state
        }

        fn set_solo_track(&mut self, track_uid: Option<TrackUid>) {
            self.track_solo_state = track_uid
        }

        fn mint_entity_uid(&self) -> Uid {
            self.entity_uid_factory.mint_next()
        }

        fn add_entity(
            &mut self,
            track_uid: TrackUid,
            mut entity: Box<dyn Entity>,
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

        fn remove_entity(&mut self, uid: Uid) -> anyhow::Result<Box<dyn Entity>> {
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

        fn generate_audio(
            &mut self,
            frames: &mut [StereoSample],
            mut midi_events_fn: Option<&mut MidiMessagesFn>,
        ) {
            let is_finished_at_start = self.is_finished();
            let time_range = self.transport.advance(frames.len());
            self.update_time_range(&time_range);
            self.work(&mut |e| match e {
                WorkEvent::Midi(channel, message) => {
                    if let Some(midi_events_fn) = midi_events_fn.as_mut() {
                        midi_events_fn(channel, message);
                    }
                }
                WorkEvent::MidiForTrack(_, _, _) => todo!(),
                WorkEvent::Control(_) => todo!(),
            });
            let is_finished_at_end = self.is_finished();
            if !is_finished_at_start && is_finished_at_end {
                self.stop();
            }
            if is_finished_at_end {
                frames.fill(StereoSample::SILENCE);
            } else {
                frames.fill(StereoSample::MAX);
            }
        }

        fn generate_and_dispatch_audio(
            &mut self,
            count: usize,
            mut midi_events_fn: Option<&mut MidiMessagesFn>,
        ) {
            if count == 0 {
                return;
            }
            const BUFFER_LEN: usize = 64;
            let mut buffer = [StereoSample::SILENCE; BUFFER_LEN];
            let mut remaining = count;

            while remaining != 0 {
                let to_generate = remaining.min(BUFFER_LEN);
                let buffer_slice = &mut buffer[0..to_generate];
                buffer_slice.fill(StereoSample::SILENCE);
                self.generate_audio(buffer_slice, midi_events_fn.as_deref_mut());
                remaining -= to_generate;
            }
        }
    }
    impl Configurable for TestProject {
        delegate! {
            to self.transport {
                fn sample_rate(&self) -> SampleRate;
                fn tempo(&self) -> Tempo;
                fn time_signature(&self) -> TimeSignature;
            }
        }

        fn update_sample_rate(&mut self, sample_rate: SampleRate) {
            self.transport.update_sample_rate(sample_rate);
            self.entity_uid_to_entity
                .values_mut()
                .for_each(|e| e.update_sample_rate(sample_rate));
        }

        fn update_tempo(&mut self, tempo: Tempo) {
            self.transport.update_tempo(tempo);
            self.entity_uid_to_entity
                .values_mut()
                .for_each(|e| e.update_tempo(tempo));
        }

        fn update_time_signature(&mut self, time_signature: TimeSignature) {
            self.transport.update_time_signature(time_signature);
            self.entity_uid_to_entity
                .values_mut()
                .for_each(|e| e.update_time_signature(time_signature));
        }

        fn reset(&mut self) {
            self.transport.reset();
            self.entity_uid_to_entity
                .values_mut()
                .for_each(|e| e.reset());
        }
    }
    impl Controls for TestProject {
        fn update_time_range(&mut self, time_range: &TimeRange) {
            self.transport.update_time_range(time_range);
            for entity in self.entity_uid_to_entity.values_mut() {
                entity.update_time_range(time_range);
            }
        }

        fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
            self.transport.work(control_events_fn);
            self.entity_uid_to_entity
                .values_mut()
                .for_each(|e| e.work(control_events_fn));
        }

        fn is_finished(&self) -> bool {
            self.transport.is_finished()
                && self.entity_uid_to_entity.values().all(|e| e.is_finished())
        }

        fn play(&mut self) {
            self.transport.play();
            self.entity_uid_to_entity
                .values_mut()
                .for_each(|e| e.play());
        }

        fn stop(&mut self) {
            self.transport.stop();
            self.entity_uid_to_entity
                .values_mut()
                .for_each(|e| e.stop());
        }

        fn skip_to_start(&mut self) {
            self.transport.skip_to_start();
            self.entity_uid_to_entity
                .values_mut()
                .for_each(|e| e.skip_to_start());
        }

        fn is_performing(&self) -> bool {
            self.transport.is_performing()
        }
    }

    #[test]
    fn trait_tests() {
        test_trait_projects(TestProject::default());
    }
}
