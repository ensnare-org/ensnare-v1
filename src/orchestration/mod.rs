// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Renders compositions with musical instruments and effects.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{Orchestrator, TrackUid};
}

pub use bus::{BusRoute, BusStation};
pub use midi_router::MidiRouter;
pub use orchestrator::Orchestrator;
pub(crate) use repositories::EntityRepository;
pub use track::{TrackTitle, TrackUid, TrackUidFactory};

mod bus;
mod humidity;
mod midi_router;
mod orchestrator;
mod repositories;
mod track;
