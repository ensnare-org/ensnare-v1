// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Ensnare services

pub use audio::{AudioPanelEvent, AudioService, AudioSettings, NeedsAudioFn};
pub use control_bar::ControlBar;
pub use egui::*;
pub use midi::{MidiPanelEvent, MidiService, MidiSettings};
pub use orchestrator::{OrchestratorEvent, OrchestratorInput, OrchestratorService};

mod audio;
mod control_bar;
mod egui;
mod midi;
mod orchestrator;

pub mod prelude {
    pub use crate::orchestrator::{OrchestratorEvent, OrchestratorInput, OrchestratorService};
}
