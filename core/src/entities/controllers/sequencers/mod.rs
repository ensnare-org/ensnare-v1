// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use midi::MidiSequencer;
pub use note::{NoteSequencer, NoteSequencerBuilder};
pub use pattern::{LivePatternSequencer, PatternSequencer, PatternSequencerBuilder};

mod midi;
mod note;
mod pattern;
mod smf;

use crate::{
    midi::{u7, MidiChannel, MidiMessage},
    time::ViewRange,
    traits::{Configurable, ControlEventsFn, Controls, EntityEvent, HandlesMidi, MidiMessagesFn},
};
use bit_vec::BitVec;
use std::fmt::Debug;

/// [MidiNoteMinder] watches a MIDI message stream and remembers which notes are
/// currently active (we've gotten a note-on without a note-off). Then, when
/// asked, it produces a list of MIDI message that turn off all active notes.
///
/// [MidiNoteMinder] doesn't know about [MidiChannel]s. It's up to the caller to
/// track channels, or else assume that if we got any message, it's for us, and
/// that the same is true for recipients of whatever we send.
#[derive(Debug)]
pub struct MidiNoteMinder {
    active_notes: BitVec,
}
impl Default for MidiNoteMinder {
    fn default() -> Self {
        Self {
            active_notes: BitVec::from_elem(128, false),
        }
    }
}
impl HandlesMidi for MidiNoteMinder {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        #[allow(unused_variables)]
        match message {
            MidiMessage::NoteOff { key, vel } => {
                self.active_notes.set(key.as_int() as usize, false);
            }
            MidiMessage::NoteOn { key, vel } => {
                self.active_notes
                    .set(key.as_int() as usize, vel != u7::from(0));
            }
            _ => {}
        }
    }
}
impl Controls for MidiNoteMinder {
    fn update_time(&mut self, _: &ViewRange) {}

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        for (i, active_note) in self.active_notes.iter().enumerate() {
            if active_note {
                control_events_fn(
                    None,
                    EntityEvent::Midi(
                        MidiChannel::default(),
                        MidiMessage::NoteOff {
                            key: u7::from_int_lossy(i as u8),
                            vel: u7::from(0),
                        },
                    ),
                );
            }
        }
        self.active_notes.clear();
    }
}
impl Configurable for MidiNoteMinder {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        midi::{new_note_off, new_note_on, MidiChannel, MidiMessage, MidiNote},
        piano_roll::{Note, Pattern, PatternBuilder, PatternUid, PianoRoll},
        prelude::{MusicalTime, Uid},
        traits::{tests::validate_sequences_midi_trait, Sequences},
    };
    use std::sync::{Arc, RwLock};

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

    fn gather_all_messages(mnm: &mut MidiNoteMinder) -> Vec<MidiMessage> {
        let mut v = Vec::default();
        mnm.work(&mut |_, e| match e {
            EntityEvent::Midi(_, message) => v.push(message),
            EntityEvent::Control(_) => panic!("didn't expect a Control event here"),
        });
        v
    }
    #[test]
    fn midi_note_minder() {
        let mut mnm = MidiNoteMinder::default();

        assert!(gather_all_messages(&mut mnm).is_empty());

        // Unexpected note-off doesn't explode
        mnm.handle_midi_message(
            MidiChannel::default(),
            new_note_off(42, 111),
            &mut |_, _| {},
        );
        assert!(gather_all_messages(&mut mnm).is_empty());

        // normal
        mnm.handle_midi_message(MidiChannel::default(), new_note_on(42, 99), &mut |_, _| {});
        let msgs = gather_all_messages(&mut mnm);
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            msgs[0],
            MidiMessage::NoteOff {
                key: u7::from(42),
                vel: u7::from(0)
            }
        );

        // duplicate on doesn't explode or add twice
        mnm.handle_midi_message(MidiChannel::default(), new_note_on(42, 88), &mut |_, _| {});
        let msgs = gather_all_messages(&mut mnm);
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            msgs[0],
            MidiMessage::NoteOff {
                key: u7::from(42),
                vel: u7::from(0)
            }
        );

        // normal
        mnm.handle_midi_message(MidiChannel::default(), new_note_off(42, 77), &mut |_, _| {});
        assert!(gather_all_messages(&mut mnm).is_empty());

        // duplicate off doesn't explode
        mnm.handle_midi_message(MidiChannel::default(), new_note_off(42, 66), &mut |_, _| {});
        assert!(gather_all_messages(&mut mnm).is_empty());

        // velocity zero treated same as note-off
        mnm.handle_midi_message(MidiChannel::default(), new_note_on(42, 99), &mut |_, _| {});
        assert_eq!(gather_all_messages(&mut mnm).len(), 1);
        mnm.handle_midi_message(MidiChannel::default(), new_note_off(42, 99), &mut |_, _| {});
        assert!(gather_all_messages(&mut mnm).is_empty());
        mnm.handle_midi_message(MidiChannel::default(), new_note_on(42, 99), &mut |_, _| {});
        assert_eq!(gather_all_messages(&mut mnm).len(), 1);
        mnm.handle_midi_message(MidiChannel::default(), new_note_on(42, 0), &mut |_, _| {});
        assert!(gather_all_messages(&mut mnm).is_empty());
    }
}
