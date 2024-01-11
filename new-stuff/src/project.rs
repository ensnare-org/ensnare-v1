// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Representation of a whole music project, including support for serialization.

use crate::parts::{Automator, MidiRouter, Orchestrator};
use anyhow::{anyhow, Result};
use delegate::delegate;
use eframe::egui::Id;
use ensnare_core::{
    piano_roll::{Pattern, PatternUid},
    prelude::*,
    time::Transport,
    traits::ControlsAsProxy,
    types::{AudioQueue, TrackTitle, VisualizationQueue},
};
use ensnare_cores::Composer;
use ensnare_cores_egui::composer;
use ensnare_entity::traits::EntityBounds;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// The most commonly used imports.
pub mod prelude {
    pub use super::ProjectTitle;
}

/// A user-visible project title.
#[derive(Clone, Debug, derive_more::Display, PartialEq, Serialize, Deserialize)]
pub struct ProjectTitle(String);
impl Default for ProjectTitle {
    fn default() -> Self {
        Self("Untitled".to_string())
    }
}
impl From<ProjectTitle> for String {
    fn from(value: ProjectTitle) -> Self {
        value.0
    }
}
impl From<&str> for ProjectTitle {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// A musical piece. Also knows how to render the piece to digital audio.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Project {
    pub title: ProjectTitle,
    pub track_titles: HashMap<TrackUid, TrackTitle>,
    pub track_to_frontmost_timeline_displayer: HashMap<TrackUid, Uid>,

    pub transport: Transport,
    pub orchestrator: Orchestrator,
    pub automator: Automator,
    pub composer: Composer,
    pub track_to_midi_router: HashMap<TrackUid, MidiRouter>,

    // The part of the project to show in the UI. We've made the explicit
    // decision to make view parameters persistent, so that a loaded project
    // looks the same as when it was saved. The downside is that saving a
    // project after browsing it will usually generate diffs. TODO: maybe put
    // these kinds of items into a "ProjectViewState" struct, and don't mark the
    // project dirty when those change. Then they'll be saved only along with
    // substantial changes.
    pub view_range: ViewRange,

    #[serde(skip)]
    is_finished: bool,

    /// If present, then this is the path that was used to load this project
    /// from disk.
    #[serde(skip)]
    pub load_path: Option<PathBuf>,

    #[serde(skip)]
    pub audio_queue: Option<AudioQueue>,

    /// A non-owned VecDeque that acts as a ring buffer of the most recent
    /// generated audio frames.
    #[serde(skip)]
    pub visualization_queue: Option<VisualizationQueue>,
}
impl Project {
    /// The fixed [Uid] for the project's Orchestrator.
    pub const ORCHESTRATOR_UID: Uid = Uid(1);

    /// The fixed [Uid] for the project's [Transport].
    pub const TRANSPORT_UID: Uid = Uid(2);

    /// Starts with a default project and configures for easy first use.
    pub fn new_project() -> Self {
        let mut r = Self::default();
        let _ = r.create_starter_tracks();

        // hack - default to a 3-minute song
        r.view_range = ViewRange(
            MusicalTime::START..MusicalTime::new_with_beats((r.transport.tempo().0 * 3.0) as usize),
        );
        r
    }

    delegate! {
        to self.orchestrator.entity_repo {
            pub fn mint_entity_uid(&self) -> Uid;
        }
        to self.orchestrator.track_repo {
            pub fn mint_track_uid(&self) -> TrackUid;
        }
    }

    pub fn temp_insert_16_random_patterns(&mut self) -> anyhow::Result<()> {
        self.composer.insert_16_random_patterns();
        Ok(())
    }

    /// Adds a set of tracks that make sense for a new project.
    pub fn create_starter_tracks(&mut self) -> anyhow::Result<()> {
        if !self.orchestrator.track_repo.uids().is_empty() {
            return Err(anyhow!("Must be invoked on an empty project."));
        }

        self.new_midi_track()?;
        self.new_midi_track()?;
        self.new_audio_track()?;
        self.new_aux_track()?;

        Ok(())
    }

    /// Adds a new MIDI track, which can contain controllers, instruments, and
    /// effects. Returns the new track's [TrackUid] if successful.
    pub fn new_midi_track(&mut self) -> anyhow::Result<TrackUid> {
        let track_uid = self.create_track(None)?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("MIDI {}", track_uid)));
        Ok(track_uid)
    }

    /// Adds a new audio track, which can contain audio clips and effects.
    /// Returns the new track's [TrackUid] if successful.
    pub fn new_audio_track(&mut self) -> anyhow::Result<TrackUid> {
        let track_uid = self.create_track(None)?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("Audio {}", track_uid)));
        Ok(track_uid)
    }

    /// Adds a new aux track, which contains only effects, and to which other
    /// tracks can *send* their output audio. Returns the new track's [TrackUid]
    /// if successful.
    pub fn new_aux_track(&mut self) -> anyhow::Result<TrackUid> {
        let track_uid = self.create_track(None)?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("Aux {}", track_uid)));
        Ok(track_uid)
    }

    delegate! {
        to self.orchestrator {
            pub fn track_uids(&self) -> &[TrackUid];
            pub fn set_track_position(&mut self, uid: TrackUid, new_position: usize) -> Result<()>;

            pub fn add_entity(&mut self, track_uid: TrackUid, entity: Box<dyn EntityBounds>, uid: Option<Uid>) -> Result<Uid>;
            pub fn move_entity(&mut self, uid: Uid, new_track_uid: Option<TrackUid>, new_position: Option<usize>)-> Result<()>;
            pub fn delete_entity(&mut self, uid: Uid) -> Result<()>;
            pub fn remove_entity(&mut self, uid: Uid) -> Result<Box<dyn EntityBounds>>;
        }
        to self.composer {
            pub fn add_pattern(&mut self, contents: Pattern, pattern_uid: Option<PatternUid>) -> Result<PatternUid>;
            pub fn pattern(&self, pattern_uid: &PatternUid) -> Option<&Pattern>;
            pub fn pattern_mut(&mut self, pattern_uid: &PatternUid) -> Option<&mut Pattern>;
            pub fn notify_pattern_change(&mut self);
            pub fn remove_pattern(&mut self, pattern_uid: PatternUid) -> Result<Pattern>;
            pub fn arrange_pattern(&mut self, track_uid: &TrackUid, pattern_uid: &PatternUid, position: MusicalTime) -> Result<()>;
            pub fn unarrange_pattern(&mut self, track_uid: &TrackUid, pattern_uid: &PatternUid, position: MusicalTime);
        }
        to self.automator {
            pub fn link(&mut self, source: Uid, target: Uid, param: ControlIndex) -> Result<()>;
            pub fn unlink(&mut self, source: Uid, target: Uid, param: ControlIndex);
        }
    }

    pub fn create_track(&mut self, uid: Option<TrackUid>) -> Result<TrackUid> {
        let track_uid = self.orchestrator.create_track(uid)?;
        self.track_to_midi_router
            .insert(track_uid, MidiRouter::default());
        Ok(track_uid)
    }

    pub fn delete_track(&mut self, uid: TrackUid) -> Result<()> {
        self.track_to_midi_router.remove(&uid);
        self.orchestrator.delete_track(uid)
    }

    pub fn set_midi_receiver_channel(
        &mut self,
        entity_uid: Uid,
        channel: Option<MidiChannel>,
    ) -> Result<()> {
        if let Some(track_uid) = self.orchestrator.track_for_entity(entity_uid) {
            if let Some(midi_router) = self.track_to_midi_router.get_mut(&track_uid) {
                midi_router.set_midi_receiver_channel(entity_uid, channel)
            } else {
                Err(anyhow!(
                    "set_midi_receiver_channel: no MidiRouter found for track {track_uid}"
                ))
            }
        } else {
            Err(anyhow!(
                "set_midi_receiver_channel: no track found for entity {entity_uid}"
            ))
        }
    }

    fn generate_frames(&mut self, frames: &mut [StereoSample]) {
        let time_range = self.transport.advance(frames.len());
        self.update_time_range(&time_range);
        self.work(&mut |_| {});
        if self.is_finished {
            self.stop();
        }
        self.generate_batch_values(frames);
    }

    fn update_is_finished(&mut self) {
        self.is_finished = self.composer.is_finished() && self.orchestrator.is_finished();
    }

    pub fn export_to_wav(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: self.sample_rate().into(),
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;

        self.play();
        while self.is_performing() {
            let mut samples = [StereoSample::SILENCE; 64];
            self.generate_frames(&mut samples);
            for sample in samples {
                let (left, right) = sample.into_i16();
                let _ = writer.write_sample(left);
                let _ = writer.write_sample(right);
            }
        }
        Ok(())
    }

    pub fn fill_audio_queue(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let mut buffer = [StereoSample::SILENCE; 64];
        let buffer_len = buffer.len();
        let mut remaining = count;

        while remaining != 0 {
            let to_generate = if remaining >= buffer_len {
                buffer_len
            } else {
                remaining
            };
            let buffer_slice = &mut buffer[0..to_generate];
            buffer_slice.fill(StereoSample::SILENCE);
            self.generate_frames(buffer_slice);
            if let Some(audio_queue) = self.audio_queue.as_ref() {
                buffer_slice.iter().for_each(|s| {
                    if let Some(_old_element) = audio_queue.force_push(*s) {
                        eprintln!("overrun! requested {count} frames");

                        // There is no point in continuing.
                        return;
                    }
                });
            }
            if let Some(queue) = self.visualization_queue.as_ref() {
                if let Ok(mut queue) = queue.0.write() {
                    buffer_slice.iter().for_each(|s| {
                        let mono_sample: Sample = (*s).into();
                        queue.push_back(mono_sample);
                    });
                }
            }
            remaining -= to_generate;
        }
    }

    fn dispatch_control_event(&mut self, uid: Uid, value: ControlValue) {
        self.automator.route(
            &mut self.orchestrator.entity_repo,
            Some(&mut |link| match link.uid {
                Self::TRANSPORT_UID => self.transport.control_set_param_by_index(link.param, value),
                _ => {
                    eprintln!("Asked to route unknown uid {uid}");
                }
            }),
            uid,
            value,
        );
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(&path)?;
        let mut project = serde_json::from_str::<Self>(&json)?;
        project.load_path = Some(path);
        project.after_deser();
        Ok(project)
    }

    pub fn save(&self, path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
        let save_path = {
            if let Some(path) = path.as_ref() {
                path.clone()
            } else if let Some(path) = self.load_path.as_ref() {
                path.clone()
            } else {
                PathBuf::from("ensnare-project.json")
            }
        };

        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(&save_path, json)?;
        Ok(save_path)
    }

    pub fn load_path(&self) -> Option<&PathBuf> {
        self.load_path.as_ref()
    }

    pub fn track_frontmost_timeline_displayer(&self, track_uid: TrackUid) -> Option<Uid> {
        self.track_to_frontmost_timeline_displayer
            .get(&track_uid)
            .copied()
    }

    pub fn ui_piano_roll(&mut self, ui: &mut eframe::egui::Ui, is_visible: &mut bool) {
        eframe::egui::Window::new("Piano Roll")
            .open(is_visible)
            .default_width(ui.available_width())
            .anchor(
                eframe::emath::Align2::LEFT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                let response = ui.add(composer(&mut self.composer));
                if response.changed() {
                    //self.sequence_repository.write().unwrap().notify_change();
                }
                response
            });
    }

    pub fn ui_detail(
        &mut self,
        ui: &mut eframe::egui::Ui,
        uid: Option<Uid>,
        title: &str,
        is_visible: &mut bool,
    ) {
        eframe::egui::Window::new(title)
            .id(Id::new("Entity Detail"))
            .open(is_visible)
            .anchor(
                eframe::emath::Align2::RIGHT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                if let Some(uid) = uid {
                    if let Some(entity) = self.orchestrator.entity_repo.entity_mut(uid) {
                        entity.ui(ui);
                    }
                }
            });
    }

    pub fn set_up_successor(&self, new_project: &mut Self) {
        if let Some(queue) = self.audio_queue.as_ref() {
            new_project.audio_queue = Some(Arc::clone(queue));
        }
        if let Some(queue) = self.visualization_queue.as_ref() {
            new_project.visualization_queue = Some(queue.clone());
        }
    }
}
impl Generates<StereoSample> for Project {
    delegate! {
        to self.orchestrator {
            fn generate_batch_values(&mut self, values: &mut [StereoSample]);
        }
    }
}
impl Ticks for Project {}
impl Configurable for Project {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.transport.update_sample_rate(sample_rate);
        self.orchestrator.update_sample_rate(sample_rate);
    }
    fn update_tempo(&mut self, tempo: Tempo) {
        self.transport.update_tempo(tempo);
        self.orchestrator.update_tempo(tempo);
    }
    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.transport.update_time_signature(time_signature);
        self.orchestrator.update_time_signature(time_signature);
    }

    delegate! {
        to self.transport {
            fn sample_rate(&self) -> SampleRate;
            fn tempo(&self) -> Tempo;
            fn time_signature(&self) -> TimeSignature;
        }
    }
}
impl Controls for Project {
    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn play(&mut self) {
        self.transport.play();
        self.orchestrator.play();
        self.composer.play();
        self.update_is_finished();
    }

    fn stop(&mut self) {
        self.transport.stop();
        self.orchestrator.stop();
        self.composer.stop();
    }

    fn skip_to_start(&mut self) {
        self.transport.skip_to_start();
        self.orchestrator.skip_to_start();
        self.composer.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.transport.is_performing()
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.orchestrator.update_time_range(time_range);
        self.composer.update_time_range(time_range);
    }

    fn work(&mut self, _: &mut ControlEventsFn) {
        let mut events = Vec::default();
        self.composer
            .work(&mut |event| events.push((Uid::default(), event)));
        self.orchestrator
            .work_as_proxy(&mut |uid, event| events.push((uid, event)));
        while let Some((uid, event)) = events.pop() {
            match event {
                WorkEvent::Midi(channel, message) => {
                    if let Some(track_uid) = self.orchestrator.track_for_entity(uid) {
                        if let Some(midi_router) = self.track_to_midi_router.get(&track_uid) {
                            let _ = midi_router.route(
                                &mut self.orchestrator.entity_repo,
                                channel,
                                message,
                            );
                        }
                    }
                }
                WorkEvent::Control(value) => {
                    self.dispatch_control_event(uid, value);
                }
            }
        }
        self.update_is_finished();
    }
}
impl HandlesMidi for Project {
    // This method handles only external MIDI messages, which potentially go to
    // every track.
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        _midi_messages_fn: &mut MidiMessagesFn,
    ) {
        self.track_to_midi_router
            .values_mut()
            .for_each(|midi_router| {
                let _ = midi_router.route(&mut self.orchestrator.entity_repo, channel, message);
            })
    }
}
impl Serializable for Project {
    fn before_ser(&mut self) {
        self.automator.before_ser();
        self.orchestrator.before_ser();
        self.composer.before_ser();
        self.track_to_midi_router
            .values_mut()
            .for_each(|midi_router| {
                let _ = midi_router.before_ser();
            })
    }

    fn after_deser(&mut self) {
        self.automator.after_deser();
        self.orchestrator.after_deser();
        self.composer.after_deser();
        self.track_to_midi_router
            .values_mut()
            .for_each(|midi_router| {
                let _ = midi_router.after_deser();
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ensnare_entities::instruments::TestInstrumentCountsMidiMessages;
    use ensnare_entities_toy::controllers::ToyControllerAlwaysSendsMidiMessage;
    use ensnare_proc_macros::{IsEntity2, Metadata};

    trait TestEntity: EntityBounds {}

    /// An [IsEntity2] that sends one Control event each time work() is called.
    #[derive(Debug, Default, IsEntity2, Metadata, Serialize, Deserialize)]
    #[entity2(
        Configurable,
        Controllable,
        Displays,
        GeneratesStereoSample,
        HandlesMidi,
        Serializable,
        SkipInner,
        Ticks,
        TransformsAudio
    )]
    pub struct TestControllerSendsOneEvent {
        uid: Uid,
    }
    impl Controls for TestControllerSendsOneEvent {
        fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
            control_events_fn(WorkEvent::Control(ControlValue::MAX));
        }
    }
    impl TestEntity for TestControllerSendsOneEvent {}

    #[test]
    fn project_basics() {
        let mut project = Project::default();

        assert!(
            project.sample_rate().0 != 0,
            "Default sample rate should be reasonable"
        );
        let new_sample_rate = SampleRate(3);
        project.update_sample_rate(new_sample_rate);
        assert_eq!(
            project.sample_rate(),
            new_sample_rate,
            "Sample rate should be settable"
        );

        assert!(
            project.tempo().0 > 0.0,
            "Default tempo should be reasonable"
        );
        let new_tempo = Tempo(64.0);
        project.update_tempo(new_tempo);
        assert_eq!(project.tempo(), new_tempo, "Tempo should be settable");
    }

    #[test]
    fn project_starter_tracks() {
        let mut project = Project::default();
        assert!(project.track_uids().is_empty());
        assert!(project.create_starter_tracks().is_ok());
        assert!(!project.track_uids().is_empty());
        assert!(project.create_starter_tracks().is_err());

        assert_eq!(
            project.track_uids().len(),
            4,
            "we should have four tracks after create_starter_tracks()."
        );
    }

    #[test]
    fn track_discovery() {
        let mut project = Project::default();
        assert!(project.create_starter_tracks().is_ok());
        project
            .track_uids()
            .iter()
            .for_each(|uid| assert!(project.track_titles.get(uid).is_some()));
    }

    #[test]
    fn track_crud() {
        let mut project = Project::default();
        assert_eq!(project.track_uids().len(), 0);
        let track_uid = project.new_midi_track().unwrap();
        assert_eq!(project.track_uids().len(), 1);

        assert!(project.track_uids()[0] == track_uid);

        assert!(project.delete_track(track_uid).is_ok());
        assert!(project.track_uids().is_empty());
    }

    #[test]
    fn zero_length_performance_ends_immediately() {
        let mut project = Project::default();

        // Controls::is_finished() is undefined before play(), so no fair
        // calling it before play().

        project.play();
        assert!(project.is_finished());
    }

    #[test]
    fn project_handles_transport_control() {
        let mut project = Project::default();

        let track_uid = project.create_track(None).unwrap();
        let uid = project
            .add_entity(
                track_uid,
                Box::new(TestControllerSendsOneEvent::default()),
                None,
            )
            .unwrap();

        assert!(
            project
                .link(
                    uid,
                    Project::TRANSPORT_UID,
                    ControlIndex(Transport::TEMPO_INDEX)
                )
                .is_ok(),
            "Linking with Transport's tempo should work"
        );

        assert_eq!(
            project.tempo(),
            Tempo::default(),
            "Initial project tempo should be default"
        );
        project.work(&mut |_| {});
        assert_eq!(
            project.tempo(),
            Tempo::from(Tempo::MAX_VALUE),
            "After a cycle of work, project tempo should be changed by automation"
        );
    }

    #[test]
    fn midi_routing_from_external_reaches_instruments() {
        let mut project = Project::default();
        let track_uid = project.new_midi_track().unwrap();

        let instrument = TestInstrumentCountsMidiMessages::default();
        let midi_messages_received = Arc::clone(instrument.received_midi_message_count_mutex());
        let instrument_uid = project
            .add_entity(track_uid, Box::new(instrument), None)
            .unwrap();
        assert!(project
            .set_midi_receiver_channel(instrument_uid, Some(MidiChannel::default()))
            .is_ok());

        let test_message = MidiMessage::NoteOn {
            key: 7.into(),
            vel: 13.into(),
        };
        if let Ok(received) = midi_messages_received.lock() {
            assert_eq!(
                *received, 0,
                "Before sending an external MIDI message to Project, count should be zero"
            );
        };
        project.handle_midi_message(
            MidiChannel::default(),
            test_message,
            &mut |channel, message| panic!("Didn't expect {channel:?} {message:?}",),
        );
        if let Ok(received) = midi_messages_received.lock() {
            assert_eq!(
                *received, 1,
                "Count should update after sending an external MIDI message to Project"
            );
        };
    }

    #[test]
    fn midi_messages_from_track_a_do_not_reach_track_b() {
        let mut project = Project::default();
        let track_a_uid = project.new_midi_track().unwrap();
        let track_b_uid = project.new_midi_track().unwrap();

        // On Track 1, put a sender and receiver.
        let sender_uid = project
            .add_entity(
                track_a_uid,
                Box::new(ToyControllerAlwaysSendsMidiMessage::default()),
                None,
            )
            .unwrap();
        let receiver_1 = TestInstrumentCountsMidiMessages::default();
        let counter_1 = Arc::clone(receiver_1.received_midi_message_count_mutex());
        let receiver_1_uid = project
            .add_entity(track_a_uid, Box::new(receiver_1), None)
            .unwrap();

        // On Track 2, put another receiver.
        let receiver_2 = TestInstrumentCountsMidiMessages::default();
        let counter_2 = Arc::clone(receiver_2.received_midi_message_count_mutex());
        let receiver_2_uid = project
            .add_entity(track_b_uid, Box::new(receiver_2), None)
            .unwrap();

        // Hook up everyone to MIDI.
        let _ = project.set_midi_receiver_channel(sender_uid, Some(MidiChannel::default()));
        let _ = project.set_midi_receiver_channel(receiver_1_uid, Some(MidiChannel::default()));
        let _ = project.set_midi_receiver_channel(receiver_2_uid, Some(MidiChannel::default()));

        // Fire everything up.
        project.play();
        project.work(&mut |_| {});

        // Sender should have sent a message that receiver #1 should receive,
        // because they're both in the same Track.
        if let Ok(c) = counter_1.lock() {
            assert_eq!(1, *c);
        }
        // But Receiver #2 shouldn't see that message, because it's in a
        // different Track.
        if let Ok(c) = counter_2.lock() {
            assert_eq!(0, *c);
        };
    }
}