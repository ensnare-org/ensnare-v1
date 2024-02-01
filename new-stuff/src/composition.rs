// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::types::{ArrangementUid, ArrangementUidFactory};
use anyhow::{anyhow, Result};
use ensnare_core::{
    prelude::*,
    selection_set::SelectionSet,
    types::{ColorScheme, ModSerial},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumCount;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Arrangement {
    pub pattern_uid: PatternUid,
    pub position: MusicalTime,
    pub duration: MusicalTime,
}

/// [Composer] owns the musical score. It doesn't know anything about
/// instruments that can help perform the score.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Composer {
    #[serde(default)]
    pattern_uid_factory: PatternUidFactory,
    #[serde(default)]
    pub patterns: HashMap<PatternUid, Pattern>,
    #[serde(default)]
    pub ordered_pattern_uids: Vec<PatternUid>,

    #[serde(default)]
    arrangement_uid_factory: ArrangementUidFactory,
    #[serde(default)]
    pub arrangements: HashMap<ArrangementUid, Arrangement>,
    #[serde(default)]
    pub tracks_to_ordered_arrangement_uids: HashMap<TrackUid, Vec<ArrangementUid>>,

    /// A reverse mapping of patterns to arrangements, so that we know which
    /// arrangements to remove when a pattern is changed (TODO) or deleted.
    #[serde(default)]
    pub patterns_to_arrangements: HashMap<PatternUid, Vec<ArrangementUid>>,

    #[serde(default)]
    pub pattern_color_schemes: Vec<(PatternUid, ColorScheme)>,

    #[serde(skip)]
    pub e: ComposerEphemerals,
}

#[derive(Debug, Default)]
pub struct ComposerEphemerals {
    pub pattern_selection_set: SelectionSet<PatternUid>,
    pub arrangement_selection_set: SelectionSet<ArrangementUid>,

    tracks_to_sequencers: HashMap<TrackUid, PatternSequencer>,

    time_range: TimeRange,
    is_finished: bool,
    is_performing: bool,

    // This copy of the global time signature exists so that we have the right
    // default when we create a new Pattern.
    time_signature: TimeSignature,

    // Each time something changes in the repo, this number will change. Use the
    // provided methods to manage a local copy of it and decide whether to act.
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
            if let Some(arrangement_uids) = self.patterns_to_arrangements.get(&pattern_uid) {
                arrangement_uids.iter().for_each(|arrangement_uid| {
                    self.arrangements.remove(arrangement_uid);
                });
                self.tracks_to_ordered_arrangement_uids
                    .values_mut()
                    .for_each(|track_auids| {
                        // TODO: keep an eye on this; it's O(NxM)
                        track_auids.retain(|auid| !arrangement_uids.contains(auid));
                    });
                self.patterns_to_arrangements.remove(&pattern_uid); // see you soon borrow checker
            }
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
    ) -> Result<ArrangementUid> {
        if let Some(pattern) = self.patterns.get(&pattern_uid) {
            let arrangement_uid = self.arrangement_uid_factory.mint_next();
            self.arrangements.insert(
                arrangement_uid,
                Arrangement {
                    pattern_uid,
                    position,
                    duration: pattern.duration(),
                },
            );
            self.tracks_to_ordered_arrangement_uids
                .entry(track_uid)
                .or_default()
                .push(arrangement_uid);
            self.patterns_to_arrangements
                .entry(pattern_uid)
                .or_default()
                .push(arrangement_uid);

            let sequencer = self.e.tracks_to_sequencers.entry(track_uid).or_default();
            sequencer.record(MidiChannel::default(), pattern, position)?;
            Ok(arrangement_uid)
        } else {
            Err(anyhow!("Pattern {pattern_uid} not found"))
        }
    }

    pub fn unarrange(&mut self, track_uid: TrackUid, arrangement_uid: ArrangementUid) {
        if let Some(arrangements) = self.tracks_to_ordered_arrangement_uids.get_mut(&track_uid) {
            arrangements.retain(|a| *a != arrangement_uid);
            if let Some(arrangement) = self.arrangements.remove(&arrangement_uid) {
                self.patterns_to_arrangements
                    .entry(arrangement.pattern_uid)
                    .or_default()
                    .retain(|auid| *auid != arrangement_uid);
            }

            self.replay_arrangements();
        }
    }

    pub fn duplicate_arrangement(
        &mut self,
        track_uid: TrackUid,
        arrangement_uid: ArrangementUid,
    ) -> Result<ArrangementUid> {
        if let Some(arrangement) = self.arrangements.get(&arrangement_uid) {
            self.arrange_pattern(
                track_uid,
                arrangement.pattern_uid,
                arrangement.position + arrangement.duration,
            )
        } else {
            Err(anyhow!(
                "Arrangement at {track_uid}-{arrangement_uid} was missing"
            ))
        }
    }

    fn replay_arrangements(&mut self) {
        self.e.tracks_to_sequencers.clear();
        self.tracks_to_ordered_arrangement_uids
            .iter()
            .for_each(|(track_uid, arrangement_uids)| {
                let sequencer = self.e.tracks_to_sequencers.entry(*track_uid).or_default();
                arrangement_uids.iter().for_each(|arrangement_uid| {
                    if let Some(arrangement) = self.arrangements.get(arrangement_uid) {
                        if let Some(pattern) = self.patterns.get(&arrangement.pattern_uid) {
                            let _ = sequencer.record(
                                MidiChannel::default(),
                                pattern,
                                arrangement.position,
                            );
                        }
                    }
                })
            });
    }

    /// Use like this:
    ///
    /// ```no_run
    /// use ensnare_new_stuff::Composer;
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

    fn gather_pattern_color_schemes(&mut self) {
        self.pattern_color_schemes =
            self.patterns
                .iter()
                .fold(Vec::default(), |mut v, (pattern_uid, pattern)| {
                    v.push((*pattern_uid, pattern.color_scheme));
                    v
                });
        self.pattern_color_schemes.sort();
    }

    fn distribute_pattern_color_schemes(&mut self) {
        self.pattern_color_schemes
            .iter()
            .for_each(|(pattern_uid, color_scheme)| {
                if let Some(pattern) = self.patterns.get_mut(pattern_uid) {
                    pattern.color_scheme = *color_scheme;
                }
            });
    }

    pub fn suggest_next_pattern_color_scheme(&self) -> ColorScheme {
        ColorScheme::from_repr(self.patterns.len() % ColorScheme::COUNT).unwrap_or_default()
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
    fn before_ser(&mut self) {
        self.gather_pattern_color_schemes();
    }

    fn after_deser(&mut self) {
        self.distribute_pattern_color_schemes();
        self.replay_arrangements();
    }
}
impl Configurable for Composer {
    fn time_signature(&self) -> TimeSignature {
        self.e.time_signature
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.e.time_signature = time_signature;
    }
}
impl HasExtent for Composer {
    fn extent(&self) -> TimeRange {
        let extent = self.e.tracks_to_sequencers.values().fold(
            TimeRange::default(),
            |mut extent, sequencer| {
                extent.expand_with_range(&sequencer.extent());
                extent
            },
        );
        extent
    }

    fn set_extent(&mut self, _: TimeRange) {
        eprintln!("Composer::set_extent() should never be called");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_pattern_crud() {
        let mut c = Composer::default();
        assert!(
            c.ordered_pattern_uids.is_empty(),
            "Default Composer is empty"
        );
        assert!(c.patterns.is_empty());
        assert!(c.tracks_to_ordered_arrangement_uids.is_empty());
        assert!(c.arrangements.is_empty());

        let pattern_1_uid = c
            .add_pattern(
                PatternBuilder::default()
                    .note(Note::new_with_midi_note(
                        MidiNote::A4,
                        MusicalTime::START,
                        MusicalTime::DURATION_QUARTER,
                    ))
                    .build()
                    .unwrap(),
                None,
            )
            .unwrap();
        let pattern_2_uid = c
            .add_pattern(PatternBuilder::default().build().unwrap(), None)
            .unwrap();
        assert_eq!(c.ordered_pattern_uids.len(), 2, "Creating patterns works");
        assert_eq!(c.patterns.len(), 2);
        assert!(c.tracks_to_ordered_arrangement_uids.is_empty());
        assert!(c.arrangements.is_empty());

        assert!(
            c.patterns.get(&pattern_1_uid).is_some(),
            "Retrieving patterns works"
        );
        assert!(c.patterns.get(&pattern_2_uid).is_some());
        assert!(
            c.patterns.get(&PatternUid(9999999)).is_none(),
            "Retrieving a nonexistent pattern returns None"
        );

        let track_1_uid = TrackUid(1);
        let track_2_uid = TrackUid(2);
        let _ = c
            .arrange_pattern(track_1_uid, pattern_1_uid, MusicalTime::START)
            .unwrap();
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids.len(),
            1,
            "Arranging patterns works"
        );
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids
                .get(&track_1_uid)
                .unwrap()
                .len(),
            1
        );
        let arrangement_1_uid = c
            .arrange_pattern(track_1_uid, pattern_1_uid, MusicalTime::DURATION_WHOLE * 1)
            .unwrap();
        let arrangement_2_uid = c
            .arrange_pattern(track_1_uid, pattern_1_uid, MusicalTime::DURATION_WHOLE * 2)
            .unwrap();
        assert_eq!(c.tracks_to_ordered_arrangement_uids.len(), 1);
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids
                .get(&track_1_uid)
                .unwrap()
                .len(),
            3
        );

        let _ = c
            .arrange_pattern(track_2_uid, pattern_2_uid, MusicalTime::DURATION_WHOLE * 3)
            .unwrap();
        let arrangement_4_uid = c
            .arrange_pattern(track_2_uid, pattern_1_uid, MusicalTime::DURATION_WHOLE * 3)
            .unwrap();
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids.len(),
            2,
            "Arranging patterns across multiple tracks works"
        );
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids
                .get(&track_1_uid)
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids
                .get(&track_2_uid)
                .unwrap()
                .len(),
            2
        );

        c.unarrange(track_1_uid, arrangement_1_uid);
        assert!(
            c.arrangements.get(&arrangement_1_uid).is_none(),
            "Unarranging should remove the arrangement"
        );
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids
                .get(&track_1_uid)
                .unwrap()
                .len(),
            2,
            "Unarranging should remove only the specified arrangment"
        );

        let removed_pattern = c.remove_pattern(pattern_1_uid).unwrap();
        assert_eq!(removed_pattern.notes().len(), 1);
        assert!(
            c.arrangements.get(&arrangement_2_uid).is_none(),
            "Removing a pattern should remove all arrangements using it"
        );
        assert!(
            c.arrangements.get(&arrangement_4_uid).is_none(),
            "Removing a pattern should remove all arrangements using it"
        );
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids
                .get(&track_1_uid)
                .unwrap()
                .len(),
            0,
            "tracks_to_ordered_arrangement_uids bookkeeping"
        );
        assert_eq!(
            c.tracks_to_ordered_arrangement_uids
                .get(&track_2_uid)
                .unwrap()
                .len(),
            1,
            "tracks_to_ordered_arrangement_uids bookkeeping"
        );
    }

    #[test]
    fn pattern_color_schemes() {
        let mut c = Composer::default();
        let p = PatternBuilder::default()
            .color_scheme(ColorScheme::Cerulean)
            .build()
            .unwrap();
        let puid = c.add_pattern(p, None).unwrap();

        assert!(c.pattern_color_schemes.is_empty());
        c.before_ser();
        assert_eq!(c.pattern_color_schemes.len(), 1);

        let _ = c.remove_pattern(puid);
        assert_eq!(c.pattern_color_schemes.len(), 1);
        c.before_ser();
        assert!(c.pattern_color_schemes.is_empty());
    }
}
