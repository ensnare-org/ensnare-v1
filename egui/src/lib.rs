// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui logic for drawing ensnare entities.

mod controllers;
mod orchestration;
mod track;
mod transport;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        controllers::trip,
        orchestration::{old_orchestrator, orchestrator},
        track::{signal_chain, track_widget, SignalChainWidgetAction},
        transport::transport,
    };
}
