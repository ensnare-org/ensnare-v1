// Copyright (c) 2023 Mike Tsao. All rights reserved.

use anyhow::anyhow;
use eframe::egui::Id;
use ensnare::{
    all_entities::{EntityParams, EntityWrapper},
    arrangement::{DescribesProject, Orchestrates, Orchestrator},
    composition::{Sequence, SequenceRepository},
    control::ControlTripParams,
    cores::ThinSequencerParams,
    entities::controllers::{ControlTrip, LivePatternSequencer},
    prelude::*,
    project::{ProjectTitle, TrackInfo},
    ui::widgets::piano_roll,
    DiskProject,
};
use ensnare_entities::{
    controllers::{Arpeggiator, LfoController, SignalPassthroughController, ThinSequencer},
    effects::{
        filter::BiQuadFilterLowPass24db, Bitcrusher, Chorus, Compressor, Gain, Limiter, Mixer,
        Reverb,
    },
    instruments::{Drumkit, FmSynth, Sampler, WelshSynth},
};
use ensnare_entities_toy::{
    controllers::{ToyController, ToyControllerAlwaysSendsMidiMessage},
    effects::ToyEffect,
    instruments::{ToyInstrument, ToySynth},
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
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
    pub(super) orchestrator: Arc<Mutex<Orchestrator<dyn EntityWrapper>>>,
    pub(super) piano_roll: Arc<RwLock<PianoRoll>>,
    pub(super) sequence_repository: Arc<RwLock<SequenceRepository>>,

    pub(super) view_range: ViewRange,
    pub(super) track_titles: HashMap<TrackUid, TrackTitle>,
    pub(super) track_frontmost_uids: HashMap<TrackUid, Uid>,

    pub(super) is_piano_roll_visible: bool,
    pub(super) detail_is_visible: bool,
    pub(super) detail_title: String,
    pub(super) detail_uid: Option<Uid>,
}
impl Default for DawProject {
    fn default() -> Self {
        Self {
            title: Default::default(),
            load_path: Default::default(),
            orchestrator: Arc::new(Mutex::new(Orchestrator::new())),
            piano_roll: Default::default(),
            sequence_repository: Default::default(),
            view_range: ViewRange(MusicalTime::START..MusicalTime::new_with_beats(4 * 4)),
            track_titles: Default::default(),
            track_frontmost_uids: Default::default(),
            is_piano_roll_visible: Default::default(),
            detail_is_visible: Default::default(),
            detail_title: Default::default(),
            detail_uid: Default::default(),
        }
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
impl PartialEq for DawProject {
    fn eq(&self, other: &Self) -> bool {
        if self.title == other.title
            && self.load_path == other.load_path
            && *self.orchestrator.lock().unwrap() == *other.orchestrator.lock().unwrap()
            && *self.piano_roll.read().unwrap() == *other.piano_roll.read().unwrap()
            && *self.sequence_repository.read().unwrap()
                == *other.sequence_repository.read().unwrap()
            && self.view_range == other.view_range
            && self.track_titles == other.track_titles
            && self.track_frontmost_uids == other.track_frontmost_uids
            && self.is_piano_roll_visible == other.is_piano_roll_visible
            && self.detail_is_visible == other.detail_is_visible
            && self.detail_title == other.detail_title
            && self.detail_uid == other.detail_uid
        {
            true
        } else {
            // This makes it easier to debug which part of the test failed
            //
            // TODO remove when stable
            assert_eq!(self.title, other.title);
            assert_eq!(self.load_path, other.load_path);
            assert_eq!(
                *self.orchestrator.lock().unwrap(),
                *other.orchestrator.lock().unwrap()
            );
            assert_eq!(
                *self.piano_roll.read().unwrap(),
                *other.piano_roll.read().unwrap()
            );
            assert_eq!(
                *self.sequence_repository.read().unwrap(),
                *other.sequence_repository.read().unwrap()
            );
            assert_eq!(self.view_range, other.view_range);
            assert_eq!(self.track_titles, other.track_titles);
            assert_eq!(self.track_frontmost_uids, other.track_frontmost_uids);
            assert_eq!(self.is_piano_roll_visible, other.is_piano_roll_visible);
            assert_eq!(self.detail_is_visible, other.detail_is_visible);
            assert_eq!(self.detail_title, other.detail_title);
            assert_eq!(self.detail_uid, other.detail_uid);

            false
        }
    }
}
impl DawProject {
    /// Starts with a default project and configures for easy first use.
    pub fn new_project() -> Self {
        let mut r = Self::default();
        let _ = r.create_starter_tracks();
        r.switch_to_next_frontmost_timeline_displayer();
        r
    }

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

        let sequencer = ThinSequencer::new_with(
            o.mint_entity_uid(),
            &ThinSequencerParams::default(),
            track_uid,
            &self.sequence_repository,
            &self.piano_roll,
        );
        // self.track_to_sequencer_sender
        //     .insert(track_uid, sequencer.sender().clone());
        let _sequencer_uid = o.add_entity(&track_uid, Box::new(sequencer))?;

        // TODO: if you want this back again, figure out how to do it with ControlTripParams
        // ControlTripBuilder::default()
        // .random(MusicalTime::START)
        // .build()
        // .unwrap()
        // .try_into()
        // .unwrap(),

        let control_trip = ControlTrip::new_with(
            o.mint_entity_uid(),
            &ControlTripParams::default(),
            &o.control_router,
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
                let mut inner = self.piano_roll.write().unwrap();
                let response = ui.add(piano_roll(&mut inner));
                if response.changed() {
                    eprintln!("yasss");
                    self.sequence_repository.write().unwrap().notify_change();
                }
                response
            });
    }

    pub(crate) fn request_pattern_add(
        &self,
        track_uid: TrackUid,
        pattern_uid: PatternUid,
        position: MusicalTime,
    ) {
        // if let Some(channel) = self.track_to_sequencer_sender.get(&track_uid) {
        //     let _ = channel.send(SequencerInput::AddPattern(pattern_uid, position));
        // }
        if let Ok(mut repo) = self.sequence_repository.write() {
            let _ = repo.add(Sequence::Pattern(pattern_uid), position, track_uid);
            eprintln!("did it");
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

    pub(crate) fn load(
        path: PathBuf,
        factory: &EntityFactory<dyn EntityWrapper>,
    ) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(&path)?;
        let project = serde_json::from_str::<DiskProject>(&json)?;
        let mut daw_project: DawProject = (&project, factory).into();
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

        let project: DiskProject = self.into();
        let json = serde_json::to_string_pretty(&project)?;
        std::fs::write(&save_path, json)?;
        Ok(save_path)
    }

    fn reconstitute_track_entities(
        &mut self,
        track_uid: &TrackUid,
        entities: &[(Uid, Box<EntityParams>)],
        project: &DiskProject,
    ) -> anyhow::Result<()> {
        entities.iter().for_each(|(uid, params)| {
            if let Err(e) = self.reconstitute_entity(params.as_ref(), *uid, &track_uid, project) {
                eprintln!("Error while reconstituting Uid {}: {}", *uid, e);
            }
        });
        Ok(())
    }

    fn reconstitute_entity(
        &self,
        params: &EntityParams,
        uid: Uid,
        track_uid: &TrackUid,
        project: &DiskProject,
    ) -> anyhow::Result<Uid> {
        let mut orchestrator = self.orchestrator.lock().unwrap();
        let entity: Box<dyn EntityWrapper> = match params {
            EntityParams::Arpeggiator(params) => Box::new(Arpeggiator::new_with(uid, params)),
            EntityParams::BiQuadFilterLowPass24db(params) => {
                Box::new(BiQuadFilterLowPass24db::new_with(uid, params))
            }
            EntityParams::ControlTrip(params) => Box::new(ControlTrip::new_with(
                uid,
                params,
                &orchestrator.control_router,
            )),
            EntityParams::FmSynth(params) => Box::new(FmSynth::new_with(uid, params)),
            EntityParams::LivePatternSequencer(params) => {
                let mut sequencer = LivePatternSequencer::new_with(uid, params, &self.piano_roll);
                self.reconstitute_sequencer_arrangements(
                    track_uid,
                    &project.arrangements,
                    &mut sequencer,
                );
                // self.track_to_sequencer_sender
                //     .insert(*track_uid, sequencer.sender().clone());
                Box::new(sequencer)
            }
            EntityParams::ToyController(params) => Box::new(ToyController::new_with(uid, params)),
            EntityParams::ToyControllerAlwaysSendsMidiMessage(params) => {
                Box::new(ToyControllerAlwaysSendsMidiMessage::new_with(uid, params))
            }
            EntityParams::ToyEffect(params) => Box::new(ToyEffect::new_with(uid, params)),
            EntityParams::ToyInstrument(params) => Box::new(ToyInstrument::new_with(uid, params)),
            EntityParams::ToySynth(params) => Box::new(ToySynth::new_with(uid, params)),
            EntityParams::WelshSynth(params) => Box::new(WelshSynth::new_with(uid, params)),
            EntityParams::LfoController(params) => Box::new(LfoController::new_with(uid, params)),
            EntityParams::SignalPassthroughController(params) => {
                Box::new(SignalPassthroughController::new_with(uid, params))
            }
            EntityParams::Sampler(params) => Box::new(Sampler::new_with(uid, params)),
            EntityParams::Trigger(params) => Box::new(Trigger::new_with(uid, params)),
            EntityParams::Timer(params) => Box::new(Timer::new_with(uid, params)),
            EntityParams::Bitcrusher(params) => Box::new(Bitcrusher::new_with(uid, params)),
            EntityParams::Drumkit(params) => {
                Box::new(Drumkit::new_with(uid, params, &Paths::default()))
            }
            EntityParams::Chorus(params) => Box::new(Chorus::new_with(uid, params)),
            EntityParams::Compressor(params) => Box::new(Compressor::new_with(uid, params)),
            EntityParams::Gain(params) => Box::new(Gain::new_with(uid, params)),
            EntityParams::Limiter(params) => Box::new(Limiter::new_with(uid, params)),
            EntityParams::Mixer(params) => Box::new(Mixer::new_with(uid, params)),
            EntityParams::Reverb(params) => Box::new(Reverb::new_with(uid, params)),
            EntityParams::ThinSequencer(params) => {
                let sequencer = ThinSequencer::new_with(
                    uid,
                    params,
                    *track_uid,
                    &self.sequence_repository,
                    &self.piano_roll,
                );
                // self.reconstitute_sequencer_arrangements(
                //     track_uid,
                //     &project.arrangements,
                //     &mut sequencer,
                // );
                // self.track_to_sequencer_sender
                //     .insert(*track_uid, sequencer.sender().clone());
                Box::new(sequencer)
            }
        };
        orchestrator.add_entity(track_uid, entity)
    }

    fn reconstitute_sequencer_arrangements(
        &self,
        track_uid: &TrackUid,
        arrangements: &[(TrackUid, Vec<(MidiChannel, MusicalTime, PatternUid)>)],
        sequencer: &mut LivePatternSequencer,
    ) {
        arrangements
            .iter()
            .filter(|(tuid, _)| *track_uid == *tuid)
            .for_each(|(_, track_arrangements)| {
                track_arrangements
                    .iter()
                    .for_each(|(channel, time, pattern_uid)| {
                        if let Err(e) = sequencer.record(*channel, pattern_uid, *time) {
                            eprintln!("While arranging: {e:?}");
                        }
                    });
            });
    }
}

// From memory to disk
impl From<&DawProject> for DiskProject {
    fn from(src: &DawProject) -> Self {
        let mut dst = DiskProject::default();
        if let Ok(src_orchestrator) = src.orchestrator.lock() {
            dst.title = src.title.clone();
            dst.tempo = src_orchestrator.transport.tempo;
            dst.time_signature = src_orchestrator.transport.time_signature();
            let track_uids = src_orchestrator.track_uids.clone();

            dst.midi_connections = (&src_orchestrator.midi_router).into();

            // Serialize each track's entities.
            track_uids.iter().for_each(|track_uid| {
                let mut dst_entities = Vec::default();
                if let Some(entities) = src_orchestrator.entities_for_track.get(track_uid) {
                    entities.iter().for_each(|uid| {
                        if let Some(entity) = src_orchestrator.get_entity(uid) {
                            let params: Box<EntityParams> = entity.try_into().unwrap();
                            dst_entities.push((*uid, params));
                        }
                    });
                    dst.entities.push((*track_uid, dst_entities));
                }
            });

            // Save each track's metadata.
            dst.tracks = track_uids.iter().fold(Vec::default(), |mut v, track_uid| {
                v.push(TrackInfo {
                    uid: *track_uid,
                    title: src
                        .track_titles
                        .get(track_uid)
                        .unwrap_or(&TrackTitle::default())
                        .clone(),
                });
                v
            });

            // Copy [PianoRoll]'s contents into the list of patterns.
            if let Ok(piano_roll) = src.piano_roll.read() {
                dst.patterns = piano_roll.ordered_pattern_uids.iter().fold(
                    Vec::default(),
                    |mut v, pattern_uid| {
                        if let Some(pattern) = piano_roll.get_pattern(pattern_uid) {
                            v.push((*pattern_uid, pattern.clone()));
                        }
                        v
                    },
                );
            }

            // Copy the sequencer's arrangements into the list of arrangements.
            if let Ok(repo) = src.sequence_repository.read() {
                let repo: &SequenceRepository = repo.deref();
                dst.arrangements = repo.into();
            }
        }
        dst
    }
}

// From disk to memory
impl From<(&DiskProject, &EntityFactory<dyn EntityWrapper>)> for DawProject {
    fn from(value: (&DiskProject, &EntityFactory<dyn EntityWrapper>)) -> Self {
        let (src, _factory) = value;
        let mut dst = DawProject::default();
        if let Ok(mut dst_orchestrator) = dst.orchestrator.lock() {
            // First deal with the functionality that requires a concrete
            // Orchestrator.
            //
            // (none at present)
            //
            // UidFactories do not need to have their next_uid counters
            // explicitly reset because we've wired the create_x_with_uid()
            // methods to notify the factory about externally created Uids,
            // which gives the factory an opportunity to bump the counter past
            // that value.

            // Next, cast to the Orchestrates trait so we can work more generically.
            let dst_orchestrator: &mut dyn Orchestrates<dyn EntityWrapper> =
                dst_orchestrator.deref_mut();
            dst.title = src.title.clone();
            dst_orchestrator.update_tempo(src.tempo);
            dst_orchestrator.update_time_signature(src.time_signature);
            src.tracks.iter().for_each(|track_info| {
                let _ = dst_orchestrator.create_track_with_uid(track_info.uid);
                dst.track_titles
                    .insert(track_info.uid, track_info.title.clone());
            });
            src.midi_connections.iter().for_each(|(channel, uids)| {
                uids.iter().for_each(|&uid| {
                    let _ = dst_orchestrator.connect_midi_receiver(uid, *channel);
                });
            });
        }
        src.entities.iter().for_each(|(track_uid, track_entities)| {
            if let Err(e) = dst.reconstitute_track_entities(track_uid, track_entities, src) {
                eprintln!("Error while reconstituting entities for track {track_uid}: {e}");
            }
        });
        if let Ok(mut piano_roll) = dst.piano_roll.write() {
            src.patterns.iter().for_each(|(pattern_uid, pattern)| {
                if let Err(e) = piano_roll.insert_with_uid(pattern.clone(), *pattern_uid) {
                    eprintln!("Error while inserting pattern: {e}");
                }
            });
        }
        if let Ok(mut repository) = dst.sequence_repository.write() {
            src.arrangements
                .iter()
                .for_each(|(track_uid, arrangements)| {
                    arrangements
                        .iter()
                        .for_each(|(channel, time, pattern_uid)| {
                            if let Ok(sequence_uid) =
                                repository.add(Sequence::Pattern(*pattern_uid), *time, *track_uid)
                            {
                                // is it done at this point? Concern that ThinSequencer doesn't know the sequence IDs, so how can it allow the user to edit one?
                            }
                        });
                });
        }

        dst
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ensnare::all_entities::EnsnareEntities;
    use ensnare_core::{controllers::TriggerParams, rng::Rng};
    use std::sync::Arc;

    #[derive(Debug, PartialEq)]
    struct OrchestratorWrapper(pub Orchestrator<dyn EntityWrapper>);
    impl OrchestratorWrapper {
        fn test_random() -> Self {
            let mut rng = Rng::default();
            let mut o = Orchestrator::<dyn EntityWrapper>::new();
            o.update_sample_rate(SampleRate(rng.rand_range(11000..30000) as usize));

            for _ in 0..10 {
                let _ = o.connect_midi_receiver(
                    Uid(rng.rand_u64() as usize),
                    MidiChannel(rng.rand_range(0..MidiChannel::MAX_VALUE as u64 + 1) as u8),
                );
            }

            for _ in 0..4 {
                if let Ok(track_uid) = o.create_track() {
                    let _ = o.add_entity(
                        &track_uid,
                        Box::new(Trigger::new_with(
                            Uid(rng.rand_u64() as usize + 10000),
                            &TriggerParams::default(),
                        )),
                    );
                }
            }

            Self(o)
        }
    }

    impl DawProject {
        fn debug_eq(&self, other: &Self) -> bool {
            debug_assert_eq!(self.title, other.title);
            debug_assert_eq!(
                *self.orchestrator.lock().unwrap(),
                *other.orchestrator.lock().unwrap()
            );
            return true;
        }

        fn test_random() -> Self {
            let mut rng = Rng::default();

            Self {
                title: ProjectTitle::from(format!("Title {}", rng.rand_u64()).as_str()),
                load_path: Some(PathBuf::from(
                    format!("/dev/proc/ouch/{}", rng.rand_u64()).to_string(),
                )),
                orchestrator: Arc::new(Mutex::new(OrchestratorWrapper::test_random().0)),
                view_range: ViewRange(
                    MusicalTime::new_with_units(rng.rand_range(0..10000) as usize)
                        ..MusicalTime::new_with_units(rng.rand_range(20000..30000) as usize),
                ),
                ..Default::default()
            }
        }
    }

    #[test]
    fn identity_starting_with_in_memory() {
        let factory =
            EnsnareEntities::register(EntityFactory::<dyn EntityWrapper>::default()).finalize();
        let src = DawProject::new_project();
        let dst: DiskProject = <&DawProject>::into(&src);
        let src_copy: DawProject =
            <(&DiskProject, &EntityFactory<dyn EntityWrapper>)>::into((&dst, &factory));
        assert!(src.debug_eq(&src_copy));
    }

    #[test]
    fn identity_starting_with_in_memory_nondefault() {
        let factory =
            EnsnareEntities::register(EntityFactory::<dyn EntityWrapper>::default()).finalize();
        let src = DawProject::test_random();
        let dst: DiskProject = <&DawProject>::into(&src);
        let src_copy: DawProject =
            <(&DiskProject, &EntityFactory<dyn EntityWrapper>)>::into((&dst, &factory));
        assert!(src.debug_eq(&src_copy));
    }

    #[test]
    fn identity_starting_with_serialized() {
        let factory =
            EnsnareEntities::register(EntityFactory::<dyn EntityWrapper>::default()).finalize();
        let src = DiskProject::default();
        let dst: DawProject =
            <(&DiskProject, &EntityFactory<dyn EntityWrapper>)>::into((&src, &factory));
        let src_copy: DiskProject = <&DawProject>::into(&dst);
        assert_eq!(src, src_copy);
    }
}
