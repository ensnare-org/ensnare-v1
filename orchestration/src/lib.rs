// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use crate::orchestration::{Orchestrator, OrchestratorHelper, ProjectAction};
pub use egui::{orchestrator, project_widget, DescribesProject};

pub mod bus_route;
pub mod control_router;
pub mod egui;
pub mod humidifier;
pub mod main_mixer;
pub mod midi_router;
pub mod orchestration;
pub mod track;
pub mod traits;

pub mod prelude {
    pub use super::egui::project_widget;
    pub use super::traits::Orchestrates;
    pub use super::{Orchestrator, OrchestratorHelper};
}
