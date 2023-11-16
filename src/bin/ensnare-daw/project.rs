// Copyright (c) 2023 Mike Tsao. All rights reserved.

use anyhow::anyhow;
use crossbeam_channel::Sender;
use ensnare::arrangement::ProjectTitle;
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
    sync::{Arc, Mutex, RwLock},
};

#[derive(Debug)]
pub(super) struct InMemoryProject {
    pub(super) title: ProjectTitle,
    pub(super) orchestrator: Arc<Mutex<Orchestrator>>,
    pub(super) piano_roll: Arc<RwLock<PianoRoll>>,

    pub(super) view_range: ViewRange,
    pub(super) track_titles: HashMap<TrackUid, TrackTitle>,
    pub(super) track_frontmost_uids: HashMap<TrackUid, Uid>,

    pub(super) track_to_sequencer_sender: HashMap<TrackUid, Sender<SequencerInput>>,

    pub(super) is_piano_roll_visible: bool,
}
impl Default for InMemoryProject {
    fn default() -> Self {
        let mut r = Self {
            title: Default::default(),
            orchestrator: Default::default(),
            piano_roll: Default::default(),
            view_range: ViewRange(MusicalTime::START..MusicalTime::new_with_beats(4 * 4)),
            track_titles: Default::default(),
            track_frontmost_uids: Default::default(),
            track_to_sequencer_sender: Default::default(),
            is_piano_roll_visible: Default::default(),
        };
        let _ = r.create_starter_tracks();
        r.switch_to_next_frontmost_timeline_displayer();
        r
    }
}
impl DescribesProject for InMemoryProject {
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
impl InMemoryProject {
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
}
