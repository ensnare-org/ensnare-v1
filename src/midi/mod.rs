// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Management of all MIDI-related information that flows within the system.

// pub mod midi {
//
//     pub mod interface {
//         //! External MIDI hardware, such as MIDI interfaces or MIDI keyboards
//         //! plugged in through USB).
//         pub use ensnare_services::{
//             MidiInterfaceService, MidiInterfaceServiceEvent, MidiInterfaceServiceInput,
//             MidiPortDescriptor,
//         };
//     }

//     /// The most commonly used imports.
//     pub mod prelude {
//         pub use super::{
//             interface::{MidiInterfaceServiceEvent, MidiInterfaceServiceInput, MidiPortDescriptor},
//             u4, u7, MidiChannel, MidiMessage, MidiNote,
//         };
//     }
// }

pub mod prelude {
    pub use super::{
        // crate::{MidiInterfaceServiceEvent, MidiInterfaceServiceInput, MidiPortDescriptor},
        u4,
        u7,
        MidiChannel,
        MidiMessage,
        MidiNote,
    };
    pub use super::{MidiInterfaceServiceEvent, MidiInterfaceServiceInput, MidiPortDescriptor};
}

pub use crate::core::midi::{u4, u7, MidiChannel, MidiEvent, MidiMessage, MidiNote};
pub use crate::services::{
    MidiInterfaceServiceEvent, MidiInterfaceServiceInput, MidiPortDescriptor,
};
