// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Subsystems.

pub use orchestrator_panel::{OrchestratorEvent, OrchestratorInput, OrchestratorPanel};

mod orchestrator_panel;

pub mod prelude {
    pub use crate::orchestrator_panel::{OrchestratorEvent, OrchestratorInput, OrchestratorPanel};
}
