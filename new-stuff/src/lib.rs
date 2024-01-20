// Copyright (c) 2024 Mike Tsao. All rights reserved.

pub use automation::Automator;
pub use composition::Composer;
pub use midi::MidiRouter;
pub use orchestration::{BusRoute, BusStation, Orchestrator};

mod automation;
mod composition;
pub mod egui;
mod midi;
mod orchestration;
pub mod project;
pub mod repositories;
pub mod types;
