// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Long-running services that are useful to a music application.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        AudioService, AudioServiceEvent, AudioServiceInput, AudioSettings, MidiService,
        MidiServiceEvent, MidiServiceInput, MidiSettings, ProjectService, ProjectServiceEvent,
        ProjectServiceInput,
    };
}
pub use audio::{AudioService, AudioServiceEvent, AudioServiceInput, AudioSettings};
pub use midi::{MidiService, MidiServiceEvent, MidiServiceInput, MidiSettings};
pub use midi_interface::{
    MidiInterfaceService, MidiInterfaceServiceEvent, MidiInterfaceServiceInput, MidiPortDescriptor,
};
pub use project::{ProjectService, ProjectServiceEvent, ProjectServiceInput};

mod audio;
mod midi;
mod midi_interface;
mod project;
