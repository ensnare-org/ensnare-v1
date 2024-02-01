// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui logic for drawing core Ensnare elements.

pub use {
    common::ColorSchemeConverter,
    controllers::{
        ArpeggiatorWidget, LfoControllerWidget, NoteSequencerWidget, PatternSequencerWidget,
        TripWidget,
    },
};

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{controllers::TripWidget, transport::TransportWidget};
}

pub(crate) mod common;
mod controllers;
pub mod effects;
pub mod instruments;
pub mod modulators;
pub mod transport;
pub mod widgets;
