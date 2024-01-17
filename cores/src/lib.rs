// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Cores are basic musical devices.

use anyhow::{anyhow, Result};
use ensnare_core::{
    piano_roll::{Pattern, PatternBuilder, PatternUid},
    prelude::*,
    selection_set::SelectionSet,
    traits::Sequences,
    types::ColorScheme,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use controllers::*;
pub use effects::*;
pub use instruments::*;

pub mod controllers;
pub mod effects;
pub mod instruments;
pub mod toys;

/// [ModSerial] is a simple counter that lets us inform subscribers that
/// something has changed. Subscribers should keep a usize and compare to see
/// whether it differs from the one that we're currently reporting. If it does,
/// then they should update it and deal with the change.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModSerial(pub usize);
impl Default for ModSerial {
    // We start at something other than usize::default() so that
    // everyone else can use the default value and fire their update
    // code on the first call to has_changed().
    fn default() -> Self {
        Self(1000)
    }
}

#[derive(Debug, Default)]
pub struct ComposerEphemerals {
    pub pattern_selection_set: SelectionSet<PatternUid>,

    tracks_to_sequencers: HashMap<TrackUid, PatternSequencer>,

    time_range: TimeRange,
    is_finished: bool,
    is_performing: bool,

    // Each time something changes in the repo, this number will change. Use the
    // provided methods to manage a local copy of it and decide whether to act.
    mod_serial: ModSerial,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Composer {
    pattern_uid_factory: UidFactory<PatternUid>,
    pub patterns: HashMap<PatternUid, Pattern>,
    pub ordered_pattern_uids: Vec<PatternUid>,
    pub tracks_to_arrangements: HashMap<TrackUid, Vec<Arrangement>>,

    #[serde(skip)]
    pub e: ComposerEphemerals,
}
impl Composer {
    // TODO temp
    pub fn insert_16_random_patterns(&mut self) {
        (0..16).for_each(|i| {
            let pattern = PatternBuilder::default()
                .random()
                .color_scheme(ColorScheme::from_repr(i).unwrap())
                .build()
                .unwrap();
            let _ = self.add_pattern(pattern, None);
        });
    }

    pub fn add_pattern(
        &mut self,
        contents: Pattern,
        pattern_uid: Option<PatternUid>,
    ) -> Result<PatternUid> {
        let pattern_uid = if let Some(pattern_uid) = pattern_uid {
            pattern_uid
        } else {
            self.pattern_uid_factory.mint_next()
        };
        self.patterns.insert(pattern_uid, contents);
        self.ordered_pattern_uids.push(pattern_uid);
        Ok(pattern_uid)
    }

    pub fn pattern(&self, pattern_uid: PatternUid) -> Option<&Pattern> {
        self.patterns.get(&pattern_uid)
    }

    pub fn pattern_mut(&mut self, pattern_uid: PatternUid) -> Option<&mut Pattern> {
        self.patterns.get_mut(&pattern_uid)
    }

    pub fn notify_pattern_change(&mut self) {
        self.replay_arrangements();
    }

    pub fn remove_pattern(&mut self, pattern_uid: PatternUid) -> Result<Pattern> {
        if let Some(pattern) = self.patterns.remove(&pattern_uid) {
            self.ordered_pattern_uids.retain(|uid| pattern_uid != *uid);
            Ok(pattern)
        } else {
            Err(anyhow!("Pattern {pattern_uid} not found"))
        }
    }

    pub fn arrange_pattern(
        &mut self,
        track_uid: TrackUid,
        pattern_uid: PatternUid,
        position: MusicalTime,
    ) -> Result<()> {
        if let Some(pattern) = self.patterns.get(&pattern_uid) {
            self.tracks_to_arrangements
                .entry(track_uid)
                .or_default()
                .push(Arrangement {
                    pattern_uid,
                    position,
                    duration: pattern.duration(),
                });

            let sequencer = self.e.tracks_to_sequencers.entry(track_uid).or_default();
            sequencer.record(MidiChannel::default(), pattern, position)
        } else {
            Err(anyhow!("Pattern {pattern_uid} not found"))
        }
    }

    pub fn unarrange_pattern(
        &mut self,
        track_uid: TrackUid,
        pattern_uid: PatternUid,
        position: MusicalTime,
    ) {
        if let Some(arrangements) = self.tracks_to_arrangements.get_mut(&track_uid) {
            if let Some(pattern) = self.patterns.get(&pattern_uid) {
                let arrangement = Arrangement {
                    pattern_uid,
                    position,
                    duration: pattern.duration(),
                };
                arrangements.retain(|a| *a != arrangement);
                self.replay_arrangements();
            }
        }
    }

    fn replay_arrangements(&mut self) {
        self.e.tracks_to_sequencers.clear();
        self.tracks_to_arrangements.keys().for_each(|track_uid| {
            if let Some(arrangements) = self.tracks_to_arrangements.get(track_uid) {
                arrangements.iter().for_each(|arrangement| {
                    if let Some(pattern) = self.patterns.get(&arrangement.pattern_uid) {
                        let sequencer = self.e.tracks_to_sequencers.entry(*track_uid).or_default();
                        let _ =
                            sequencer.record(MidiChannel::default(), pattern, arrangement.position);
                    }
                });
            }
        });
    }

    /// Use like this:
    ///
    /// ```no_run
    /// use ensnare_cores::Composer;
    ///
    /// let composer = Composer::default();
    /// let mut composer_serial = 0;
    ///
    /// if composer.has_changed(&mut composer_serial) {
    ///     // Update local data
    /// } else {
    ///     // We're up to date, nothing to do    
    /// }
    /// ```
    pub fn has_changed(&self, last_known: &mut usize) -> bool {
        let has_changed = self.e.mod_serial.0 != *last_known;
        *last_known = self.e.mod_serial.0;
        has_changed
    }

    fn update_is_finished(&mut self) {
        self.e.is_finished = self
            .e
            .tracks_to_sequencers
            .values()
            .all(|s| s.is_finished());
    }
}
impl Controls for Composer {
    fn time_range(&self) -> Option<TimeRange> {
        Some(self.e.time_range.clone())
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.e
            .tracks_to_sequencers
            .values_mut()
            .for_each(|s| s.update_time_range(time_range));
        self.e.time_range = time_range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.is_performing() {
            // TODO: no duplicate time range detection
            // No note killer
            self.e
                .tracks_to_sequencers
                .iter_mut()
                .for_each(|(track_uid, sequencer)| {
                    sequencer.work(&mut |event| match event {
                        WorkEvent::Midi(channel, message) => {
                            control_events_fn(WorkEvent::MidiForTrack(
                                track_uid.clone(),
                                channel,
                                message,
                            ));
                        }
                        _ => control_events_fn(event),
                    });
                });
        }
        self.update_is_finished();
    }

    fn is_finished(&self) -> bool {
        self.e.is_finished
    }

    fn play(&mut self) {
        self.e.is_performing = true;
        self.update_is_finished();
    }

    fn stop(&mut self) {
        self.e.is_performing = false;
    }

    // TODO: this doesn't fit. Ignore here? Or problem with trait?
    fn skip_to_start(&mut self) {}

    fn is_performing(&self) -> bool {
        self.e.is_performing
    }
}
impl Serializable for Composer {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.replay_arrangements();
    }
}
