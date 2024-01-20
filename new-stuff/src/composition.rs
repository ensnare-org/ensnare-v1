// Copyright (c) 2024 Mike Tsao. All rights reserved.

use anyhow::{anyhow, Result};
use ensnare_core::{
    prelude::*,
    selection_set::SelectionSet,
    types::{ColorScheme, ModSerial},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// [Composer] owns the musical score. It doesn't know anything about
/// instruments that can help perform the score.
///
/// [Composer] defines several terms.
///
/// 1. A [Sequence] is a reusable series of notes. It can be of any length.
/// 2. A [Pattern] is a more constrained Sequence that has a [TimeSignature]. A
///    Pattern's length and granularity depends on the time signature. The
///    length is always a round number of bars/measures, and divisions are a
///    quarter of the time signature's note value.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Composer {
    #[serde(default)]
    pattern_uid_factory: UidFactory<PatternUid>,
    #[serde(default)]
    pub patterns: HashMap<PatternUid, Pattern>,
    #[serde(default)]
    pub ordered_pattern_uids: Vec<PatternUid>,
    #[serde(default)]
    pub tracks_to_arrangements: HashMap<TrackUid, Vec<Arrangement>>,
    #[serde(default)]
    pub pattern_color_schemes: Vec<(PatternUid, ColorScheme)>,

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
            self.tracks_to_arrangements
                .values_mut()
                .for_each(|v| v.retain(|a| a.pattern_uid != pattern_uid));
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
    /// use ensnare_core::composition::Composer;
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
        assert!(c.tracks_to_arrangements.is_empty());

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
        assert!(c.tracks_to_arrangements.is_empty());

        assert!(
            c.pattern(pattern_1_uid).is_some(),
            "Retrieving patterns works"
        );
        assert!(c.pattern(pattern_2_uid).is_some());
        assert!(
            c.pattern(PatternUid(9999999)).is_none(),
            "Retrieving a nonexistent pattern returns None"
        );

        let track_1_uid = TrackUid(1);
        let track_2_uid = TrackUid(2);
        let _ = c
            .arrange_pattern(track_1_uid, pattern_1_uid, MusicalTime::START)
            .unwrap();
        assert_eq!(
            c.tracks_to_arrangements.len(),
            1,
            "Arranging patterns works"
        );
        assert_eq!(c.tracks_to_arrangements.get(&track_1_uid).unwrap().len(), 1);
        let _ = c
            .arrange_pattern(track_1_uid, pattern_1_uid, MusicalTime::DURATION_WHOLE * 1)
            .unwrap();
        let _ = c
            .arrange_pattern(track_1_uid, pattern_1_uid, MusicalTime::DURATION_WHOLE * 2)
            .unwrap();
        assert_eq!(c.tracks_to_arrangements.len(), 1);
        assert_eq!(c.tracks_to_arrangements.get(&track_1_uid).unwrap().len(), 3);

        let _ = c
            .arrange_pattern(track_2_uid, pattern_2_uid, MusicalTime::DURATION_WHOLE * 3)
            .unwrap();
        let _ = c
            .arrange_pattern(track_2_uid, pattern_1_uid, MusicalTime::DURATION_WHOLE * 3)
            .unwrap();
        assert_eq!(
            c.tracks_to_arrangements.len(),
            2,
            "Arranging patterns across multiple tracks works"
        );
        assert_eq!(c.tracks_to_arrangements.get(&track_1_uid).unwrap().len(), 3);
        assert_eq!(c.tracks_to_arrangements.get(&track_2_uid).unwrap().len(), 2);

        c.unarrange_pattern(track_1_uid, pattern_1_uid, MusicalTime::START);
        assert_eq!(
            c.tracks_to_arrangements.get(&track_1_uid).unwrap().len(),
            2,
            "Removing an arrangement works"
        );

        let removed_pattern = c.remove_pattern(pattern_1_uid).unwrap();
        assert_eq!(removed_pattern.notes().len(), 1);
        assert_eq!(
            c.tracks_to_arrangements.get(&track_1_uid).unwrap().len(),
            0,
            "Removing a pattern should also unarrange it"
        );
        assert_eq!(c.tracks_to_arrangements.get(&track_2_uid).unwrap().len(), 1);
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