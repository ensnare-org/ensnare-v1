// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use crate::orchestration::{Orchestrator, OrchestratorHelper};
pub use control_router::ControlRouter;
pub use egui::{orchestrator, project_widget};

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
    pub use super::{
        egui::{project_widget, ProjectAction},
        traits::Orchestrates,
        Orchestrator, OrchestratorHelper,
    };
}
