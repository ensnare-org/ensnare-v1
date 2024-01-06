// Copyright (c) 2023 Mike Tsao. All rights reserved.

use anyhow::anyhow;
use eframe::egui::Id;
use ensnare::{
    arrangement::{DescribesProject, Orchestrates, Orchestrator},
    composition::{Sequence, SequenceRepository},
    control::ControlTripParams,
    cores::ThinSequencerParams,
    entities::controllers::{ControlTrip, LivePatternSequencer},
    prelude::*,
    ui::widgets::piano_roll,
};
use ensnare_core::sequence_repository::ArrangementInfo;
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
/// #[derive(Serialize, Deserialize)]. Complemented by [DiskProject], which *is*
/// #[derive(Serialize, Deserialize)] and hopefully moves more slowly.
///
/// [DawProject] is located within the `ensnare-daw` app module, whereas
/// [DiskProject] is in the top-level crate. This difference is meant to
/// indicate that [DiskProject] should be the serialization format for multiple
/// applications (such as the `render` example), while [DawProject] is just for
/// the DAW app.
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

    // Which of the timeline-displaying Entities is currently frontmost for a
    // track.
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
        let track_uid = o.create_track(None)?;

        self.track_titles
            .insert(track_uid, TrackTitle(format!("MIDI {}", track_uid)));

        let sequencer = ThinSequencer::new_with(
            o.mint_entity_uid(),
            &ThinSequencerParams::default(),
            track_uid,
            &self.sequence_repository,
            &self.piano_roll,
        );
        // self.track_to_sequencer_sender .insert(track_uid,
        //     sequencer.sender().clone());
        let _sequencer_uid = o.add_entity(&track_uid, Box::new(sequencer))?;

        // TODO: if you want this back again, figure out how to do it with
        // ControlTripParams ControlTripBuilder::default()
        // .random(MusicalTime::START) .build() .unwrap() .try_into() .unwrap(),

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
        let track_uid = o.create_track(None)?;
        self.track_titles
            .insert(track_uid, TrackTitle(format!("Audio {}", track_uid)));
        Ok(track_uid)
    }

    /// Adds a new aux track, which contains only effects, and to which other
    /// tracks can *send* their output audio. Returns the new track's [TrackUid]
    /// if successful.
    pub fn new_aux_track(&mut self) -> anyhow::Result<TrackUid> {
        let mut o = self.orchestrator.lock().unwrap();
        let track_uid = o.create_track(None)?;
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
        if let Ok(mut repo) = self.sequence_repository.write() {
            let _ = repo.add(Sequence::Pattern(pattern_uid), position, track_uid);
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
                            // Pick the one after the current one, wrapping back
                            // if needed.
                            let position = (position + 1) % entity_uids.len();
                            self.track_frontmost_uids
                                .insert(*track_uid, entity_uids[position]);
                        } else {
                            // We couldn't find the current one. Go back to the
                            // first one.
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
        track_entities: &TrackEntities,
        project: &DiskProject,
    ) -> anyhow::Result<()> {
        track_entities.entities.iter().for_each(|entity| {
            if let Err(e) = self.reconstitute_entity(&entity, &track_entities.track_uid, project) {
                eprintln!("Error while reconstituting Uid {}: {}", entity.uid, e);
            }
        });
        Ok(())
    }

    fn reconstitute_entity(
        &self,
        entity_info: &EntityInfo,
        track_uid: &TrackUid,
        project: &DiskProject,
    ) -> anyhow::Result<Uid> {
        let mut orchestrator = self.orchestrator.lock().unwrap();
        let uid = entity_info.uid;
        let entity: Box<dyn EntityWrapper> = match entity_info.params.as_ref() {
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
                // self.track_to_sequencer_sender .insert(*track_uid,
                //     sequencer.sender().clone());
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
                // self.reconstitute_sequencer_arrangements( track_uid,
                //     &project.arrangements, &mut sequencer, );
                //     self.track_to_sequencer_sender .insert(*track_uid,
                //     sequencer.sender().clone());
                Box::new(sequencer)
            }
        };
        orchestrator.add_entity(track_uid, entity)
    }

    fn reconstitute_sequencer_arrangements(
        &self,
        track_uid: &TrackUid,
        arrangements: &[ArrangementInfo],
        sequencer: &mut LivePatternSequencer,
    ) {
        arrangements
            .iter()
            .filter(|arrangement| *track_uid == arrangement.track_uid)
            .for_each(|arrangement| {
                arrangement
                    .arranged_sequences
                    .iter()
                    .for_each(|arranged_sequence| {
                        // TODO: note that sequence_uid is dropped on the floor.
                        // This means that LivePatternSequencer doesn't track
                        // where it came from.
                        if let Err(e) = sequencer.record(
                            arranged_sequence.channel,
                            &arranged_sequence.pattern_uid,
                            arranged_sequence.position,
                        ) {
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
                            dst_entities.push(EntityInfo { uid: *uid, params });
                        }
                    });
                    dst.entities.push(TrackEntities {
                        track_uid: *track_uid,
                        entities: dst_entities,
                    });
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
                            v.push(PatternInfo {
                                pattern_uid: *pattern_uid,
                                pattern: pattern.clone(),
                            });
                        }
                        v
                    },
                );
            }

            // Copy the sequencer's arrangements into the list of arrangements.
            if let Ok(repo) = src.sequence_repository.read() {
                dst.arrangements = repo.deref().into();
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

            // Next, cast to the Orchestrates trait so we can work more
            // generically.
            let dst_orchestrator: &mut dyn Orchestrates<dyn EntityWrapper> =
                dst_orchestrator.deref_mut();
            dst.title = src.title.clone();
            dst_orchestrator.update_tempo(src.tempo);
            dst_orchestrator.update_time_signature(src.time_signature);
            src.tracks.iter().for_each(|track_info| {
                let _ = dst_orchestrator.create_track(Some(track_info.uid));
                dst.track_titles
                    .insert(track_info.uid, track_info.title.clone());
            });
            src.midi_connections.iter().for_each(|connection| {
                connection.receiver_uids.iter().for_each(|&uid| {
                    let _ = dst_orchestrator.connect_midi_receiver(uid, connection.channel);
                });
            });
        }
        src.entities.iter().for_each(|track_entities| {
            if let Err(e) = dst.reconstitute_track_entities(track_entities, src) {
                eprintln!(
                    "Error while reconstituting entities for track {}: {e}",
                    track_entities.track_uid
                );
            }
        });
        if let Ok(mut piano_roll) = dst.piano_roll.write() {
            src.patterns.iter().for_each(|pattern_info| {
                if let Err(e) = piano_roll
                    .insert_with_uid(pattern_info.pattern.clone(), pattern_info.pattern_uid)
                {
                    eprintln!("Error while inserting pattern: {e}");
                }
            });
        }
        if let Ok(mut repo) = dst.sequence_repository.write() {
            *repo.deref_mut() = (&src.arrangements).into();
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

    impl DawProject {
        fn debug_random_orchestrator() -> Orchestrator<dyn EntityWrapper> {
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
                if let Ok(track_uid) = o.create_track(None) {
                    let _ = o.add_entity(
                        &track_uid,
                        Box::new(Trigger::new_with(
                            Uid(rng.rand_u64() as usize + 10000),
                            &TriggerParams::default(),
                        )),
                    );
                }
            }

            o
        }

        fn debug_random_track_titles(track_uids: &[TrackUid]) -> HashMap<TrackUid, TrackTitle> {
            let mut rng = Rng::default();

            track_uids.iter().fold(
                HashMap::default(),
                |mut h: HashMap<TrackUid, TrackTitle>, uid| {
                    h.insert(*uid, TrackTitle(format!("track name {:?}", rng.rand_u64())));
                    h
                },
            )
        }

        fn debug_random_piano_roll() -> PianoRoll {
            let mut piano_roll = PianoRoll::default();
            piano_roll.insert_16_random_patterns();
            piano_roll
        }

        fn debug_random_sequence_repository(track_uids: &[TrackUid]) -> SequenceRepository {
            let mut sequence_repository = SequenceRepository::default();
            track_uids.iter().for_each(|track_uid| {
                let _ = sequence_repository.add(
                    Sequence::Pattern(PatternUid(track_uid.0 * 1000 + 1)),
                    MusicalTime::START,
                    *track_uid,
                );

                // We don't support this right now -- only Sequence::Pattern let
                // _ = sequence_repository.add( Sequence::Note(vec![Note { key:
                //     45, range: TimeRange::new_with_start_and_duration(
                //         MusicalTime::DURATION_EIGHTH,
                //         MusicalTime::DURATION_SIXTEENTH, ), }]),
                //             MusicalTime::DURATION_EIGHTH, *track_uid, );
            });
            sequence_repository
        }

        fn debug_random() -> Self {
            let mut rng = Rng::default();

            let orchestrator = Self::debug_random_orchestrator();
            Self {
                title: ProjectTitle::from(format!("Title {}", rng.rand_u64()).as_str()),
                load_path: Some(PathBuf::from(
                    format!("/dev/proc/ouch/{}", rng.rand_u64()).to_string(),
                )),
                piano_roll: Arc::new(RwLock::new(Self::debug_random_piano_roll())),
                sequence_repository: Arc::new(RwLock::new(Self::debug_random_sequence_repository(
                    &orchestrator.track_uids,
                ))),
                view_range: ViewRange(
                    MusicalTime::new_with_units(rng.rand_range(0..10000) as usize)
                        ..MusicalTime::new_with_units(rng.rand_range(20000..30000) as usize),
                ),
                track_titles: Self::debug_random_track_titles(&orchestrator.track_uids),
                orchestrator: Arc::new(Mutex::new(orchestrator)),
                ..Default::default()
            }
        }

        /// This is modeled after PartialEq except...
        ///
        /// (1) it asserts rather than comparing; (2) it leaves out some fields
        /// that are inconsequential for testing serializing.
        fn debug_compare(&self, other: &Self) {
            assert_eq!(self.title, other.title);
            // skip load_path because we don't care if two otherwise identical
            // projects were loaded from different places.

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
            assert_eq!(self.track_titles, other.track_titles);

            // The following fields are for ephemeral UI and are OK to not
            // match:
            //
            // - view_range
            // - track_frontmost_uids
            // - is_piano_roll_visible
            // - all detail_ fields
        }
    }

    #[test]
    fn identity_memory_default_to_disk_to_memory() {
        let factory =
            EnsnareEntities::register(EntityFactory::<dyn EntityWrapper>::default()).finalize();
        let src = DawProject::new_project();
        let dst: DiskProject = <&DawProject>::into(&src);
        let src_copy: DawProject =
            <(&DiskProject, &EntityFactory<dyn EntityWrapper>)>::into((&dst, &factory));
        src.debug_compare(&src_copy);
    }

    #[test]
    fn identity_random_memory_to_disk_to_memory() {
        let factory =
            EnsnareEntities::register(EntityFactory::<dyn EntityWrapper>::default()).finalize();
        let src = DawProject::debug_random();
        let dst: DiskProject = <&DawProject>::into(&src);
        let src_copy: DawProject =
            <(&DiskProject, &EntityFactory<dyn EntityWrapper>)>::into((&dst, &factory));
        src.debug_compare(&src_copy);
    }

    #[test]
    fn identity_disk_to_memory_to_disk() {
        let factory =
            EnsnareEntities::register(EntityFactory::<dyn EntityWrapper>::default()).finalize();
        let src = DiskProject::default();
        let dst: DawProject =
            <(&DiskProject, &EntityFactory<dyn EntityWrapper>)>::into((&src, &factory));
        let src_copy: DiskProject = <&DawProject>::into(&dst);
        assert_eq!(src, src_copy);
    }
}
