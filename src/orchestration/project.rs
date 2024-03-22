// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Representation of a whole music project, including support for
//! serialization.

use crate::{
    automation::Automator,
    composition::Composer,
    egui::TargetInstrument,
    orchestration::{MidiRouter, Orchestrator, TrackTitle},
    prelude::*,
    types::{AudioQueue, ColorScheme, VisualizationQueue},
    util::{Rng, SelectionSet},
};
use anyhow::{anyhow, Result};
use delegate::delegate;
use derivative::Derivative;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// A user-visible project title.
#[derive(Clone, Debug, Derivative, derive_more::Display, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectTitle(#[derivative(Default(value = "\"Untitled\".into()"))] String);
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
impl ProjectTitle {
    /// TODO: I don't know why From<&str> didn't give us this for free.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Indicates what should be shown in the track view.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TrackViewMode {
    /// Shows arranged note [Pattern]s.
    #[default]
    Composition,
    /// Shows one [SignalPath].
    Control(PathUid),
}

enum SignalPathNextError {
    ThereAreNone,
    ReachedEndOfList,
}

/// Groups all the persistent fields related to the project GUI view.
///
/// We've made the explicit decision to make view parameters persistent, so that
/// a loaded project looks the same as when it was saved. The downside is that
/// saving a project after browsing it will usually generate diffs. TODO: maybe
/// put these kinds of items into a "ProjectViewState" struct, and don't mark
/// the project dirty when those change. Then they'll be saved only along with
/// substantial changes.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectViewState {
    /// The part of the project to show in the UI.
    pub view_range: ViewRange,
    /// Which tracks are selected.
    pub track_selection_set: SelectionSet<TrackUid>,
    /// Which widget to render in the track arrangement section.
    #[serde(default)]
    pub track_view_mode: FxHashMap<TrackUid, TrackViewMode>,
    /// The current playback point. This is redundant -- copied from Transport.
    pub cursor: Option<MusicalTime>,
}

/// Utility
#[derive(Debug)]
pub struct SignalChainItem {
    pub uid: Uid,
    pub name: String,
    pub is_control_source: bool,
}

/// Temporary information associated with each track.
#[derive(Debug, Default)]
pub struct TrackInfo {
    pub signal_chain: Vec<SignalChainItem>,
    pub targets: Vec<TargetInstrument>,
}

#[derive(Debug, Default)]
pub struct ProjectEphemerals {
    /// Whether the project has been saved.
    is_clean: bool,

    /// Whether the project has finished a performance.
    is_finished: bool,

    /// If present, then this is the path that was used to load this project
    /// from disk.
    pub load_path: Option<PathBuf>,

    pub audio_queue: Option<AudioQueue>,

    /// A non-owned VecDeque that acts as a ring buffer of the most recent
    /// generated audio frames.
    pub visualization_queue: Option<VisualizationQueue>,

    /// The random source for all entropic behavior that should be consistenly
    /// replayable. Example: a noise generator or a random arpeggiator should
    /// use this source because a the project should render the same way each
    /// time. Non-example: a GUI action that generates a random pattern should
    /// not use this source because the expectation is that the GUI action will
    /// be different each time.
    pub rng: Rng,

    pub track_info: FxHashMap<TrackUid, TrackInfo>,
}

/// A musical piece. Also knows how to render the piece to digital audio.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    #[serde(default)]
    pub title: Option<ProjectTitle>,
    #[serde(default)]
    pub track_titles: FxHashMap<TrackUid, TrackTitle>,
    #[serde(default)]
    pub track_color_schemes: FxHashMap<TrackUid, ColorScheme>,

    /// The seed to instantiate a new RNG on each replay of the project.
    #[serde(default)]
    pub rng_seed: u128,

    pub transport: Transport,
    pub orchestrator: Orchestrator,
    pub automator: Automator,
    pub composer: Composer,
    pub track_to_midi_router: FxHashMap<TrackUid, MidiRouter>,
    track_to_paths: FxHashMap<TrackUid, Vec<PathUid>>,

    pub view_state: ProjectViewState,

    #[serde(skip)]
    pub e: ProjectEphemerals,
}
impl Project {
    /// The fixed [Uid] for the project's Orchestrator.
    pub const ORCHESTRATOR_UID: Uid = Uid(1);

    /// The fixed [Uid] for the project's [Transport].
    pub const TRANSPORT_UID: Uid = Uid(2);

    delegate! {
        to self.orchestrator {
            pub fn track_uids(&self) -> &[TrackUid];
            pub fn set_track_position(&mut self, uid: TrackUid, new_position: usize) -> Result<()>;

            pub fn get_humidity(&self, uid: &Uid) -> Normal;
            pub fn set_humidity(&mut self, uid: Uid, humidity: Normal);

            pub fn track_output(&mut self, track_uid: TrackUid) -> Normal;
            pub fn set_track_output(&mut self, track_uid: TrackUid, output: Normal);
            pub fn mute_track(&mut self, track_uid: TrackUid, muted: bool);
            pub fn is_track_muted(&mut self, track_uid: TrackUid) -> bool;
            pub fn solo_track(&self) -> Option<TrackUid>;
            pub fn set_solo_track(&mut self, track_uid: Option<TrackUid>);

            pub fn add_send(&mut self, src_uid: TrackUid, dst_uid: TrackUid, amount: Normal) -> anyhow::Result<()>;
            pub fn remove_send(&mut self, send_track_uid: TrackUid, aux_track_uid: TrackUid);

            pub fn mint_entity_uid(&self) -> Uid;
            pub fn mint_track_uid(&self) -> TrackUid;
        }
        to self.composer {
            pub fn add_pattern(&mut self, contents: Pattern, pattern_uid: Option<PatternUid>) -> Result<PatternUid>;
            pub fn pattern(&self, pattern_uid: PatternUid) -> Option<&Pattern>;
            pub fn pattern_mut(&mut self, pattern_uid: PatternUid) -> Option<&mut Pattern>;
            pub fn notify_pattern_change(&mut self);
            pub fn remove_pattern(&mut self, pattern_uid: PatternUid) -> Result<Pattern>;
            pub fn arrange_pattern(&mut self, track_uid: TrackUid, pattern_uid: PatternUid, position: MusicalTime) -> Result<ArrangementUid>;
            pub fn move_arrangement(&mut self, track_uid: TrackUid, arrangement_uid: ArrangementUid, new_position: MusicalTime, copy_original: bool) -> Result<ArrangementUid>;
            pub fn unarrange(&mut self, track_uid: TrackUid, arrangement_uid: ArrangementUid);
        }
        to self.automator {
            pub fn link(&mut self, source: Uid, target: Uid, param: ControlIndex) -> Result<()>;
            pub fn unlink(&mut self, source: Uid, target: Uid, param: ControlIndex);
            pub fn remove_path(&mut self, path_uid: PathUid) -> Option<SignalPath>;
            pub fn link_path(&mut self, path_uid: PathUid, target_uid: Uid, param: ControlIndex) -> Result<()> ;
            pub fn unlink_path(&mut self, path_uid: PathUid, target_uid: Uid, param: ControlIndex);
        }
    }

    /// Starts with a default project and configures for easy first use.
    pub fn new_project() -> Self {
        let mut r = Self::default();
        r.set_rng_seed(Rng::generate_seed().unwrap());
        let _ = r.create_starter_tracks();

        // hack - default to a 1-minute song
        r.view_state.view_range = ViewRange(
            MusicalTime::START..MusicalTime::new_with_beats(r.transport.tempo().0 as usize),
        );
        r
    }

    /// Adds a set of tracks that make sense for a new project.
    pub fn create_starter_tracks(&mut self) -> anyhow::Result<()> {
        if !self.orchestrator.track_repo.uids().is_empty() {
            return Err(anyhow!("Must be invoked on an empty project."));
        }

        let t1 = self.new_midi_track()?;
        let t2 = self.new_midi_track()?;
        let t3 = self.new_audio_track()?;
        let t4 = self.new_aux_track()?;

        self.track_color_schemes.insert(t1, ColorScheme::Amber);
        self.track_color_schemes.insert(t2, ColorScheme::Chartreuse);
        self.track_color_schemes.insert(t3, ColorScheme::Red);
        self.track_color_schemes.insert(t4, ColorScheme::Violet);

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
        self.orchestrator.aux_track_uids.push(track_uid);
        Ok(track_uid)
    }

    pub fn create_track(&mut self, uid: Option<TrackUid>) -> Result<TrackUid> {
        let track_uid = self.orchestrator.create_track(uid)?;
        self.track_to_midi_router
            .insert(track_uid, MidiRouter::default());
        Ok(track_uid)
    }

    pub fn delete_track(&mut self, uid: TrackUid) -> Result<()> {
        self.track_to_midi_router.remove(&uid);
        self.orchestrator.aux_track_uids.retain(|t| *t != uid);
        self.orchestrator.delete_track(uid)
    }

    pub fn get_midi_receiver_channel(&mut self, entity_uid: Uid) -> Option<MidiChannel> {
        if let Some(track_uid) = self.orchestrator.track_for_entity(entity_uid) {
            if let Some(midi_router) = self.track_to_midi_router.get_mut(&track_uid) {
                return midi_router.uid_to_channel.get(&entity_uid).cloned();
            }
        }
        None
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

    pub fn move_entity(
        &mut self,
        uid: Uid,
        new_track_uid: Option<TrackUid>,
        new_position: Option<usize>,
    ) -> Result<()> {
        let midi_channel = self.get_midi_receiver_channel(uid);
        let result = self
            .orchestrator
            .move_entity(uid, new_track_uid, new_position);
        self.set_midi_receiver_channel(uid, midi_channel);
        result
    }

    fn generate_frames(
        &mut self,
        frames: &mut [StereoSample],
        mut midi_events_fn: Option<&mut MidiMessagesFn>,
    ) {
        let is_finished_at_start = self.e.is_finished;
        let time_range = self.transport.advance(frames.len());
        self.update_time_range(&time_range);
        self.work(&mut |e| {
            if let Some( midi_events_fn) = midi_events_fn.as_mut() {
                match e {
                    WorkEvent::Midi(channel, message) => midi_events_fn(channel,message),
                    WorkEvent::MidiForTrack(_track, channel, message) => midi_events_fn(channel,message),
                    WorkEvent::Control(_control_value) => panic!("generate_frames() received WorkEvent::Control, which should be handled elsewhere"),
                }
            }
        });
        if !is_finished_at_start && self.e.is_finished {
            self.stop();
        }
        self.generate(frames);
    }

    fn update_is_finished(&mut self) {
        self.e.is_finished = self.composer.is_finished() && self.orchestrator.is_finished();
    }

    /// Renders the project as a WAV file to the specified path.
    pub fn export_to_wav(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: self.sample_rate().into(),
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;

        self.skip_to_start();
        self.play();

        let mut detected_silence_after_performance = false;
        while !detected_silence_after_performance {
            let mut samples = [StereoSample::SILENCE; 64];
            self.generate_frames(&mut samples, None);
            for sample in samples {
                let (left, right) = sample.into_i16();
                let _ = writer.write_sample(left);
                let _ = writer.write_sample(right);
            }
            if !self.is_performing() {
                if samples.iter().all(|s| s.almost_silent()) {
                    detected_silence_after_performance = true;
                }
            }
        }
        Ok(())
    }

    pub fn fill_audio_queue(
        &mut self,
        count: usize,
        mut midi_events_fn: Option<&mut MidiMessagesFn>,
    ) {
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
            self.generate_frames(buffer_slice, midi_events_fn.as_deref_mut());
            if let Some(audio_queue) = self.e.audio_queue.as_ref() {
                buffer_slice.iter().for_each(|s| {
                    if let Some(_old_element) = audio_queue.force_push(*s) {
                        eprintln!("overrun! requested {count} frames");

                        // There is no point in continuing.
                        return;
                    }
                });
            }
            if let Some(queue) = self.e.visualization_queue.as_ref() {
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

    fn dispatch_control_event(&mut self, source: ControlLinkSource, value: ControlValue) {
        self.automator.route(
            &mut self.orchestrator.entity_repo,
            Some(&mut |link| match link.uid {
                Self::TRANSPORT_UID => self.transport.control_set_param_by_index(link.param, value),
                _ => {
                    eprintln!("Asked to route from unknown source {source}");
                }
            }),
            source,
            value,
        );
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(&path)?;
        let mut project = serde_json::from_str::<Self>(&json)?;
        project.e.load_path = Some(path);
        project.after_deser();
        project.e.is_clean = true;
        Ok(project)
    }

    pub fn save(&mut self, path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
        let save_path = {
            if let Some(path) = path.as_ref() {
                path.clone()
            } else if let Some(path) = self.e.load_path.as_ref() {
                path.clone()
            } else {
                return Err(anyhow!("Can't save without a path"));
            }
        };

        self.before_ser();
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(&save_path, json)?;
        self.e.is_clean = true;
        Ok(save_path)
    }

    pub fn load_path(&self) -> Option<&PathBuf> {
        self.e.load_path.as_ref()
    }

    pub fn add_path(&mut self, track_uid: TrackUid, path: SignalPath) -> Result<PathUid> {
        let path_uid = self.automator.add_path(path)?;
        self.track_to_paths
            .entry(track_uid)
            .or_default()
            .push(path_uid);
        Ok(path_uid)
    }

    /// Picks the next view mode for the given track.
    pub fn advance_track_view_mode(&mut self, track_uid: TrackUid) {
        let mode = match self.track_view_mode(track_uid) {
            TrackViewMode::Composition => self.next_track_view_mode_with_path_uid(track_uid, None),
            TrackViewMode::Control(path_uid) => {
                self.next_track_view_mode_with_path_uid(track_uid, Some(path_uid))
            }
        };
        self.set_track_view_mode(track_uid, mode);
    }

    fn track_view_mode(&self, track_uid: TrackUid) -> TrackViewMode {
        self.view_state
            .track_view_mode
            .get(&track_uid)
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn set_track_view_mode(&mut self, track_uid: TrackUid, mode: TrackViewMode) {
        self.view_state
            .track_view_mode
            .insert(track_uid.clone(), mode);
        self.regenerate_signal_chain(track_uid);
    }

    fn next_track_view_mode_with_path_uid(
        &mut self,
        track_uid: TrackUid,
        path_uid: Option<PathUid>,
    ) -> TrackViewMode {
        match self.next_signal_path(track_uid, path_uid) {
            Ok(next_path_uid) => TrackViewMode::Control(next_path_uid),
            Err(err) => match err {
                SignalPathNextError::ThereAreNone => {
                    let path = SignalPathBuilder::default().build().unwrap();
                    if let Ok(new_path_uid) = self.add_path(track_uid, path) {
                        TrackViewMode::Control(new_path_uid)
                    } else {
                        TrackViewMode::Composition
                    }
                }
                SignalPathNextError::ReachedEndOfList => TrackViewMode::Composition,
            },
        }
    }

    fn next_signal_path(
        &self,
        track_uid: TrackUid,
        current_path_uid: Option<PathUid>,
    ) -> Result<PathUid, SignalPathNextError> {
        if let Some(path_uids) = self.track_to_paths.get(&track_uid) {
            if let Some(current_path_uid) = current_path_uid {
                if let Some(index) = path_uids.iter().position(|puid| *puid == current_path_uid) {
                    let next_index = index + 1;
                    if next_index < path_uids.len() {
                        return Ok(path_uids[next_index]);
                    } else {
                        return Err(SignalPathNextError::ReachedEndOfList);
                    }
                }
            }
            if let Some(path_uid) = path_uids.first().copied() {
                return Ok(path_uid);
            }
        }
        Err(SignalPathNextError::ThereAreNone)
    }

    pub fn notify_transport_sample_rate_change(&mut self) {
        self.update_sample_rate(self.sample_rate());
    }

    pub fn notify_transport_tempo_change(&mut self) {
        self.update_tempo(self.tempo());
    }

    pub fn notify_transport_time_signature_change(&mut self) {
        self.update_time_signature(self.time_signature());
    }

    /// A convenience method for automated tests to spit out their work product.
    pub fn save_and_export(&mut self, path_prefix: PathBuf) -> anyhow::Result<()> {
        let mut path = path_prefix.clone();
        path.set_extension("json");
        self.save(Some(path))?;
        let mut path = path_prefix.clone();
        path.set_extension("wav");
        self.export_to_wav(path)?;
        Ok(())
    }

    fn reset_rng(&mut self) {
        self.e.rng = Rng::new_with_seed(self.rng_seed);
    }

    /// Sets this project's random number generator seed, which is used to
    /// deterministically (repeatably) generate a stream of pseudorandom numbers
    /// used throughout the project.
    pub fn set_rng_seed(&mut self, seed: u128) {
        self.rng_seed = seed;
        self.reset_rng();
    }

    /// Adds an entity to the given track. If the [Uid] is not specified,
    /// generates a new one. Returns the [Uid] if adding was successful.
    pub fn add_entity(
        &mut self,
        track_uid: TrackUid,
        entity: Box<dyn EntityBounds>,
        uid: Option<Uid>,
    ) -> Result<Uid> {
        let r = self.orchestrator.add_entity(track_uid, entity, uid);
        if let Ok(uid) = r {
            self.regenerate_signal_chain(track_uid);
        }
        r
    }

    /// Deletes and discards an existing entity.
    pub fn delete_entity(&mut self, uid: Uid) -> Result<()> {
        self.set_midi_receiver_channel(uid, None)?;
        let track_uid = self.orchestrator.track_for_entity(uid);
        let r = self.orchestrator.delete_entity(uid);
        if let Some(track_uid) = track_uid {
            self.regenerate_signal_chain(track_uid);
        }
        r
    }

    /// Removes an existing entity from the project and returns it to the
    /// caller.
    pub fn remove_entity(&mut self, uid: Uid) -> Result<Box<dyn EntityBounds>> {
        self.set_midi_receiver_channel(uid, None)?;
        let track_uid = self.orchestrator.track_for_entity(uid);
        let r = self.orchestrator.remove_entity(uid);
        if let Some(track_uid) = track_uid {
            self.regenerate_signal_chain(track_uid);
        }
        r
    }

    /// Regenerates cacheable information associated with a track's entities.
    /// TODO: figure out easy-to-understand rules about when this needs to be
    /// called.
    pub(crate) fn regenerate_signal_chain(&mut self, track_uid: TrackUid) {
        let track_info = self.e.track_info.entry(track_uid).or_default();
        let mut targets = Vec::default();

        // If the current view mode is to show a SignalPath, then we need to
        // update the cache of controllable states for it.
        let path_uid = if let Some(mode) = self.view_state.track_view_mode.get(&track_uid) {
            match mode {
                TrackViewMode::Composition => None,
                TrackViewMode::Control(path_uid) => Some(path_uid.clone()),
            }
        } else {
            None
        };

        let signal_chain: Vec<SignalChainItem> = {
            if let Some(entity_uids) = self.orchestrator.entity_repo.uids_for_track.get(&track_uid)
            {
                entity_uids.iter().fold(Vec::default(), |mut v, uid| {
                    if let Some(entity) = self.orchestrator.entity_repo.entity(*uid) {
                        v.push(SignalChainItem {
                            uid: *uid,
                            name: entity.name().to_string(),
                            is_control_source: true,
                        });
                        let mut controllables = Vec::default();
                        for i in 0..entity.control_index_count() {
                            let index = ControlIndex(i);

                            let is_linked = if let Some(path_uid) = path_uid {
                                self.automator.is_path_linked(path_uid, *uid, index)
                            } else {
                                false
                            };

                            controllables.push((
                                ControlName(
                                    entity.control_name_for_index(index).unwrap().to_string(),
                                ),
                                is_linked,
                            ));
                        }
                        if !controllables.is_empty() {
                            targets.push(TargetInstrument {
                                uid: *uid,
                                name: entity.name().to_string(),
                                controllables,
                            });
                        }
                    }

                    v
                })
            } else {
                Vec::default()
            }
        };
        track_info.signal_chain = signal_chain;
        track_info.targets = targets;
    }

    /// We have just switched to this track in the UI, so we want to update
    /// Composer's note labels to match the track's lead instrument. (This is
    /// flaky and temporary TODO)
    pub(crate) fn refresh_note_labels(&mut self, track_uid: TrackUid) {
        if let Some(entity_uids) = self.orchestrator.entity_repo.uids_for_track.get(&track_uid) {
            if entity_uids
                .iter()
                .find(|uid| {
                    if let Some(entity) = self.orchestrator.entity_repo.entity(**uid) {
                        if let Some(label_metadata) = entity.midi_note_label_metadata() {
                            self.composer
                                .set_midi_note_label_metadata(&Arc::new(label_metadata));
                            return true;
                        }
                    }
                    false
                })
                .is_some()
            {
                return;
            }
            // Nobody responded to note_labels(), so everyone's OK with the
            // default.
        }
        // No entities. Maybe it doesn't make sense to allow creating a
        // pattern, but let's go with it for now. TODO
        self.composer.clear_midi_note_label_metadata();
    }
}
impl Generates<StereoSample> for Project {
    delegate! {
        to self.orchestrator {
            fn generate(&mut self, values: &mut [StereoSample]);
        }
    }
}
impl Ticks for Project {}
impl Configurable for Project {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.transport.update_sample_rate(sample_rate);
        self.orchestrator.update_sample_rate(sample_rate);
        self.composer.update_sample_rate(sample_rate);
    }
    fn update_tempo(&mut self, tempo: Tempo) {
        self.transport.update_tempo(tempo);
        self.orchestrator.update_tempo(tempo);
        self.composer.update_tempo(tempo);
    }
    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.transport.update_time_signature(time_signature);
        self.orchestrator.update_time_signature(time_signature);
        self.composer.update_time_signature(time_signature);
    }

    fn reset(&mut self) {
        self.reset_rng();
        self.transport.reset();
        self.orchestrator.reset();
        self.composer.reset();
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
        self.e.is_finished
    }

    fn play(&mut self) {
        self.transport.play();
        self.automator.play();
        self.orchestrator.play();
        self.composer.play();
        self.update_is_finished();
    }

    fn stop(&mut self) {
        self.transport.stop();
        self.automator.stop();
        self.orchestrator.stop();
        self.composer.stop();

        self.track_to_midi_router
            .values_mut()
            .for_each(|router| router.all_notes_off(&mut self.orchestrator.entity_repo));
    }

    fn skip_to_start(&mut self) {
        self.reset();
        self.transport.skip_to_start();
        self.automator.skip_to_start();
        self.orchestrator.skip_to_start();
        self.composer.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.transport.is_performing()
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.automator.update_time_range(time_range);
        self.orchestrator.update_time_range(time_range);
        self.composer.update_time_range(time_range);
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        let mut events = Vec::default();
        self.automator
            .work_as_proxy(&mut |source, event| events.push((Some(source), event)));
        self.composer.work(&mut |event| events.push((None, event)));
        self.orchestrator
            .work_as_proxy(&mut |source, event| events.push((Some(source), event)));
        while let Some((source, event)) = events.pop() {
            match event {
                WorkEvent::Midi(_, _) => {
                    // This is a logic error because it means that we don't know
                    // which track created this MIDI event, which means that we
                    // don't know which entities are eligible to receive it. (We
                    // confine MIDI events to their originating track to avoid
                    // the need for a system to route channels to tracks.)
                    todo!("Project must know a MIDI event's originating track. Please map WorkEvent::Midi to WorkEvent::MidiForTrack before passing it to Project.");
                }
                WorkEvent::MidiForTrack(track_uid, channel, message) => {
                    if let Some(midi_router) = self.track_to_midi_router.get(&track_uid) {
                        let _ =
                            midi_router.route(&mut self.orchestrator.entity_repo, channel, message);
                    }
                    // Give caller an opportunity to route messages elsewhere.
                    control_events_fn(event);
                }
                WorkEvent::Control(value) => {
                    if let Some(source) = source {
                        self.dispatch_control_event(source, value);
                    }
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
            });
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
    use crate::{
        cores::instruments::TestAudioSourceCoreBuilder,
        entities::{
            TestAudioSource, TestControllerAlwaysSendsMidiMessage, TestEffectNegatesInput,
            TestInstrumentCountsMidiMessages,
        },
    };
    use ensnare_proc_macros::{Control, IsEntity, Metadata};
    use std::sync::Arc;

    trait TestEntity: EntityBounds {}

    /// An [IsEntity] that sends one Control event each time work() is called.
    #[derive(Debug, Default, Control, IsEntity, Metadata, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    #[entity(
        Configurable,
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
    fn project_makes_sounds() {
        let mut project = Project::default();
        let track_uid = project.create_track(None).unwrap();
        let instrument_uid = project
            .add_entity(
                track_uid,
                Box::new(TestAudioSource::new_with(
                    Uid::default(),
                    TestAudioSourceCoreBuilder::default()
                        .level(TestAudioSource::MEDIUM)
                        .build()
                        .unwrap(),
                )),
                None,
            )
            .unwrap();
        assert!(project
            .set_midi_receiver_channel(instrument_uid, Some(MidiChannel::default()))
            .is_ok());
        let pattern_uid = project
            .add_pattern(
                PatternBuilder::default()
                    .note(Note::new_with_midi_note(
                        MidiNote::A4,
                        MusicalTime::START,
                        MusicalTime::DURATION_EIGHTH,
                    ))
                    .build()
                    .unwrap(),
                None,
            )
            .unwrap();
        let _ = project.arrange_pattern(track_uid, pattern_uid, MusicalTime::START);

        project.play();
        let mut frames = [StereoSample::SILENCE; 4];
        project.generate_frames(&mut frames, None);
        assert!(frames
            .iter()
            .any(|frame| { *frame != StereoSample::SILENCE }));
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
                Box::new(TestControllerAlwaysSendsMidiMessage::default()),
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

    #[test]
    fn sends_send() {
        const EXPECTED_LEVEL: ParameterType = TestAudioSource::MEDIUM;
        let mut project = Project::default();
        let midi_track_uid = project.new_midi_track().unwrap();
        let aux_track_uid = project.new_aux_track().unwrap();

        let _ = project.add_entity(
            midi_track_uid,
            Box::new(TestAudioSource::new_with(
                Uid::default(),
                TestAudioSourceCoreBuilder::default()
                    .level(EXPECTED_LEVEL)
                    .build()
                    .unwrap(),
            )),
            None,
        );
        let mut samples = [StereoSample::SILENCE; 64];
        project.generate_frames(&mut samples, None);
        let expected_sample = StereoSample::from(EXPECTED_LEVEL);
        assert!(
            samples.iter().all(|s| *s == expected_sample),
            "Without a send, original signal should pass through unchanged."
        );

        assert!(project
            .add_send(midi_track_uid, aux_track_uid, Normal::from(0.5))
            .is_ok());
        let mut samples = [StereoSample::SILENCE; 64];
        project.generate_frames(&mut samples, None);
        let expected_sample = StereoSample::from(0.75);
        samples.iter().enumerate().for_each(|(index, s)| {
            assert_eq!(*s, expected_sample, "With a 50% send to an aux track with no effects, we should see the original MEDIUM=0.5 plus 50% of it = 0.75, but at sample #{index} we got {:?}", s);
        });

        // Add an effect to the aux track.
        let _ = project.add_entity(
            aux_track_uid,
            Box::new(TestEffectNegatesInput::default()),
            None,
        );

        let mut samples = [StereoSample::SILENCE; 64];
        project.generate_frames(&mut samples, None);
        let expected_sample = StereoSample::from(0.5 + 0.5 * 0.5 * -1.0);
        samples.iter().enumerate().for_each(|(index,s)| {
            assert_eq!(*s ,expected_sample, "With a 50% send to an aux with a negating effect, we should see the original 0.5 plus a negation of 50% of 0.5 = 0.250, but at sample #{index} we got {:?}", s);
        });
    }

    #[test]
    fn mixer_works() {
        const EXPECTED_LEVEL: ParameterType = TestAudioSource::MEDIUM;
        let mut project = Project::default();
        let track_1_uid = project.new_midi_track().unwrap();
        let track_2_uid = project.new_midi_track().unwrap();

        let _ = project.add_entity(
            track_1_uid,
            Box::new(TestAudioSource::new_with(
                Uid::default(),
                TestAudioSourceCoreBuilder::default()
                    .level(EXPECTED_LEVEL)
                    .build()
                    .unwrap(),
            )),
            None,
        );
        let _ = project.add_entity(
            track_2_uid,
            Box::new(TestAudioSource::new_with(
                Uid::default(),
                TestAudioSourceCoreBuilder::default()
                    .level(EXPECTED_LEVEL)
                    .build()
                    .unwrap(),
            )),
            None,
        );

        let mut samples = [StereoSample::SILENCE; 4];
        project.generate_frames(&mut samples, None);
        let expected_sample = StereoSample::from(0.5 + 0.5);
        samples.iter().enumerate().for_each(|(index, s)| {
            assert_eq!(*s, expected_sample, "Two tracks each with a 0.5 output should mix to {expected_sample:?}, but at sample #{index} we got {s:?}");
        });

        project.set_track_output(track_1_uid, Normal::from(0.5));
        let mut samples = [StereoSample::SILENCE; 4];
        project.generate_frames(&mut samples, None);
        let expected_sample = StereoSample::from(0.5 + 0.5 * 0.5);
        samples.iter().enumerate().for_each(|(index, s)| {
            assert_eq!(*s, expected_sample, "Setting one track's output to 50% should mix to {expected_sample:?}, but at sample #{index} we got {s:?}");
        });

        project.set_solo_track(Some(track_1_uid));
        let mut samples = [StereoSample::SILENCE; 4];
        project.generate_frames(&mut samples, None);
        let expected_sample = StereoSample::from(0.5 * 0.5);
        samples.iter().enumerate().for_each(|(index, s)| {
            assert_eq!(*s, expected_sample, "Soloing Track #1 (which is set to 50%) should mix to {expected_sample:?}, but at sample #{index} we got {s:?}");
        });

        project.set_solo_track(None);
        project.mute_track(track_1_uid, true);
        let mut samples = [StereoSample::SILENCE; 4];
        project.generate_frames(&mut samples, None);
        let expected_sample = StereoSample::from(0.5);
        samples.iter().enumerate().for_each(|(index, s)| {
            assert_eq!(*s, expected_sample, "Muting Track #1 and ending solo should mix to {expected_sample:?}, but at sample #{index} we got {s:?}");
        });
    }

    #[test]
    fn project_routes_midi_to_external() {
        let mut project = Project::default();

        let pattern_uid = project
            .add_pattern(
                PatternBuilder::default()
                    .note(Note::new_with_midi_note(
                        MidiNote::A0,
                        MusicalTime::START,
                        MusicalTime::DURATION_QUARTER,
                    ))
                    .build()
                    .unwrap(),
                None,
            )
            .unwrap();
        let track_1_uid = project.new_midi_track().unwrap();

        let _ = project.arrange_pattern(track_1_uid, pattern_uid, MusicalTime::START);

        let mut messages = Vec::default();
        let mut samples = [StereoSample::SILENCE; 4];
        project.play();
        project.generate_frames(&mut samples, Some(&mut |c, m| messages.push((c, m))));

        assert!(
            !messages.is_empty(),
            "Project should route MIDI messages to caller"
        );
        let (channel, message) = messages.first().unwrap();
        assert_eq!(
            *channel,
            MidiChannel::default(),
            "Received message should be on default channel"
        );
        assert!(
            matches!(message, MidiMessage::NoteOn { .. },),
            "Received message should be a note-on message"
        );
    }

    #[test]
    fn project_rng_is_random_but_repeatable() {
        let mut prior_seed = 0;
        let are_any_different = (0..10).any(|_| {
            let p = Project::new_project();
            let seed = p.rng_seed;
            let is_different = prior_seed != seed;
            prior_seed = seed;
            is_different
        });
        assert!(
            are_any_different,
            "Among several new projects, at least one's RNG seed should be different"
        );

        let mut p = Project::new_project();
        let seed = p.rng_seed;
        let prior_num = p.e.rng.rand_i64();
        let prior_num_2 = p.e.rng.rand_i64();

        let mut p = Project::new_project();

        // Just in case RNG seed is totally broken, ask for one number and throw
        // it away so that the sequence is misaligned.
        let _throwaway_num = p.e.rng.rand_i64();

        p.set_rng_seed(seed);
        assert_eq!(
            prior_num,
            p.e.rng.rand_i64(),
            "Restoring a seed should cause project to generate same RNG stream"
        );
        assert_eq!(
            prior_num_2,
            p.e.rng.rand_i64(),
            "Restoring a seed should cause project to generate same RNG stream"
        );
    }

    #[test]
    fn track_view_modes() {
        let mut p = Project::new_project();
        let track_1 = p.create_track(None).unwrap();
        let track_2 = p.create_track(None).unwrap();

        assert_eq!(
            p.track_view_mode(track_1),
            TrackViewMode::default(),
            "Initial view is composition/default"
        );
        assert_eq!(
            p.track_view_mode(track_2),
            TrackViewMode::Composition,
            "Initial view is composition"
        );

        p.advance_track_view_mode(track_1);
        let mode_phase_1 = p.track_view_mode(track_1);
        assert!(
            matches!(mode_phase_1, TrackViewMode::Control(..)),
            "Advancing past composition view should create a new control view"
        );
        assert_eq!(
            p.track_view_mode(track_2),
            TrackViewMode::Composition,
            "Setting one track's view shouldn't change any other's view"
        );

        p.advance_track_view_mode(track_1);
        assert_eq!(
            p.track_view_mode(track_1),
            TrackViewMode::Composition,
            "Advancing past last view should go to first"
        );

        p.advance_track_view_mode(track_1);
        let mode_phase_2 = p.track_view_mode(track_1);
        assert_eq!(
            mode_phase_2, mode_phase_1,
            "Second time around, should return to same control view"
        );
        p.advance_track_view_mode(track_1);
        assert_eq!(
            p.track_view_mode(track_2),
            TrackViewMode::Composition,
            "If there was already one control view, we shouldn't keep creating new ones"
        );
    }
}
