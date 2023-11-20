// Copyright (c) 2023 Mike Tsao. All rights reserved.

use anyhow::anyhow;
use crossbeam_channel::Sender;
use eframe::egui::Id;
use ensnare::{arrangement::ProjectTitle, Project};
use ensnare_core::{
    controllers::ControlTripBuilder,
    piano_roll::{PatternUid, PianoRoll},
    prelude::*,
    types::TrackTitle,
};
use ensnare_cores_egui::piano_roll::piano_roll;
use ensnare_egui_widgets::ViewRange;
use ensnare_entities::controllers::{ControlTrip, LivePatternSequencer, SequencerInput};
use ensnare_orchestration::{traits::Orchestrates, DescribesProject, Orchestrator};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

/// An in-memory representation of the project. Explicitly meant *not* to be
/// #[derive(Serialize, Deserialize)]. Complemented by [Project], which *is*
/// #[derive(Serialize, Deserialize)] and hopefully moves more slowly.
///
/// [DawProject] is located within the `ensnare-daw` app module, whereas
/// [Project] is in the top-level crate. This difference is meant to indicate
/// that [Project] should be the serialization format for multiple applications
/// (such as the `render` example), while [DawProject] is just for the DAW app.
#[derive(Debug)]
pub(super) struct DawProject {
    pub(super) title: ProjectTitle,

    // If present, then this is the path that was used to load this project from
    // disk.
    pub(super) load_path: Option<PathBuf>,
    pub(super) orchestrator: Arc<Mutex<Orchestrator>>,
    pub(super) piano_roll: Arc<RwLock<PianoRoll>>,

    pub(super) view_range: ViewRange,
    pub(super) track_titles: HashMap<TrackUid, TrackTitle>,
    pub(super) track_frontmost_uids: HashMap<TrackUid, Uid>,

    pub(super) track_to_sequencer_sender: HashMap<TrackUid, Sender<SequencerInput>>,

    pub(super) is_piano_roll_visible: bool,
    pub(super) detail_is_visible: bool,
    pub(super) detail_title: String,
    pub(super) detail_uid: Option<Uid>,
}
impl Default for DawProject {
    fn default() -> Self {
        let mut r = Self {
            title: Default::default(),
            load_path: Default::default(),
            orchestrator: Default::default(),
            piano_roll: Default::default(),
            view_range: ViewRange(MusicalTime::START..MusicalTime::new_with_beats(4 * 4)),
            track_titles: Default::default(),
            track_frontmost_uids: Default::default(),
            track_to_sequencer_sender: Default::default(),
            is_piano_roll_visible: Default::default(),
            detail_is_visible: Default::default(),
            detail_title: Default::default(),
            detail_uid: Default::default(),
        };
        let _ = r.create_starter_tracks();
        r.switch_to_next_frontmost_timeline_displayer();
        r
    }
}
impl DescribesProject for DawProject {
    fn track_title(&self, track_uid: &TrackUid) -> Option<&TrackTitle> {
        self.track_titles.get(track_uid)
    }

    fn track_frontmost_timeline_displayer(&self, track_uid: &TrackUid) -> Option<Uid> {
        if let Some(uid) = self.track_frontmost_uids.get(track_uid) {
            Some(*uid)
        } else {
            None
        }
    }
}
impl DawProject {
    /// Adds a set of tracks that make sense for a new project.
    pub fn create_starter_tracks(&mut self) -> anyhow::Result<()> {
        if !self.orchestrator.lock().unwrap().track_uids.is_empty() {
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
        let mut o = self.orchestrator.lock().unwrap();
        let track_uid = o.create_track()?;

        self.track_titles
            .insert(track_uid, TrackTitle(format!("MIDI {}", track_uid)));

        let sequencer =
            LivePatternSequencer::new_with(o.mint_entity_uid(), Arc::clone(&self.piano_roll));
        self.track_to_sequencer_sender
            .insert(track_uid, sequencer.sender().clone());
        let _sequencer_uid = o.add_entity(&track_uid, Box::new(sequencer))?;

        let control_trip = ControlTrip::new_with(
            o.mint_entity_uid(),
            ControlTripBuilder::default()
                .random(MusicalTime::START)
                .build()
                .unwrap(),
            o.control_router.clone(),
        );
        let _trip_uid = o.add_entity(&track_uid, Box::new(control_trip));

        Ok(track_uid)
    }

    /// Adds a new audio track, which can contain audio clips and effects.
    /// Returns the new track's [TrackUid] if successful.
    pub fn new_audio_track(&mut self) -> anyhow::Result<TrackUid> {
        let mut o = self.orchestrator.lock().unwrap();
        let track_uid = o.create_track()?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("Audio {}", track_uid)));
        Ok(track_uid)
    }

    /// Adds a new aux track, which contains only effects, and to which other
    /// tracks can *send* their output audio. Returns the new track's [TrackUid]
    /// if successful.
    pub fn new_aux_track(&mut self) -> anyhow::Result<TrackUid> {
        let mut o = self.orchestrator.lock().unwrap();
        let track_uid = o.create_track()?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("Aux {}", track_uid)));
        Ok(track_uid)
    }

    pub(crate) fn show_piano_roll(&mut self, ui: &mut eframe::egui::Ui) {
        eframe::egui::Window::new("Piano Roll")
            .open(&mut self.is_piano_roll_visible)
            .default_width(ui.available_width())
            .anchor(
                eframe::emath::Align2::LEFT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                // let mut inner = self.piano_roll.write().unwrap();
                // let inner_piano_roll: &mut PianoRoll = &mut inner;
                // ui.add(piano_roll(inner_piano_roll))
                let mut inner = self.piano_roll.write().unwrap();
                ui.add(piano_roll(&mut inner))
            });
    }

    pub(crate) fn request_pattern_add(
        &self,
        track_uid: TrackUid,
        pattern_uid: PatternUid,
        position: MusicalTime,
    ) {
        if let Some(channel) = self.track_to_sequencer_sender.get(&track_uid) {
            let _ = channel.send(SequencerInput::AddPattern(pattern_uid, position));
        }
    }

    pub(crate) fn switch_to_next_frontmost_timeline_displayer(&mut self) {
        if let Ok(o) = self.orchestrator.lock() {
            for track_uid in o.track_uids() {
                // Have the current frontmost Uid ready.
                let frontmost_uid = self.track_frontmost_uids.get(track_uid).copied();
                if let Ok(entity_uids) = o.get_track_timeline_entities(track_uid) {
                    if entity_uids.is_empty() {
                        // We don't have any timeline displayers in this track.
                        // Remove any existing entry.
                        self.track_frontmost_uids.remove(track_uid);
                    } else {
                        // Find where the current one is in the list.
                        let position = entity_uids
                            .iter()
                            .position(|uid| Some(*uid) == frontmost_uid);
                        if let Some(position) = position {
                            // Pick the one after the current one, wrapping back if needed.
                            let position = (position + 1) % entity_uids.len();
                            self.track_frontmost_uids
                                .insert(*track_uid, entity_uids[position]);
                        } else {
                            // We couldn't find the current one. Go back to the first one.
                            self.track_frontmost_uids.insert(*track_uid, entity_uids[0]);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn show_detail(&mut self, ui: &mut eframe::egui::Ui) {
        eframe::egui::Window::new(self.detail_title.to_string())
            .id(Id::new("Entity Detail"))
            .open(&mut self.detail_is_visible)
            .anchor(
                eframe::emath::Align2::RIGHT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                if let Some(uid) = self.detail_uid {
                    if let Some(entity) = self.orchestrator.lock().unwrap().get_entity_mut(&uid) {
                        entity.ui(ui);
                    }
                }
            });
    }

    pub(crate) fn select_detail(&mut self, uid: Uid, name: String) {
        self.detail_is_visible = true;
        self.detail_title = name;
        self.detail_uid = Some(uid);
    }

    pub(crate) fn load(path: PathBuf) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(&path)?;
        let project = serde_json::from_str::<Project>(&json)?;
        let mut daw_project: DawProject = (&project).into();
        daw_project.load_path = Some(path);
        Ok(daw_project)
    }

    pub(crate) fn save(&self, path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
        let save_path = {
            if let Some(path) = path.as_ref() {
                path.clone()
            } else if let Some(path) = self.load_path.as_ref() {
                path.clone()
            } else {
                PathBuf::from("ensnare-project.json")
            }
        };

        let project: Project = self.into();
        let json = serde_json::to_string_pretty(&project)?;
        std::fs::write(&save_path, json)?;
        Ok(save_path)
    }
}

impl From<&DawProject> for Project {
    fn from(src: &DawProject) -> Self {
        let mut dst = Project::default();
        if let Ok(src_orchestrator) = src.orchestrator.lock() {
            dst.title = src.title.clone();
            dst.tempo = src_orchestrator.transport.tempo;
            dst.time_signature = src_orchestrator.transport.time_signature();
        }
        dst
    }
}
impl From<&Project> for DawProject {
    fn from(src: &Project) -> Self {
        let mut dst = DawProject::default();
        if let Ok(mut dst_orchestrator) = dst.orchestrator.lock() {
            dst.title = src.title.clone();
            dst_orchestrator.transport.tempo = src.tempo;
            dst_orchestrator
                .transport
                .update_time_signature(src.time_signature);
        }

        dst
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ensnare::Project;

    struct OrchestratorWrapper<'a>(&'a Orchestrator);
    impl<'a> OrchestratorWrapper<'a> {
        fn debug_eq(&self, other: &Self) -> bool {
            if self.0.tempo() != other.0.tempo() {
                return false;
            }
            if self.0.time_signature() != other.0.time_signature() {
                return false;
            }
            if self.0.time_range() != other.0.time_range() {
                return false;
            }
            return true;
        }
    }

    impl DawProject {
        fn debug_eq(&self, other: &Self) -> bool {
            if self.title != other.title {
                return false;
            }
            if let Ok(o) = self.orchestrator.lock() {
                if let Ok(other_o) = other.orchestrator.lock() {
                    let o = OrchestratorWrapper(&o);
                    let other_o = OrchestratorWrapper(&other_o);
                    if !o.debug_eq(&other_o) {
                        return false;
                    }
                }
            }
            return true;
        }
    }

    #[test]
    fn identity_starting_with_in_memory() {
        let src = DawProject::default();
        let dst: Project = <&DawProject>::into(&src);
        let src_copy: DawProject = <&Project>::into(&dst);
        assert!(src.debug_eq(&src_copy));
    }

    #[test]
    fn identity_starting_with_serialized() {
        let src = Project::default();
        let dst: DawProject = <&Project>::into(&src);
        let src_copy: Project = <&DawProject>::into(&dst);
        assert_eq!(src, src_copy);
    }
}
