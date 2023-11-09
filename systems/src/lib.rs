// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Subsystems.

pub use audio_panel::{audio_settings, AudioPanel, AudioPanelEvent, AudioSettings, NeedsAudioFn};
pub use control_panel::{ControlPanel, ControlPanelAction};
pub use midi_panel::{midi_settings, MidiPanel, MidiPanelEvent, MidiSettings};
pub use orchestrator_panel::{OrchestratorEvent, OrchestratorInput, OrchestratorPanel};
pub use palette_panel::PalettePanel;

mod audio_panel;
mod control_panel;
mod midi_panel;
mod orchestrator_panel;
mod palette_panel;

pub mod prelude {
    pub use crate::orchestrator_panel::{OrchestratorEvent, OrchestratorInput, OrchestratorPanel};
}
