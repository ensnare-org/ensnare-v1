// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Ensnare services

pub use audio::{AudioService, AudioServiceEvent, AudioServiceInput, AudioSettings};
pub use control_bar::ControlBar;
pub use egui::*;
pub use midi::{MidiService, MidiServiceEvent, MidiSettings};
pub use project::{ProjectService, ProjectServiceEvent, ProjectServiceInput};

mod audio;
mod control_bar;
mod egui;
mod midi;
mod orchestrator;
mod project;

pub mod prelude {
    pub use super::{ProjectService, ProjectServiceEvent, ProjectServiceInput};
}
