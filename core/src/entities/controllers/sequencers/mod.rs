// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use midi::MidiSequencer;
pub use note::{NoteSequencer, NoteSequencerBuilder};
pub use pattern::{LivePatternEvent, LivePatternSequencer};
pub use pattern::{PatternSequencer, PatternSequencerBuilder};

mod midi;
mod note;
mod pattern;
mod smf;
mod util;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::tests::{
        validate_sequences_live_patterns_trait, validate_sequences_midi_trait,
        validate_sequences_notes_trait, validate_sequences_patterns_trait,
    };

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
        let mut s = LivePatternSequencer::new_with(std::sync::Arc::clone(&piano_roll));

        validate_sequences_live_patterns_trait(piano_roll, &mut s);
    }
}
