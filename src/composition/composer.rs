// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{
    composition::{ArrangementUid, ArrangementUidFactory},
    orchestration::TrackUid,
    prelude::*,
    traits::MidiNoteLabelMetadata,
    types::ColorScheme,
    util::Rng,
};
use anyhow::{anyhow, Result};
use core::ops::Range;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::EnumCount;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Arrangement {
    pub pattern_uid: PatternUid,
    pub midi_channel: MidiChannel,
    pub position: MusicalTime,
    pub duration: MusicalTime,
}
impl HasExtent for Arrangement {
    fn extent(&self) -> TimeRange {
        TimeRange(self.position..self.position + self.duration)
    }

    fn set_extent(&mut self, extent: TimeRange) {
        self.position = extent.start();
        self.duration = extent.duration();
    }
}

/// [Composer] owns the musical score. It doesn't know anything about
/// instruments that can help perform the score.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Composer {
    #[serde(default)]
    pattern_uid_factory: PatternUidFactory,
    #[serde(default)]
    pub patterns: FxHashMap<PatternUid, Pattern>,
    #[serde(default)]
    pub ordered_pattern_uids: Vec<PatternUid>,

    #[serde(default)]
    arrangement_uid_factory: ArrangementUidFactory,
    #[serde(default)]
    pub arrangements: FxHashMap<ArrangementUid, Arrangement>,
    #[serde(default)]
    pub tracks_to_ordered_arrangement_uids: FxHashMap<TrackUid, Vec<ArrangementUid>>,

    /// A reverse mapping of patterns to arrangements, so that we know which
    /// arrangements to remove when a pattern is changed (TODO) or deleted.
    #[serde(default)]
    pub patterns_to_arrangements: FxHashMap<PatternUid, Vec<ArrangementUid>>,

    #[serde(default)]
    pub pattern_color_schemes: Vec<(PatternUid, ColorScheme)>,

    #[serde(skip)]
    pub e: ComposerEphemerals,
}

#[derive(Debug, Default)]
pub struct ComposerEphemerals {
    // This RNG shouldn't be repeatable. Composer should normally generate truly
    // new elements each time.
    pub rng: Rng,

    tracks_to_sequencers: FxHashMap<TrackUid, PatternSequencer>,

    time_range: TimeRange,
    is_finished: bool,
    is_performing: bool,

    // This copy of the global time signature exists so that we have the right
    // default when we create a new Pattern.
    time_signature: TimeSignature,

    // Each time something changes in the repo, this number will change. Use the
    // provided methods to manage a local copy of it and decide whether to act.
    mod_serial: ModSerial,

    // The visible portion of the pattern editor.
    pub(crate) editor_bounds_x: Range<MusicalTime>,
    pub(crate) editor_bounds_y: Range<MidiNote>,

    // Which pattern, if any, is being edited right now.
    pub edited_pattern: Option<PatternUid>,

    pub midi_note_label_metadata: Option<Arc<MidiNoteLabelMetadata>>,
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
        midi_channel: MidiChannel,
        position: MusicalTime,
    ) -> Result<ArrangementUid> {
        if let Some(pattern) = self.patterns.get(&pattern_uid) {
            if !self.is_arrangement_area_available(
                track_uid,
                &pattern.extent().translate_to(position),
                None,
            ) {
                return Err(anyhow!("Pattern {pattern_uid} at position {position} would overlap with existing arrangement"));
            }

            let arrangement_uid = self.arrangement_uid_factory.mint_next();
            self.arrangements.insert(
                arrangement_uid,
                Arrangement {
                    pattern_uid,
                    midi_channel,
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
            sequencer.record(midi_channel, pattern, position)?;
            Ok(arrangement_uid)
        } else {
            Err(anyhow!("Pattern {pattern_uid} not found"))
        }
    }

    /// Sets a new position for an existing arrangement. Optionally duplicates
    /// the existing arrangement, leaves it in place, and creates a new one in
    /// the new position.
    pub fn move_arrangement(
        &mut self,
        track_uid: TrackUid,
        arrangement_uid: ArrangementUid,
        new_position: MusicalTime,
        copy_original: bool,
    ) -> Result<ArrangementUid> {
        if let Some(_arrangement_uids) = self.tracks_to_ordered_arrangement_uids.get_mut(&track_uid)
        {
            if let Some(arrangement) = self.arrangements.get(&arrangement_uid) {
                if copy_original {
                    self.arrange_pattern(
                        track_uid,
                        arrangement.pattern_uid,
                        arrangement.midi_channel,
                        new_position,
                    )
                } else {
                    let new_extent = arrangement.extent().translate_to(new_position);
                    if !self.is_arrangement_area_available(
                        track_uid,
                        &new_extent,
                        Some(arrangement_uid),
                    ) {
                        return Err(anyhow!("Moving arrangement {arrangement_uid} to {new_extent:?} would overlap with existing arrangement"));
                    }

                    // We have to look this up twice, first immutably, then _mut, to
                    // keep the borrow checker happy.
                    if let Some(arrangement) = self.arrangements.get_mut(&arrangement_uid) {
                        arrangement.position = new_position;
                        self.replay_arrangements();
                    }
                    Ok(arrangement_uid)
                }
            } else {
                Err(anyhow!("Arrangement {arrangement_uid} not found"))
            }
        } else {
            Err(anyhow!(
                "Arrangement {arrangement_uid} not found in track {track_uid}"
            ))
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
                arrangement.midi_channel,
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
    /// use ensnare_v1::Composer;
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

    // Composer enforces that no patterns should overlap in the arrangement.
    // Optionally supply a single ArrangementUid that is to be excluded from the
    // test set, which is useful if you are moving an existing arrangement and
    // don't care if it overlaps with itself. TODO: this will be tricky if the
    // user arranges a pattern and then edits it later!
    fn is_arrangement_area_available(
        &self,
        track_uid: TrackUid,
        pattern_extent: &TimeRange,
        arrangement_to_skip: Option<ArrangementUid>,
    ) -> bool {
        if let Some(arrangement_uids) = self.tracks_to_ordered_arrangement_uids.get(&track_uid) {
            arrangement_uids
                .iter()
                .filter(|auid| Some(*auid) != arrangement_to_skip.as_ref())
                .all(|auid| {
                    if let Some(arrangement) = self.arrangements.get(auid) {
                        !(pattern_extent.overlaps(arrangement.extent()))
                    } else {
                        true
                    }
                })
        } else {
            true
        }
    }

    pub(crate) fn clear_edited_pattern(&mut self) {
        self.e.edited_pattern = None;
    }

    pub(crate) fn set_edited_pattern(&mut self, pattern_uid: PatternUid) {
        self.e.edited_pattern = Some(pattern_uid);
    }

    pub(crate) fn set_midi_note_label_metadata(
        &mut self,
        midi_note_label_metadata: &Arc<MidiNoteLabelMetadata>,
    ) {
        self.e.midi_note_label_metadata = Some(Arc::clone(midi_note_label_metadata));
    }

    pub(crate) fn clear_midi_note_label_metadata(&mut self) {
        self.e.midi_note_label_metadata = None;
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
        self.clear_midi_note_label_metadata();
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
        // This is placed far out in the track so that it doesn't interfere with
        // subsequent pattern arrangements. We want it to stay in the
        // arrangement because we use it when we're testing pattern deletion.
        let _ = c
            .arrange_pattern(
                track_1_uid,
                pattern_1_uid,
                MidiChannel::default(),
                MusicalTime::ONE_BEAT * 16,
            )
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
            .arrange_pattern(
                track_1_uid,
                pattern_1_uid,
                MidiChannel::default(),
                MusicalTime::DURATION_WHOLE * 1,
            )
            .unwrap();
        let arrangement_2_uid = c
            .arrange_pattern(
                track_1_uid,
                pattern_1_uid,
                MidiChannel::default(),
                MusicalTime::DURATION_WHOLE * 1 + MusicalTime::ONE_BEAT * 4,
            )
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
            .arrange_pattern(
                track_2_uid,
                pattern_2_uid,
                MidiChannel::default(),
                MusicalTime::DURATION_WHOLE * 3,
            )
            .unwrap();
        let arrangement_4_uid = c
            .arrange_pattern(
                track_2_uid,
                pattern_1_uid,
                MidiChannel::default(),
                MusicalTime::DURATION_WHOLE * 3 + MusicalTime::ONE_BEAT * 4,
            )
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

    #[test]
    fn composer_durations() {
        let mut rng = Rng::new_with_seed(42);
        let track_uid = TrackUid(1);
        let mut c = Composer::default();
        assert_eq!(
            c.duration(),
            MusicalTime::TIME_ZERO,
            "New composer should have duration zero"
        );

        let p1 = PatternBuilder::default().random(&mut rng).build().unwrap();
        let p1_duration = p1.duration();
        let p2 = PatternBuilder::default().random(&mut rng).build().unwrap();
        let p2_duration = p2.duration();

        let puid1 = c.add_pattern(p1, None).unwrap();
        let puid2 = c.add_pattern(p2, None).unwrap();

        let a1 = c
            .arrange_pattern(track_uid, puid1, MidiChannel::default(), MusicalTime::START)
            .unwrap();
        assert_eq!(c.duration(), p1_duration, "After adding one pattern at start, composer's duration should equal pattern's duration");
        let a2 = c
            .arrange_pattern(
                track_uid,
                puid2,
                MidiChannel::default(),
                MusicalTime::START + p1_duration,
            )
            .unwrap();
        assert_eq!(c.duration(), p1_duration + p2_duration, "After adding two consecutive normal-sized patterns, composer's duration should equal their durations' sum");

        c.unarrange(track_uid, a2);
        assert_eq!(c.duration(), p1_duration, "After removing last pattern in arrangement, composer duration should shrink appropriately");
        c.unarrange(track_uid, a1);
        assert_eq!(
            c.duration(),
            MusicalTime::TIME_ZERO,
            "After removing only pattern, composer duration should return to zero"
        );
    }

    #[test]
    fn composer_disallows_overlapping_patterns() {
        let mut rng = Rng::new_with_seed(42);
        let mut c = Composer::default();

        let p1 = PatternBuilder::default().random(&mut rng).build().unwrap();
        let p1_duration = p1.duration();
        let p2 = PatternBuilder::default().random(&mut rng).build().unwrap();
        let p2_duration = p2.duration();
        let p3 = PatternBuilder::default().random(&mut rng).build().unwrap();
        let _p3_duration = p3.duration();

        let puid1 = c.add_pattern(p1, None).unwrap();
        let puid2 = c.add_pattern(p2, None).unwrap();
        let puid3 = c.add_pattern(p3, None).unwrap();
        let track_uid = TrackUid(1);

        let a1 = c
            .arrange_pattern(track_uid, puid1, MidiChannel::default(), MusicalTime::START)
            .unwrap();
        let a2 = c
            .arrange_pattern(track_uid, puid2, MidiChannel::default(), p1_duration)
            .unwrap();
        assert_eq!(
            c.duration(),
            p1_duration + p2_duration,
            "Sanity check: composer duration == duration of two arranged patterns"
        );
        assert!(c
            .arrange_pattern(
                track_uid,
                puid3,
                MidiChannel::default(),
                p1_duration + p2_duration - MusicalTime::ONE_UNIT
            )
            .is_err(), "Composer should disallow arrangement of pattern whose start is within another arranged pattern.");

        c.unarrange(track_uid, a1);
        assert!(
            c.arrange_pattern(track_uid, puid3, MidiChannel::default(), MusicalTime::START + MusicalTime::ONE_UNIT)
                .is_err(),
            "Composer should disallow arrangement of pattern whose extent crosses a later pattern's start."
        );

        c.unarrange(track_uid, a2);
        let a3_result = c.arrange_pattern(
            track_uid,
            puid3,
            MidiChannel::default(),
            MusicalTime::START + MusicalTime::ONE_UNIT,
        );
        assert!(a3_result.is_ok(), "Composer should allow arrangement of pattern in area formerly occupied by since-unarranged patterns.");
    }

    #[test]
    fn composer_detects_overlaps_after_pattern_edit() {
        // TODO: we don't handle this yet.
    }
}
