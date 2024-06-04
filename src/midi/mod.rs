// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Management of all MIDI-related information that flows within the system.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        u4, u7, MidiChannel, MidiEvent, MidiMessage, MidiNote, MidiPortDescriptor, MidiUtils,
    };
}

pub use {
    ensnare::types::MidiPortDescriptor,
    general_midi::GeneralMidiPercussionCode,
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
