// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Combines a composition with musical instruments and effects to produce a
//! rendition of the composition.

/// The most commonly used imports.
pub mod prelude {
    pub use super::Orchestrator;
}

pub use bus::{BusRoute, BusStation};
pub use midi_router::MidiRouter;
pub use orchestrator::Orchestrator;
pub(crate) use repositories::EntityRepository;

mod bus;
mod humidity;
mod midi_router;
mod orchestrator;
mod repositories;
