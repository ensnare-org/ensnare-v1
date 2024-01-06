// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Cores are basic musical devices.

use anyhow::{anyhow, Result};
use ensnare_core::{
    piano_roll::{Pattern, PatternUid},
    prelude::*,
    traits::Sequences,
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Composer {
    pattern_uid_factory: UidFactory<PatternUid>,
    patterns: HashMap<PatternUid, Pattern>,
    arrangements: HashMap<TrackUid, Vec<Arrangement>>,

    #[serde(skip)]
    inner: PatternSequencer,

    #[serde(skip)]
    time_range: TimeRange,
    #[serde(skip)]
    is_performing: bool,

    // Each time something changes in the repo, this number will change. Use the
    // provided methods to manage a local copy of it and decide whether to act.
    #[serde(skip)]
    mod_serial: ModSerial,
}
impl Composer {
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
        Ok(pattern_uid)
    }

    pub fn pattern(&self, pattern_uid: &PatternUid) -> Option<&Pattern> {
        self.patterns.get(pattern_uid)
    }

    pub fn pattern_mut(&mut self, pattern_uid: &PatternUid) -> Option<&mut Pattern> {
        self.patterns.get_mut(pattern_uid)
    }

    pub fn notify_pattern_change(&mut self) {
        self.replay_arrangements();
    }

    pub fn remove_pattern(&mut self, pattern_uid: PatternUid) -> Result<Pattern> {
        if let Some(pattern) = self.patterns.remove(&pattern_uid) {
            Ok(pattern)
        } else {
            Err(anyhow!("Pattern {pattern_uid} not found"))
        }
    }

    pub fn arrange_pattern(
        &mut self,
        track_uid: &TrackUid,
        pattern_uid: &PatternUid,
        position: MusicalTime,
    ) -> Result<()> {
        if let Some(pattern) = self.patterns.get(pattern_uid) {
            self.arrangements
                .entry(*track_uid)
                .or_default()
                .push(Arrangement {
                    pattern_uid: *pattern_uid,
                    position,
                });

            // TODO: we're not remembering the track
            self.inner.record(MidiChannel::default(), pattern, position)
        } else {
            Err(anyhow!("Pattern {pattern_uid} not found"))
        }
    }

    pub fn unarrange_pattern(
        &mut self,
        track_uid: &TrackUid,
        pattern_uid: &PatternUid,
        position: MusicalTime,
    ) {
        if let Some(arrangements) = self.arrangements.get_mut(track_uid) {
            let arrangement = Arrangement {
                pattern_uid: *pattern_uid,
                position,
            };
            arrangements.retain(|a| *a != arrangement);
            self.replay_arrangements();
        }
    }

    fn replay_arrangements(&mut self) {
        self.inner.clear();
        self.arrangements.values().for_each(|arrangements| {
            arrangements.iter().for_each(|a| {
                if let Some(pattern) = self.patterns.get(&a.pattern_uid) {
                    let _ = self
                        .inner
                        .record(MidiChannel::default(), pattern, a.position);
                }
            })
        })
    }

    /// Use like this:
    ///
    /// ```no_run
    /// use crate::Composer;
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
        let has_changed = self.mod_serial.0 != *last_known;
        *last_known = self.mod_serial.0;
        has_changed
    }
}
impl Controls for Composer {
    fn time_range(&self) -> Option<TimeRange> {
        Some(self.time_range.clone())
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.inner.update_time_range(time_range);
        self.time_range = time_range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.is_performing() {
            // TODO: no duplicate time range detection
            // No note killer
            self.inner.work(control_events_fn);
        }
    }

    fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    // TODO: this doesn't fit. Ignore here? Or problem with trait?
    fn skip_to_start(&mut self) {}

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Serializable for Composer {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.replay_arrangements();
    }
}
