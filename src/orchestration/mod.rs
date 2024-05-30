// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Support for project rendering.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{Orchestrator, Project, ProjectTitle, Projects, TrackUid};
}

pub use bus::{BusRoute, BusStation};
pub use midi_router::MidiRouter;
pub use orchestrator::Orchestrator;
pub use project::{Project, ProjectTitle, ProjectViewState, SignalChainItem, TrackViewMode};
pub(crate) use repositories::EntityRepository;
pub use track::{TrackTitle, TrackUid, TrackUidFactory};
pub use traits::Projects;

mod bus;
mod humidity;
mod midi_router;
mod orchestrator;
mod project;
mod repositories;
mod track;
mod traits;
