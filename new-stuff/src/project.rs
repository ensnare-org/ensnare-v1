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
    pub midi_router: MidiRouter,

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
        let track_uid = self.orchestrator.create_track(None)?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("MIDI {}", track_uid)));
        Ok(track_uid)
    }

    /// Adds a new audio track, which can contain audio clips and effects.
    /// Returns the new track's [TrackUid] if successful.
    pub fn new_audio_track(&mut self) -> anyhow::Result<TrackUid> {
        let track_uid = self.orchestrator.create_track(None)?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("Audio {}", track_uid)));
        Ok(track_uid)
    }

    /// Adds a new aux track, which contains only effects, and to which other
    /// tracks can *send* their output audio. Returns the new track's [TrackUid]
    /// if successful.
    pub fn new_aux_track(&mut self) -> anyhow::Result<TrackUid> {
        let track_uid = self.orchestrator.create_track(None)?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("Aux {}", track_uid)));
        Ok(track_uid)
    }

    delegate! {
        to self.orchestrator {
            pub fn create_track(&mut self, uid: Option<TrackUid>) -> Result<TrackUid>;
            pub fn track_uids(&self) -> &[TrackUid];
            pub fn set_track_position(&mut self, uid: TrackUid, new_position: usize) -> Result<()>;
            pub fn delete_track(&mut self, uid: TrackUid) -> Result<()>;

            pub fn add_entity(&mut self, track_uid: TrackUid, entity: Box<dyn EntityBounds>, uid: Option<Uid>) -> Result<Uid>;
            pub fn move_entity(&mut self, uid: Uid, new_track_uid: Option<TrackUid>, new_position: Option<usize>)-> Result<()>;
            pub fn delete_entity(&mut self, uid: Uid) -> Result<()>;
            pub fn remove_entity(&mut self, uid: Uid) -> Result<Box<dyn EntityBounds>>;
        }
        to self.midi_router {
            pub fn set_midi_receiver_channel(&mut self, entity_uid: Uid, channel: Option<MidiChannel>) -> Result<()>;
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

    fn generate_frames(&mut self, frames: &mut [StereoSample]) {
        let time_range = self.transport.advance(frames.len());
        self.update_time_range(&time_range);
        self.work(&mut |_| {});
        self.is_finished = self.calculate_is_finished();
        if self.is_finished {
            self.stop();
        }
        self.generate_batch_values(frames);
    }

    fn calculate_is_finished(&mut self) -> bool {
        self.composer.is_finished() && self.orchestrator.is_finished()
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
        self.automator
            .route(&mut self.orchestrator.entity_repo, uid, value);
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
                    self.handle_midi_message(channel, message, &mut |c, m| {
                        events.push((Uid::default(), WorkEvent::Midi(c, m)))
                    });
                }
                WorkEvent::Control(value) => {
                    self.dispatch_control_event(uid, value);
                }
            }
        }
    }
}
impl HandlesMidi for Project {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
        if let Some(receivers) = self.midi_router.midi_receivers.get(&channel) {
            receivers.iter().for_each(|receiver_uid| {
                if let Some(entity) = self.orchestrator.get_entity_mut(receiver_uid) {
                    entity.handle_midi_message(channel, message, &mut |c, m| {
                        midi_messages_fn(c, m);
                    });
                }
            });
        }
    }
}
impl Serializable for Project {
    fn before_ser(&mut self) {
        self.automator.before_ser();
        self.orchestrator.before_ser();
        self.composer.before_ser();
        self.midi_router.before_ser();
    }

    fn after_deser(&mut self) {
        self.automator.after_deser();
        self.orchestrator.after_deser();
        self.composer.after_deser();
        self.midi_router.after_deser();
    }
}
