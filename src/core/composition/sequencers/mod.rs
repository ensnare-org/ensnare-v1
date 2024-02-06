// Copyright (c) 2024 Mike Tsao. All rights reserved.

pub mod midi;
pub mod note;
pub mod pattern;

pub use midi::{MidiSequencer, MidiSequencerBuilder};
pub use note::{NoteSequencer, NoteSequencerBuilder};
pub use pattern::{PatternSequencer, PatternSequencerBuilder};

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::prelude::*;

    fn replay_units<MU>(
        sequences: &mut dyn Sequences<MU = MU>,
        start_time: MusicalTime,
        duration: MusicalTime,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        let mut v = Vec::default();
        sequences.update_time_range(&TimeRange(start_time..start_time + duration));
        sequences.work(&mut |event| match event {
            WorkEvent::Midi(channel, message) => v.push((channel, message)),
            WorkEvent::MidiForTrack(_, channel, message) => v.push((channel, message)),
            WorkEvent::Control(_) => panic!(),
        });
        v
    }

    fn replay_all_units<MU>(
        sequences: &mut dyn Sequences<MU = MU>,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        replay_units(sequences, MusicalTime::TIME_ZERO, MusicalTime::TIME_MAX)
    }

    /// Validates the provided implementation of [Sequences] for a [Note].
    pub fn validate_sequences_notes_trait(s: &mut dyn Sequences<MU = Note>) {
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
                MusicalTime::START + MusicalTime::ONE_UNIT
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
                    MusicalTime::ONE_BEAT,
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
                    MusicalTime::START + MusicalTime::ONE_UNIT
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

    #[test]
    fn midi_sequencer_passes_trait_validation() {
        let mut s = MidiSequencer::default();

        validate_sequences_midi_trait(&mut s);
    }

    #[test]
    fn pattern_sequencer_passes_trait_validation() {
        let mut s = PatternSequencer::default();

        validate_sequences_patterns_trait(&mut s);
    }

    fn replay_messages(
        sequences_midi: &mut dyn SequencesMidi,
        start_time: MusicalTime,
        duration: MusicalTime,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        let mut v = Vec::default();
        sequences_midi.update_time_range(&TimeRange(start_time..start_time + duration));
        sequences_midi.work(&mut |event| match event {
            WorkEvent::Midi(channel, message) => v.push((channel, message)),
            WorkEvent::MidiForTrack(_, channel, message) => v.push((channel, message)),
            WorkEvent::Control(_) => panic!(),
        });
        v
    }

    fn replay_all_messages(
        sequences_midi: &mut dyn SequencesMidi,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        replay_messages(
            sequences_midi,
            MusicalTime::TIME_ZERO,
            MusicalTime::TIME_MAX,
        )
    }

    /// Validates the provided implementation of [SequencesMidi].
    pub fn validate_sequences_midi_trait(sequences: &mut dyn SequencesMidi) {
        const SAMPLE_NOTE_ON_MESSAGE: MidiMessage = MidiMessage::NoteOn {
            key: u7::from_int_lossy(60),
            vel: u7::from_int_lossy(100),
        };
        const SAMPLE_NOTE_OFF_MESSAGE: MidiMessage = MidiMessage::NoteOff {
            key: u7::from_int_lossy(60),
            vel: u7::from_int_lossy(100),
        };
        const SAMPLE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);

        assert!(replay_all_messages(sequences).is_empty());
        assert!(sequences
            .record_midi_message(
                SAMPLE_MIDI_CHANNEL,
                SAMPLE_NOTE_OFF_MESSAGE,
                MusicalTime::START
            )
            .is_ok());
        assert_eq!(
            replay_all_messages(sequences).len(),
            1,
            "sequencer should contain one recorded message"
        );
        sequences.clear();
        assert!(replay_all_messages(sequences).is_empty());

        assert!(
            sequences.is_finished(),
            "An empty sequencer should always be finished."
        );
        assert!(
            !sequences.is_performing(),
            "A sequencer should not be performing before play()"
        );

        let mut do_nothing = |_, _| {};

        assert!(!sequences.is_recording());
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_ON_MESSAGE,
            &mut do_nothing,
        );
        assert!(
            replay_all_messages(sequences).is_empty(),
            "sequencer should ignore incoming messages when not recording"
        );

        sequences.start_recording();
        assert!(sequences.is_recording());
        sequences.update_time_range(&TimeRange(
            MusicalTime::ONE_BEAT..MusicalTime::DURATION_QUARTER,
        ));
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_ON_MESSAGE,
            &mut do_nothing,
        );
        sequences.update_time_range(&TimeRange(
            MusicalTime::new_with_beats(2)..MusicalTime::DURATION_QUARTER,
        ));
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_OFF_MESSAGE,
            &mut do_nothing,
        );
        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should reflect recorded messages even while recording"
        );
        sequences.stop();
        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should reflect recorded messages after recording"
        );

        assert!(
            replay_messages(
                sequences,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_QUARTER,
            )
            .is_empty(),
            "sequencer should replay no events for time slice before recorded events"
        );

        assert_eq!(
            replay_messages(
                sequences,
                MusicalTime::ONE_BEAT,
                MusicalTime::DURATION_QUARTER,
            )
            .len(),
            1,
            "sequencer should produce appropriate messages for time slice"
        );

        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should produce appropriate messages for time slice"
        );
    }
}
