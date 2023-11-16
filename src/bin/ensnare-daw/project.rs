// Copyright (c) 2023 Mike Tsao. All rights reserved.

use anyhow::anyhow;
use ensnare::arrangement::ProjectTitle;
use ensnare_core::{piano_roll::PianoRoll, prelude::*, types::TrackTitle};
use ensnare_cores_egui::piano_roll::piano_roll;
use ensnare_egui_widgets::ViewRange;
use ensnare_entities::controllers::LivePatternSequencer;
use ensnare_orchestration::{traits::Orchestrates, Orchestrator};
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
            is_piano_roll_visible: Default::default(),
        };
        let _ = r.create_starter_tracks();
        r
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
        let sequencer =
            LivePatternSequencer::new_with(o.mint_entity_uid(), Arc::clone(&self.piano_roll));

        // let sequencer = NoteSequencer::new_with_inner(
        //     o.mint_entity_uid(),
        //     ensnare_cores::NoteSequencerBuilder::default()
        //         .build()
        //         .unwrap(),
        // );

        //     let trip_uid = o.mint_entity_uid();
        let track_uid = o.create_track()?;
        let _sequencer_uid = o.add_entity(&track_uid, Box::new(sequencer))?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("MIDI {}", track_uid)));
        // o.add_entity(
        //     &track_uid,
        //     Box::new(
        //         ControlTripBuilder::default()
        //             .random(MusicalTime::START)
        //             .build()
        //             .unwrap(),
        //     ),
        // );
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
}
