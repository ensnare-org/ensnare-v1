// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Support for project rendering.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{Orchestrator, Project, ProjectTitle, TrackUid};
}

pub use bus::{BusRoute, BusStation};
pub use ensnare::orchestration::{TrackTitle, TrackUid, TrackUidFactory};
pub use midi_router::MidiRouter;
pub use orchestrator::Orchestrator;
pub use project::{
    AudioSenderFn, Project, ProjectTitle, ProjectViewState, SignalChainItem, TrackViewMode,
};
pub(crate) use repositories::EntityRepository;

mod bus;
mod humidity;
mod midi_router;
mod orchestrator;
mod project;
mod repositories;
mod track;
