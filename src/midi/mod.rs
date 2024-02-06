// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Management of all MIDI-related information that flows within the system.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        new_note_off, new_note_on, u4, u7, MidiChannel, MidiEvent, MidiInterfaceServiceEvent,
        MidiInterfaceServiceInput, MidiMessage, MidiNote, MidiPortDescriptor,
    };
}

pub use crate::services::{
    MidiInterfaceServiceEvent, MidiInterfaceServiceInput, MidiPortDescriptor,
};
pub use unfiled::{
    new_note_off, new_note_on, u4, u7, GeneralMidiPercussionProgram, GeneralMidiProgram, LiveEvent,
    MidiChannel, MidiEvent, MidiMessage, MidiNote,
};

mod unfiled;
