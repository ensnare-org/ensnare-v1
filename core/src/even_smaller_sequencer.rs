// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    midi::prelude::*,
    piano_roll::{Note, Pattern},
    prelude::*,
    rng::Rng,
    traits::{prelude::*, Sequences, SequencesMidi},
    widgets::controllers::es_sequencer,
};
use btreemultimap::BTreeMultiMap;
use derive_builder::Builder;
use ensnare_proc_macros::{Control, IsControllerWithTimelineDisplay, Uid};
use serde::{Deserialize, Serialize};

impl ESSequencerBuilder {
    /// Builds the [ESSequencer].
    pub fn build(&self) -> anyhow::Result<ESSequencer, ESSequencerBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    /// Produces a random sequence of quarter-note notes. For debugging.
    pub fn random(&mut self, range: std::ops::Range<MusicalTime>) -> &mut Self {
        let mut rng = Rng::default();

        for _ in 0..32 {
            let beat_range = range.start.total_beats() as u64..range.end.total_beats() as u64;
            let note_start = MusicalTime::new_with_beats(rng.0.rand_range(beat_range) as usize);
            self.note(Note {
                key: rng.0.rand_range(16..100) as u8,
                range: note_start..note_start + MusicalTime::DURATION_QUARTER,
            });
        }
        self
    }
}

/// Parts of [ESSequencer] that shouldn't be persisted.
#[derive(Debug, Default)]
pub struct ESSequencerEphemerals {
    /// The sequencer should be performing work for this time slice.
    range: std::ops::Range<MusicalTime>,
    /// The actual events that the sequencer emits.
    events: BTreeMultiMap<MusicalTime, MidiMessage>,
    /// The latest end time (exclusive) of all the events.
    final_event_time: MusicalTime,
    /// The next place to insert a note.
    cursor: MusicalTime,
    /// Whether we're performing, in the [Performs] sense.
    is_performing: bool,
    /// Whether we're recording.
    is_recording: bool,

    view_range: std::ops::Range<MusicalTime>,
}

/// [ESSequencer] replays [MidiMessage]s according to [MusicalTime].
#[derive(
    Debug, Default, Control, IsControllerWithTimelineDisplay, Uid, Serialize, Deserialize, Builder,
)]
#[builder(build_fn(private, name = "build_from_builder"))]
#[deprecated]
pub struct ESSequencer {
    #[allow(missing_docs)]
    #[builder(default)]
    uid: Uid,
    #[allow(missing_docs)]
    #[builder(default)]
    midi_channel_out: MidiChannel,

    /// The [Note]s to be sequenced.
    #[builder(default, setter(each(name = "note", into)))]
    notes: Vec<Note>,

    /// The [Pattern]s to be sequenced.
    #[builder(default, setter(each(name = "pattern", into)))]
    patterns: Vec<(MusicalTime, Pattern)>,

    /// The default time signature.
    #[builder(default)]
    time_signature: TimeSignature,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: ESSequencerEphemerals,
}
impl SequencesMidi for ESSequencer {
    fn clear(&mut self) {
        self.notes.clear();
        self.patterns.clear();
        self.calculate_events();
    }

    fn record_midi_event(
        &mut self,
        channel: MidiChannel,
        event: crate::midi::MidiEvent,
    ) -> anyhow::Result<()> {
        self.e.events.insert(event.time, event.message);
        Ok(())
    }

    fn remove_midi_event(
        &mut self,
        channel: MidiChannel,
        event: crate::midi::MidiEvent,
    ) -> anyhow::Result<()> {
        self.e
            .events
            .retain(|&e_time, &e_message| event.time == e_time && event.message == e_message);
        Ok(())
    }

    fn start_recording(&mut self) {
        self.e.is_recording = true;
    }

    fn is_recording(&self) -> bool {
        self.e.is_recording
    }
}
impl Sequences for ESSequencer {
    type MU = Note;

    fn record(
        &mut self,
        channel: MidiChannel,
        unit: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let note = Note {
            key: unit.key,
            range: (unit.range.start + position)..(unit.range.end + position),
        };
        self.notes.push(note);
        self.calculate_events();
        Ok(())
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        unit: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let note = Note {
            key: unit.key,
            range: (unit.range.start + position)..(unit.range.end + position),
        };
        self.notes.retain(|n| *n != note);
        self.calculate_events();
        Ok(())
    }

    fn clear(&mut self) {
        self.notes.clear();
        self.calculate_events();
    }
}
impl ESSequencer {
    #[allow(dead_code)]
    fn cursor(&self) -> MusicalTime {
        self.e.cursor
    }

    /// Adds the [Pattern] at the specified location. Returns the duration of
    /// the inserted pattern.
    pub fn insert_pattern(
        &mut self,
        pattern: &Pattern,
        position: MusicalTime,
    ) -> anyhow::Result<MusicalTime> {
        self.patterns.push((position, pattern.clone()));
        self.calculate_events();
        Ok(pattern.duration())
    }

    /// Adds the [Pattern] at the sequencer cursor, and advances the cursor.
    pub fn append_pattern(&mut self, pattern: &Pattern) -> anyhow::Result<()> {
        let position = self.e.cursor;
        let duration = self.insert_pattern(pattern, position)?;
        self.e.cursor += duration;
        Ok(())
    }

    /// Adds the [Note] at the specified location.
    pub fn insert_note(&mut self, note: &Note, position: MusicalTime) -> anyhow::Result<()> {
        self.notes.push(Note {
            key: note.key,
            range: (note.range.start + position)..(note.range.end + position),
        });
        self.calculate_events();
        Ok(())
    }

    /// Adds the [Note] at the sequencer cursor, and advances the cursor.
    pub fn append_note(&mut self, note: &Note) -> anyhow::Result<()> {
        let position = self.e.cursor;
        self.insert_note(note, position)?;
        self.e.cursor += MusicalTime::new_with_beats(1);
        Ok(())
    }

    fn insert_note_as_event(&mut self, note: &Note) {
        self.e.events.insert(
            note.range.start,
            MidiMessage::NoteOn {
                key: note.key.into(),
                vel: 127.into(),
            },
        );
        self.e.events.insert(
            note.range.end,
            MidiMessage::NoteOff {
                key: note.key.into(),
                vel: 0.into(),
            },
        );
        if note.range.end > self.e.final_event_time {
            self.e.final_event_time = note.range.end;
        }
    }

    pub fn view_range(&self) -> &std::ops::Range<MusicalTime> {
        &self.e.view_range
    }

    // TODO: can we reduce visibility?
    pub(crate) fn calculate_events(&mut self) {
        self.e.events.clear();
        self.e.final_event_time = MusicalTime::START;

        self.notes.clone().iter().for_each(|note| {
            self.insert_note_as_event(note);
        });
        self.patterns
            .clone()
            .iter()
            .for_each(|(position, pattern)| {
                pattern.notes().iter().for_each(|note| {
                    self.insert_note_as_event(&Note {
                        key: note.key,
                        range: (note.range.start + *position)..(note.range.end + *position),
                    });
                });
            });
    }

    // This method is private because callers need to remember to call
    // calculate_events() when they're done.
    pub(crate) fn toggle_note(&mut self, note: Note) {
        if self.notes.contains(&note) {
            self.notes.retain(|n| n != &note);
        } else {
            self.notes.push(note);
        }
    }

    #[allow(missing_docs)]
    pub fn notes(&self) -> &[Note] {
        self.notes.as_ref()
    }

    #[allow(missing_docs)]
    pub fn patterns(&self) -> &[(MusicalTime, Pattern)] {
        self.patterns.as_ref()
    }
}
impl Displays for ESSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(es_sequencer(self))
    }
}
impl DisplaysInTimeline for ESSequencer {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.e.view_range = view_range.clone();
    }
}
impl HandlesMidi for ESSequencer {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        if self.is_recording() {
            self.e.events.insert(self.e.range.start, message);
        }
    }
}
impl Controls for ESSequencer {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {
        self.e.range = range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        let events = self.e.events.range(self.e.range.start..self.e.range.end);
        for event in events {
            control_events_fn(self.uid, EntityEvent::Midi(MidiChannel(0), *event.1));
        }
    }

    fn is_finished(&self) -> bool {
        // both these are exclusive range bounds
        self.e.range.end >= self.e.final_event_time
    }

    fn play(&mut self) {
        self.e.is_performing = true;
        self.e.is_recording = false;
    }

    fn stop(&mut self) {
        self.e.is_performing = false;
        self.e.is_recording = false;
    }

    fn skip_to_start(&mut self) {
        self.update_time(&(MusicalTime::START..MusicalTime::START))
    }

    fn is_performing(&self) -> bool {
        self.e.is_performing
    }
}
impl Configurable for ESSequencer {}
impl Serializable for ESSequencer {
    fn after_deser(&mut self) {
        self.calculate_events();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        piano_roll::PatternBuilder,
        traits::tests::{validate_sequences_midi_trait, validate_sequences_notes_trait},
    };

    use super::*;

    #[test]
    fn basic() {
        let s = ESSequencer::default();
        assert!(s.notes.is_empty(), "default sequencer has no notes");
        assert!(s.e.events.is_empty(), "default sequencer has no events");
    }

    #[test]
    fn adding_notes_translates_to_events() {
        let mut s = ESSequencerBuilder::default()
            .note(Note {
                key: 69,
                range: MusicalTime::DURATION_WHOLE
                    ..MusicalTime::DURATION_WHOLE + MusicalTime::DURATION_QUARTER,
            })
            .build()
            .unwrap();
        s.after_deser();
        assert_eq!(
            s.e.events.len(),
            2,
            "Adding one note should create two events"
        );

        let _ = s.append_note(&Note {
            key: 70,
            range: MusicalTime::DURATION_WHOLE
                ..MusicalTime::DURATION_WHOLE + MusicalTime::DURATION_QUARTER,
        });
        assert_eq!(
            s.e.events.len(),
            4,
            "Adding a second note should create two more events"
        );
    }

    #[test]
    fn adding_patterns_translates_to_events() {
        let mut s = ESSequencerBuilder::default()
            .pattern((
                MusicalTime::DURATION_QUARTER,
                PatternBuilder::default()
                    .note(Note {
                        key: 1,
                        range: MusicalTime::START
                            ..MusicalTime::START + MusicalTime::DURATION_QUARTER,
                    })
                    .build()
                    .unwrap(),
            ))
            .build()
            .unwrap();
        s.after_deser();
        assert_eq!(
            s.e.events.len(),
            2,
            "Adding a pattern with one note should create two events"
        );

        let _ = s.append_pattern(
            &PatternBuilder::default()
                .note(Note {
                    key: 1,
                    range: MusicalTime::START..MusicalTime::START + MusicalTime::DURATION_QUARTER,
                })
                .build()
                .unwrap(),
        );
        assert_eq!(
            s.e.events.len(),
            4,
            "Appending another pattern with one note should create two more events"
        );
    }

    #[test]
    fn sequencer_passes_sequences_trait_validation() {
        let mut s = ESSequencerBuilder::default().build().unwrap();

        validate_sequences_midi_trait(&mut s);
        validate_sequences_notes_trait(&mut s);
    }
}
