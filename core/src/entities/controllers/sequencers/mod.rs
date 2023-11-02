// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use midi::MidiSequencer;
pub use note::{NoteSequencer, NoteSequencerBuilder};
pub use pattern::{LivePatternSequencer, PatternSequencer, PatternSequencerBuilder};

mod midi;
mod note;
mod pattern;
mod smf;
mod util;

#[cfg(test)]
pub mod tests {
    use std::sync::{Arc, RwLock};

    use super::*;
    use crate::{
        midi::{MidiChannel, MidiMessage, MidiNote},
        piano_roll::{Note, Pattern, PatternBuilder, PatternUid, PianoRoll},
        prelude::{MusicalTime, Uid},
        traits::{tests::validate_sequences_midi_trait, Sequences},
    };

    fn replay_units<MU>(
        sequences: &mut dyn Sequences<MU = MU>,
        start_time: MusicalTime,
        duration: MusicalTime,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        let mut v = Vec::default();
        sequences.update_time(&(start_time..start_time + duration));
        sequences.work(&mut |_, event| match event {
            crate::traits::EntityEvent::Midi(channel, message) => v.push((channel, message)),
            crate::traits::EntityEvent::Control(_) => panic!(),
        });
        v
    }

    fn replay_all_units<MU>(
        sequences: &mut dyn Sequences<MU = MU>,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        replay_units(sequences, MusicalTime::TIME_ZERO, MusicalTime::TIME_MAX)
    }

    /// Validates the provided implementation of [Sequences] for a [Note].
    pub(crate) fn validate_sequences_notes_trait(s: &mut dyn Sequences<MU = Note>) {
        const SAMPLE_NOTE: Note =
            Note::new_with(60, MusicalTime::START, MusicalTime::DURATION_QUARTER);
        const SAMPLE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);

        s.clear();

        assert!(replay_all_units(s).is_empty());
        assert!(s
            .record(SAMPLE_MIDI_CHANNEL, &SAMPLE_NOTE, MusicalTime::START)
            .is_ok());
        let message_count = replay_all_units(s).len();
        assert_eq!(
            message_count, 2,
            "After recording a Note, two new messages should be recorded."
        );

        assert!(s
            .remove(
                SAMPLE_MIDI_CHANNEL,
                &SAMPLE_NOTE,
                MusicalTime::START + MusicalTime::new_with_units(1)
            )
            .is_ok());
        assert_eq!(
            replay_all_units(s).len(),
            message_count,
            "Number of messages should remain unchanged after removing nonexistent Note"
        );

        assert!(s
            .remove(SAMPLE_MIDI_CHANNEL, &SAMPLE_NOTE, MusicalTime::START)
            .is_ok());
        assert!(
            replay_all_units(s).is_empty(),
            "Sequencer should be empty after removing last note"
        );
    }

    /// Validates the provided implementation of [Sequences] for a [Pattern].
    pub(crate) fn validate_sequences_patterns_trait(s: &mut dyn Sequences<MU = Pattern>) {
        const SAMPLE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);

        s.clear();

        {
            let pattern = PatternBuilder::default().build().unwrap();

            assert!(replay_all_units(s).is_empty());
            assert!(s
                .record(SAMPLE_MIDI_CHANNEL, &pattern, MusicalTime::START)
                .is_ok());
            let message_count = replay_all_units(s).len();
            assert_eq!(
                message_count, 0,
                "After recording an empty pattern, no new messages should be recorded."
            );
        }
        {
            let pattern = PatternBuilder::default()
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

            assert!(s
                .record(SAMPLE_MIDI_CHANNEL, &pattern, MusicalTime::START)
                .is_ok());
            let message_count = replay_all_units(s).len();
            assert_eq!(
                message_count, 8,
                "After recording an pattern with four notes, eight new messages should be recorded."
            );

            assert!(s
                .remove(
                    SAMPLE_MIDI_CHANNEL,
                    &pattern,
                    MusicalTime::START + MusicalTime::new_with_units(1)
                )
                .is_ok());
            assert_eq!(
                replay_all_units(s).len(),
                message_count,
                "Number of messages should remain unchanged after removing nonexistent item"
            );

            assert!(s
                .remove(SAMPLE_MIDI_CHANNEL, &pattern, MusicalTime::START)
                .is_ok());
            assert!(
                replay_all_units(s).is_empty(),
                "Sequencer should be empty after removing pattern"
            );
        }
    }

    /// Validates the provided implementation of [Sequences] for a [Pattern].
    pub(crate) fn validate_sequences_live_patterns_trait(
        piano_roll: Arc<RwLock<PianoRoll>>,
        s: &mut dyn Sequences<MU = PatternUid>,
    ) {
        const SAMPLE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);

        let empty_pattern_uid = piano_roll
            .write()
            .unwrap()
            .insert(PatternBuilder::default().build().unwrap());
        let ordinary_pattern_uid = piano_roll.write().unwrap().insert(
            PatternBuilder::default()
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
                .unwrap(),
        );

        s.clear();

        {
            assert!(replay_all_units(s).is_empty());
            assert!(s
                .record(SAMPLE_MIDI_CHANNEL, &empty_pattern_uid, MusicalTime::START)
                .is_ok());
            let message_count = replay_all_units(s).len();
            assert_eq!(
                message_count, 0,
                "After recording an empty pattern, no new messages should be recorded."
            );
        }
        {
            assert!(s
                .record(
                    SAMPLE_MIDI_CHANNEL,
                    &ordinary_pattern_uid,
                    MusicalTime::START
                )
                .is_ok());
            let message_count = replay_all_units(s).len();
            assert_eq!(
                message_count, 8,
                "After recording an pattern with four notes, eight new messages should be recorded."
            );

            assert!(s
                .remove(
                    SAMPLE_MIDI_CHANNEL,
                    &ordinary_pattern_uid,
                    MusicalTime::START + MusicalTime::new_with_units(1)
                )
                .is_ok());
            assert_eq!(
                replay_all_units(s).len(),
                message_count,
                "Number of messages should remain unchanged after removing nonexistent item"
            );

            assert!(s
                .remove(
                    SAMPLE_MIDI_CHANNEL,
                    &ordinary_pattern_uid,
                    MusicalTime::START
                )
                .is_ok());
            assert!(
                replay_all_units(s).is_empty(),
                "Sequencer should be empty after removing pattern"
            );
        }
    }

    #[test]
    fn midi_sequencer_passes_trait_validation() {
        let mut s = MidiSequencer::default();

        validate_sequences_midi_trait(&mut s);
    }

    #[test]
    fn note_sequencer_passes_trait_validation() {
        let mut s = NoteSequencer::default();

        validate_sequences_notes_trait(&mut s);
    }

    #[test]
    fn pattern_sequencer_passes_trait_validation() {
        let mut s = PatternSequencer::default();

        validate_sequences_patterns_trait(&mut s);
    }

    #[test]
    fn live_pattern_sequencer_passes_trait_validation() {
        let piano_roll = std::sync::Arc::new(std::sync::RwLock::new(
            crate::piano_roll::PianoRoll::default(),
        ));
        let mut s = LivePatternSequencer::new_with(Uid(2048), std::sync::Arc::clone(&piano_roll));

        validate_sequences_live_patterns_trait(piano_roll, &mut s);
    }
}
