// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Ensnare services

pub use audio::{AudioService, AudioServiceEvent, AudioServiceInput, AudioSettings};
pub use egui::*;
pub use midi::{MidiService, MidiServiceEvent, MidiServiceInput, MidiSettings};
pub use project::{ProjectService, ProjectServiceEvent, ProjectServiceInput};

mod audio;
mod egui;
mod midi;
mod project;

pub mod prelude {
    pub use super::{ProjectService, ProjectServiceEvent, ProjectServiceInput};
}
