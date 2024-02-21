// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Management of all MIDI-related information that flows within the system.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        u4, u7, MidiChannel, MidiEvent, MidiInterfaceServiceEvent, MidiInterfaceServiceInput,
        MidiMessage, MidiNote, MidiPortDescriptor, MidiUtils,
    };
}

pub use {
    crate::services::{MidiInterfaceServiceEvent, MidiInterfaceServiceInput, MidiPortDescriptor},
    general_midi::GeneralMidiPercussionProgram,
    midly::{
        live::LiveEvent,
        num::{u4, u7},
        MidiMessage,
    },
    note::MidiNote,
    types::{MidiChannel, MidiEvent},
    util::MidiUtils,
};

mod general_midi;
mod note;
mod types;
mod util;
