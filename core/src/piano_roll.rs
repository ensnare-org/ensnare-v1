// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::rng::Rng;
use crate::{
    prelude::*,
    selection_set::SelectionSet,
    traits::TimeRange,
    uid::{IsUid, UidFactory},
};
use anyhow::anyhow;
use derive_builder::Builder;
use ensnare_proc_macros::Params;
use std::{collections::HashMap, fmt::Display, ops::Add, sync::atomic::AtomicUsize};

/// Identifies a [Pattern].
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PatternUid(pub usize);
impl IsUid for PatternUid {}
impl From<usize> for PatternUid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for PatternUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
pub type PatternUidFactory = UidFactory<PatternUid>;
impl UidFactory<PatternUid> {
    pub const FIRST_UID: AtomicUsize = AtomicUsize::new(1);
}
impl Default for UidFactory<PatternUid> {
    fn default() -> Self {
        Self {
            next_uid_value: Self::FIRST_UID,
            _phantom: Default::default(),
        }
    }
}

/// A [Note] is a single played note. It knows which key it's playing (which
/// is more or less assumed to be a MIDI key value), and when (start/end) it's
/// supposed to play, relative to time zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    /// The MIDI key code for the note. 69 is (usually) A4.
    pub key: u8,
    /// The range of time when this note should play.
    pub range: TimeRange,
}
impl Note {
    /// Creates a [Note] from a u8.
    pub const fn new_with(key: u8, start: MusicalTime, duration: MusicalTime) -> Self {
        let end = MusicalTime::new_with_units(start.total_units() + duration.total_units());
        Self {
            key,
            range: TimeRange(start..end),
        }
    }

    /// Creates a [Note] from a [MidiNote].
    pub fn new_with_midi_note(key: MidiNote, start: MusicalTime, duration: MusicalTime) -> Self {
        Self {
            key: key as u8,
            range: TimeRange(start..(start + duration)),
        }
    }
}
impl Add<MusicalTime> for Note {
    type Output = Self;

    fn add(self, rhs: MusicalTime) -> Self::Output {
        Self {
            key: self.key,
            range: TimeRange((self.range.0.start + rhs)..(self.range.0.end + rhs)),
        }
    }
}
// TODO: I don't think this is the best choice to expose this idea. If there's a
// way to do it as an iterator, so that we don't always have to create a Vec,
// that would probably be better.
impl Into<Vec<MidiEvent>> for Note {
    fn into(self) -> Vec<MidiEvent> {
        vec![
            MidiEvent {
                message: MidiMessage::NoteOn {
                    key: u7::from(self.key),
                    vel: u7::from(127),
                },
                time: self.range.0.start,
            },
            MidiEvent {
                message: MidiMessage::NoteOff {
                    key: u7::from(self.key),
                    vel: u7::from(127),
                },
                time: self.range.0.end,
            },
        ]
    }
}

/// A [Pattern] contains a musical sequence that is suitable for
/// pattern-based composition. It is a series of [Note]s and a
/// [TimeSignature]. All the notes should fit into the pattern's duration, and
/// the duration should be a round multiple of the length implied by the time
/// signature.
#[derive(Clone, Debug, Builder, PartialEq)]
#[builder(build_fn(private, name = "build_from_builder"))]
pub struct Pattern {
    /// The pattern's [TimeSignature].
    #[builder(default)]
    time_signature: TimeSignature,

    /// The duration is the amount of time from the start of the pattern to the
    /// point when the next pattern should start. This does not necessarily mean
    /// the time between the first note-on and the first note-off! For example,
    /// an empty 4/4 pattern lasts for 4 beats.
    #[builder(setter(skip))]
    pub duration: MusicalTime,

    /// The notes that make up this pattern. When it is in a [Pattern], a
    /// [Note]'s range is relative to the start of the [Pattern]. For example, a
    /// note that plays immediately would have a range start of zero. TODO:
    /// specify any ordering restrictions.
    #[builder(default, setter(each(name = "note", into)))]
    pub notes: Vec<Note>,
    // TODO: Nobody is writing to this. I haven't implemented selection
    // operations on notes yet.
    //     // #[builder(setter(skip))]
    // note_selection_set: HashSet<usize>,
}
impl PatternBuilder {
    /// The length of a note generated by the random() method
    pub const DURATION: MusicalTime = MusicalTime::DURATION_QUARTER;

    /// Builds the [Pattern].
    pub fn build(&self) -> Result<Pattern, PatternBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    fn random(&mut self) -> &mut Self {
        let mut rng = Rng::default();

        for _ in 0..rng.rand_range(8..16) {
            let start = MusicalTime::new_with_parts(rng.rand_range(0..64) as usize);
            let duration = Self::DURATION;
            self.note(Note {
                key: rng.rand_range(32..96) as u8,
                range: TimeRange(start..start + duration),
            });
        }
        self
    }

    /// Given a sequence of MIDI note numbers and an optional grid value that
    /// overrides the one implied by the time signature, adds [Note]s one after
    /// another into the pattern. The value 255 is reserved for rest (no note,
    /// or silence).
    ///
    /// The optional grid_value is similar to the time signature's bottom value
    /// (1 is a whole note, 2 is a half note, etc.). For example, for a 4/4
    /// pattern, None means each note number produces a quarter note, and we
    /// would provide sixteen note numbers to fill the pattern with 4 beats of
    /// four quarter-notes each. For a 4/4 pattern, Some(8) means each note
    /// number should produce an eighth note., and 4 x 8 = 32 note numbers would
    /// fill the pattern.
    ///
    /// If midi_note_numbers contains fewer than the maximum number of note
    /// numbers for the grid value, then the rest of the pattern is silent.
    pub fn note_sequence(
        &mut self,
        midi_note_numbers: Vec<u8>,
        grid_value: Option<usize>,
    ) -> &mut Self {
        let grid_value = grid_value.unwrap_or(self.time_signature.unwrap_or_default().bottom);
        let mut position = MusicalTime::START;
        let position_delta = MusicalTime::new_with_fractional_beats(1.0 / grid_value as f64);
        for note in midi_note_numbers {
            if note != 255 {
                self.note(Note {
                    key: note,
                    range: TimeRange(position..position + position_delta),
                });
            }
            position += position_delta;
        }
        self
    }
}
impl Default for Pattern {
    fn default() -> Self {
        let mut r = Self {
            time_signature: TimeSignature::default(),
            duration: Default::default(),
            notes: Default::default(),
            // note_selection_set: Default::default(),
        };
        r.after_deser();
        r
    }
}
impl Serializable for Pattern {
    fn after_deser(&mut self) {
        self.refresh_internals();
    }
}
impl Add<MusicalTime> for Pattern {
    type Output = Self;

    fn add(self, rhs: MusicalTime) -> Self::Output {
        Self {
            time_signature: self.time_signature,
            duration: self.duration,
            notes: self.notes.iter().map(|note| note.clone() + rhs).collect(),
        }
    }
}
impl Into<Vec<MidiEvent>> for Pattern {
    fn into(self) -> Vec<MidiEvent> {
        self.notes.iter().fold(Vec::default(), |mut v, note| {
            let mut note_as_events: Vec<MidiEvent> = note.clone().into();
            v.append(&mut note_as_events);
            v
        })
    }
}

impl Pattern {
    /// Returns the number of notes in the pattern.
    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    /// Returns the pattern grid's number of subdivisions, which is calculated
    /// from the time signature. The number is simply the time signature's top x
    /// bottom. For example, a 3/4 pattern will have 12 subdivisions (three
    /// beats per measure, each beat divided into four quarter notes).
    ///
    /// This is just a UI default and doesn't affect the actual granularity of a
    /// note position.
    pub fn default_grid_value(&self) -> usize {
        self.time_signature.top * self.time_signature.bottom
    }

    fn refresh_internals(&mut self) {
        let final_event_time = self
            .notes
            .iter()
            .map(|n| n.range.0.end)
            .max()
            .unwrap_or_default();

        // This is how we deal with std::ops::Range<> being inclusive start, exclusive
        // end. It matters because we want the calculated duration to be rounded
        // up to the next measure, but we don't want a note-off event right on
        // the edge to extend that calculation to include another bar.
        let final_event_time = if final_event_time == MusicalTime::START {
            final_event_time
        } else {
            final_event_time - MusicalTime::new_with_units(1)
        };
        let beats = final_event_time.total_beats();
        let top = self.time_signature.top;
        let rounded_up_bars = (beats + top) / top;
        self.duration = MusicalTime::new_with_bars(&self.time_signature, rounded_up_bars);
    }

    /// Adds a note to this pattern. Does not check for duplicates. It's OK to
    /// add notes in any time order.
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
        self.refresh_internals();
    }

    /// Removes all notes matching this one in this pattern.
    pub fn remove_note(&mut self, note: &Note) {
        self.notes.retain(|v| v != note);
        self.refresh_internals();
    }

    /// Removes all notes in this pattern.
    pub fn clear(&mut self) {
        self.notes.clear();
        self.refresh_internals();
    }

    /// This pattern's duration in [MusicalTime].
    pub fn duration(&self) -> MusicalTime {
        self.duration
    }

    /// Sets a new start time for all notes in the Pattern matching the given
    /// [Note]. If any are found, returns the new version.
    pub fn move_note(&mut self, note: &Note, new_start: MusicalTime) -> anyhow::Result<Note> {
        let mut new_note = note.clone();
        let new_note_length = new_note.range.0.end - new_note.range.0.start;
        new_note.range = TimeRange(new_start..new_start + new_note_length);
        self.replace_note(note, new_note)
    }

    /// Sets a new start time and duration for all notes in the Pattern matching
    /// the given [Note]. If any are found, returns the new version.
    pub fn move_and_resize_note(
        &mut self,
        note: &Note,
        new_start: MusicalTime,
        duration: MusicalTime,
    ) -> anyhow::Result<Note> {
        let mut new_note = note.clone();
        new_note.range = TimeRange(new_start..new_start + duration);
        self.replace_note(note, new_note)
    }

    /// Sets a new key for all notes in the Pattern matching the given [Note].
    /// If any are found, returns the new version.
    pub fn change_note_key(&mut self, note: &Note, new_key: u8) -> anyhow::Result<Note> {
        let mut new_note = note.clone();
        new_note.key = new_key;
        self.replace_note(note, new_note)
    }

    /// Replaces all notes in the Pattern matching the given [Note] with a new
    /// [Note]. If any are found, returns the new version.
    pub fn replace_note(&mut self, note: &Note, new_note: Note) -> anyhow::Result<Note> {
        let mut found = false;

        self.notes.iter_mut().filter(|n| n == &note).for_each(|n| {
            *n = new_note.clone();
            found = true;
        });
        if found {
            self.refresh_internals();
            Ok(new_note)
        } else {
            Err(anyhow!("replace_note: couldn't find note {:?}", note))
        }
    }

    #[allow(missing_docs)]
    pub fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    /// Returns a read-only slice of all the [Note]s in this pattern. No order
    /// is currently defined.
    pub fn notes(&self) -> &[Note] {
        self.notes.as_ref()
    }
}

/// [PianoRoll] manages all [Pattern]s.
#[derive(Debug, Params, PartialEq)]
pub struct PianoRoll {
    uid_factory: PatternUidFactory,
    pub uids_to_patterns: HashMap<PatternUid, Pattern>,
    pub ordered_pattern_uids: Vec<PatternUid>,
    pub pattern_selection_set: SelectionSet<PatternUid>,
}
impl Default for PianoRoll {
    fn default() -> Self {
        let mut r = Self {
            uid_factory: Default::default(),
            uids_to_patterns: Default::default(),
            ordered_pattern_uids: Default::default(),
            pattern_selection_set: Default::default(),
        };
        for _ in 0..16 {
            let _ = r.insert(PatternBuilder::default().random().build().unwrap());
        }
        r
    }
}
impl PianoRoll {
    /// Adds a [Pattern]. Returns its [PatternUid].
    pub fn insert(&mut self, pattern: Pattern) -> PatternUid {
        let uid = self.uid_factory.mint_next();
        self.uids_to_patterns.insert(uid, pattern);
        self.ordered_pattern_uids.push(uid);
        uid
    }

    /// Removes the [Pattern] having the given [PatternUid], if any.
    pub fn remove(&mut self, pattern_uid: &PatternUid) {
        self.uids_to_patterns.remove(pattern_uid);
        self.ordered_pattern_uids.retain(|uid| uid != pattern_uid);
    }

    /// Returns a reference to the specified [Pattern].
    pub fn get_pattern(&self, pattern_uid: &PatternUid) -> Option<&Pattern> {
        self.uids_to_patterns.get(pattern_uid)
    }

    /// Returns a mutable reference to the specified [Pattern].
    pub fn get_pattern_mut(&mut self, pattern_uid: &PatternUid) -> Option<&mut Pattern> {
        self.uids_to_patterns.get_mut(pattern_uid)
    }
}

// TODO: move back to tests mod when everything is integrated
impl PianoRoll {
    /// For testing only; adds simple patterns.
    pub fn populate_pattern(&mut self, pattern_number: usize) -> (PatternUid, usize, MusicalTime) {
        let pattern = match pattern_number {
            0 => PatternBuilder::default()
                .notes(vec![
                    Note::new_with_midi_note(
                        MidiNote::C4,
                        MusicalTime::TIME_ZERO,
                        MusicalTime::DURATION_WHOLE,
                    ),
                    Note::new_with_midi_note(
                        MidiNote::D4,
                        MusicalTime::TIME_END_OF_FIRST_BEAT,
                        MusicalTime::DURATION_WHOLE,
                    ),
                    Note::new_with_midi_note(
                        MidiNote::E4,
                        MusicalTime::TIME_END_OF_FIRST_BEAT * 2,
                        MusicalTime::DURATION_WHOLE,
                    ),
                ])
                .build(),
            1 => PatternBuilder::default()
                .notes(vec![
                    Note::new_with_midi_note(
                        MidiNote::C5,
                        MusicalTime::TIME_ZERO,
                        MusicalTime::DURATION_WHOLE,
                    ),
                    Note::new_with_midi_note(
                        MidiNote::D5,
                        MusicalTime::TIME_END_OF_FIRST_BEAT,
                        MusicalTime::DURATION_WHOLE,
                    ),
                    Note::new_with_midi_note(
                        MidiNote::E5,
                        MusicalTime::TIME_END_OF_FIRST_BEAT * 2,
                        MusicalTime::DURATION_WHOLE,
                    ),
                ])
                .build(),
            _ => panic!(),
        }
        .unwrap();

        // Optimize this. I dare you.
        let len = pattern.notes().len();
        let duration = pattern.duration();
        (self.insert(pattern), len, duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Note {
        /// half-note
        const TEST_C4: Note = Note {
            key: MidiNote::C4 as u8,
            range: TimeRange(MusicalTime::START..MusicalTime::DURATION_HALF),
        };
        /// whole note
        const TEST_D4: Note = Note {
            key: MidiNote::D4 as u8,
            range: TimeRange(MusicalTime::START..MusicalTime::DURATION_WHOLE),
        };
        /// two whole notes
        const TEST_E4: Note = Note {
            key: MidiNote::E4 as u8,
            range: TimeRange(MusicalTime::START..MusicalTime::DURATION_BREVE),
        };
    }

    #[test]
    fn pattern_defaults() {
        let p = Pattern::default();
        assert_eq!(p.note_count(), 0, "Default pattern should have zero notes");

        let p = PatternBuilder::default().build().unwrap();
        assert_eq!(
            p.note_count(),
            0,
            "Default built pattern should have zero notes"
        );

        assert_eq!(
            p.time_signature(),
            TimeSignature::COMMON_TIME,
            "Default built pattern should have 4/4 time signature"
        );

        assert_eq!(
            p.duration(),
            MusicalTime::new_with_bars(&TimeSignature::COMMON_TIME, 1),
            "Default built pattern's duration should be one measure"
        );
    }

    #[test]
    fn pattern_one_half_note_is_one_bar() {
        let mut p = PatternBuilder::default().build().unwrap();
        p.add_note(Note::TEST_C4);
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with one half-note should be 1 bar"
        );
    }

    #[test]
    fn pattern_one_breve_is_one_bar() {
        let mut p = PatternBuilder::default().build().unwrap();
        p.add_note(Note::TEST_E4);
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with one note of length breve should be 1 bar"
        );
    }

    #[test]
    fn pattern_one_long_note_is_one_bar() {
        let p = PatternBuilder::default()
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::new_with_beats(4),
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with a single bar-long note is one bar"
        );
    }

    #[test]
    fn pattern_one_beat_with_1_4_time_signature_is_one_bar() {
        let p = PatternBuilder::default()
            .time_signature(TimeSignature::new_with(1, 4).unwrap())
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::new_with_beats(1),
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with a single whole note in 1/4 time is one bar"
        );
    }

    #[test]
    fn pattern_three_half_notes_is_one_bar() {
        let p = PatternBuilder::default()
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_HALF,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(1),
                MusicalTime::DURATION_HALF,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(2),
                MusicalTime::DURATION_HALF,
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with three half-notes on beat should be 1 bar"
        );
    }

    #[test]
    fn pattern_four_whole_notes_is_one_bar() {
        let p = PatternBuilder::default()
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(1),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(2),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(3),
                MusicalTime::DURATION_WHOLE,
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with four whole notes on beat should be 1 bar"
        );
    }

    #[test]
    fn pattern_five_notes_is_two_bars() {
        let p = PatternBuilder::default()
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(1),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(2),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(3),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(4),
                MusicalTime::DURATION_SIXTEENTH,
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            2,
            "Pattern with four whole notes and then a sixteenth should be 2 bars"
        );
    }

    #[test]
    fn default_pattern_builder() {
        let p = PatternBuilder::default().build().unwrap();
        assert_eq!(
            p.notes.len(),
            0,
            "Default PatternBuilder yields pattern with zero notes"
        );
        assert_eq!(
            p.duration,
            MusicalTime::new_with_bars(&p.time_signature, 1),
            "Default PatternBuilder yields one-measure pattern"
        );
    }

    #[test]
    fn pattern_api_is_ergonomic() {
        let mut p = PatternBuilder::default()
            .note(Note::TEST_C4.clone())
            .note(Note::TEST_D4.clone())
            .build()
            .unwrap();
        assert_eq!(p.notes.len(), 2, "PatternBuilder can add multiple notes");

        p.add_note(Note::TEST_C4.clone());
        assert_eq!(
            p.notes.len(),
            3,
            "Pattern can add duplicate notes. This is probably not desirable to allow."
        );

        assert!(p
            .move_note(&Note::TEST_C4, MusicalTime::new_with_beats(4))
            .is_ok());
        assert_eq!(p.notes.len(), 3, "Moving a note doesn't copy or destroy");
        p.remove_note(&Note::TEST_D4);
        assert_eq!(p.notes.len(), 2, "remove_note() removes notes");
        p.remove_note(&Note::TEST_C4);
        assert_eq!(
            p.notes.len(),
            2,
            "remove_note() must specify the note correctly."
        );
        p.remove_note(&Note::new_with_midi_note(
            MidiNote::C4,
            MusicalTime::new_with_beats(4),
            MusicalTime::DURATION_HALF,
        ));
        assert!(p.notes.is_empty(), "remove_note() removes duplicate notes.");
    }

    #[test]
    fn move_note_inside_pattern() {
        let mut p = PatternBuilder::default().build().unwrap();

        p.add_note(Note::TEST_C4.clone());
        assert!(p
            .move_note(
                &Note::TEST_C4,
                MusicalTime::START + MusicalTime::DURATION_SIXTEENTH,
            )
            .is_ok());
        assert_eq!(
            p.notes[0].range.0.start,
            MusicalTime::START + MusicalTime::DURATION_SIXTEENTH,
            "moving a note works"
        );
        assert_eq!(
            p.duration,
            MusicalTime::new_with_beats(4),
            "Moving a note in pattern doesn't change duration"
        );

        assert!(
            p.move_note(&Note::TEST_E4, MusicalTime::default()).is_err(),
            "moving nonexistent note should fail"
        );
    }

    #[test]
    fn move_note_outside_pattern() {
        let mut p = PatternBuilder::default().build().unwrap();

        p.add_note(Note::TEST_C4.clone());
        assert!(p
            .move_note(&Note::TEST_C4, MusicalTime::new_with_beats(4))
            .is_ok());
        assert_eq!(
            p.duration,
            MusicalTime::new_with_beats(4 * 2),
            "Moving a note out of pattern increases duration"
        );
    }

    #[test]
    fn move_and_resize_note() {
        let mut p = PatternBuilder::default().build().unwrap();

        p.add_note(Note::TEST_C4.clone());

        assert!(p
            .move_and_resize_note(
                &Note::TEST_C4,
                MusicalTime::START + MusicalTime::DURATION_EIGHTH,
                MusicalTime::DURATION_WHOLE,
            )
            .is_ok());
        let expected_range = TimeRange(
            (MusicalTime::START + MusicalTime::DURATION_EIGHTH)
                ..(MusicalTime::START + MusicalTime::DURATION_EIGHTH + MusicalTime::DURATION_WHOLE),
        );
        assert_eq!(
            p.notes[0].range, expected_range,
            "moving/resizing a note works"
        );
        assert_eq!(
            p.duration,
            MusicalTime::new_with_beats(4),
            "moving/resizing within pattern doesn't change duration"
        );

        assert!(p
            .move_and_resize_note(
                &Note::new_with_midi_note(
                    MidiNote::C4,
                    expected_range.0.start,
                    expected_range.0.end - expected_range.0.start,
                ),
                MusicalTime::new_with_beats(4),
                MusicalTime::DURATION_WHOLE,
            )
            .is_ok());
        assert_eq!(
            p.duration,
            MusicalTime::new_with_beats(8),
            "moving/resizing outside current pattern makes the pattern longer"
        );

        assert!(
            p.move_and_resize_note(
                &Note::TEST_E4,
                MusicalTime::default(),
                MusicalTime::default()
            )
            .is_err(),
            "moving/resizing nonexistent note should fail"
        );
    }

    #[test]
    fn change_note_key() {
        let mut p = PatternBuilder::default().build().unwrap();

        p.add_note(Note::TEST_C4.clone());
        assert_eq!(p.notes[0].key, MidiNote::C4 as u8);
        assert!(p
            .change_note_key(&Note::TEST_C4, MidiNote::C5 as u8)
            .is_ok());
        assert_eq!(p.notes[0].key, MidiNote::C5 as u8);

        assert!(
            p.change_note_key(&Note::TEST_C4, 254).is_err(),
            "changing key of nonexistent note should fail"
        );
    }

    #[test]
    fn pattern_dimensions_are_valid() {
        let p = Pattern::default();
        assert_eq!(
            p.time_signature,
            TimeSignature::COMMON_TIME,
            "default pattern should have sensible time signature"
        );

        for ts in [
            TimeSignature::COMMON_TIME,
            TimeSignature::CUT_TIME,
            TimeSignature::new_with(7, 64).unwrap(),
        ] {
            let p = PatternBuilder::default()
                .time_signature(ts)
                .build()
                .unwrap();
            assert_eq!(
                p.duration,
                MusicalTime::new_with_beats(ts.top),
                "Pattern's beat count matches its time signature"
            );

            // A typical 4/4 pattern has 16 subdivisions, which is a common
            // pattern resolution in other pattern-based sequencers and piano
            // rolls.
            assert_eq!(p.default_grid_value(), ts.bottom * ts.top,
                "Pattern's default grid value should be the time signature's beat count times its note value");
        }
    }

    #[test]
    fn pattern_note_insertion_is_easy() {
        let sixteen_notes = vec![
            60, 61, 62, 63, 64, 65, 66, 67, 60, 61, 62, 63, 64, 65, 66, 67,
        ];
        let len_16 = sixteen_notes.len();
        let p = PatternBuilder::default()
            .note_sequence(sixteen_notes, None)
            .build()
            .unwrap();
        assert_eq!(p.note_count(), len_16, "sixteen quarter notes");
        assert_eq!(p.notes[15].key, 67);
        assert_eq!(
            p.notes[15].range,
            TimeRange(
                MusicalTime::DURATION_QUARTER * 15
                    ..MusicalTime::DURATION_WHOLE * p.time_signature.top
            )
        );
        assert_eq!(
            p.duration,
            MusicalTime::DURATION_WHOLE * p.time_signature.top
        );

        let seventeen_notes = vec![
            60, 61, 62, 63, 64, 65, 66, 67, 60, 61, 62, 63, 64, 65, 66, 67, 68,
        ];
        let p = PatternBuilder::default()
            .note_sequence(seventeen_notes, None)
            .build()
            .unwrap();
        assert_eq!(
            p.duration,
            MusicalTime::DURATION_WHOLE * p.time_signature.top * 2,
            "17 notes in 4/4 pattern produces two bars"
        );

        let four_notes = vec![60, 61, 62, 63];
        let len_4 = four_notes.len();
        let p = PatternBuilder::default()
            .note_sequence(four_notes, Some(4))
            .build()
            .unwrap();
        assert_eq!(p.note_count(), len_4, "four quarter notes");
        assert_eq!(
            p.duration,
            MusicalTime::DURATION_WHOLE * p.time_signature.top
        );

        let three_notes_and_silence = vec![60, 0, 62, 63];
        let len_3_1 = three_notes_and_silence.len();
        let p = PatternBuilder::default()
            .note_sequence(three_notes_and_silence, Some(4))
            .build()
            .unwrap();
        assert_eq!(p.note_count(), len_3_1, "three quarter notes with one rest");
        assert_eq!(
            p.duration,
            MusicalTime::DURATION_WHOLE * p.time_signature.top
        );

        let eight_notes = vec![60, 61, 62, 63, 64, 65, 66, 67];
        let len_8 = eight_notes.len();
        let p = PatternBuilder::default()
            .time_signature(TimeSignature::CUT_TIME)
            .note_sequence(eight_notes, None)
            .build()
            .unwrap();
        assert_eq!(
            p.note_count(),
            len_8,
            "eight eighth notes in 2/2 time is two bars long"
        );
        assert_eq!(
            p.duration,
            MusicalTime::DURATION_WHOLE * p.time_signature.top * 2
        );

        let one_note = vec![60];
        let len_1 = one_note.len();
        let p = PatternBuilder::default()
            .note_sequence(one_note, None)
            .build()
            .unwrap();
        assert_eq!(
            p.note_count(),
            len_1,
            "one quarter note, and the rest is silence"
        );
        assert_eq!(p.notes[0].key, 60);
        assert_eq!(
            p.notes[0].range,
            TimeRange(MusicalTime::START..MusicalTime::DURATION_QUARTER)
        );
        assert_eq!(
            p.duration,
            MusicalTime::DURATION_WHOLE * p.time_signature.top
        );
    }

    #[test]
    fn cut_time_duration() {
        let p = PatternBuilder::default()
            .time_signature(TimeSignature::CUT_TIME)
            .build()
            .unwrap();
        assert_eq!(p.duration, MusicalTime::new_with_beats(2));
    }
}
