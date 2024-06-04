// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Management of all MIDI-related information that flows within the system.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        u4, u7, MidiChannel, MidiEvent, MidiMessage, MidiNote, MidiPortDescriptor, MidiUtils,
    };
}

pub use {
    ensnare::types::{
        u4, u7, GeneralMidiPercussionCode, MidiChannel, MidiNote, MidiPortDescriptor,
    },
    midly::{live::LiveEvent, MidiMessage},
    types::MidiEvent,
    util::MidiUtils,
};

mod types;
mod util;
