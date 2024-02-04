// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Ensnare is a library for generating digital audio.

#![cfg_attr(not(feature = "std"), no_std)]

/// Structs that represent music.
pub mod composition;
/// Handles automation, or real-time automatic control of one entity's
/// parameters by another entity's output.
pub mod control;
/// Core controllers.
pub mod controllers;
// /// Infrastructure for managing [Entities](Entity).
// pub mod entities;
/// Building blocks for signal generation.
pub mod generators;
/// Scaffolding for implementing instruments.
pub mod instruments;
/// MIDI-related functionality.
pub mod midi;
/// Building blocks for signal modulation.
pub mod modulators;
/// Provides a random-number generator for debugging and testing.
pub mod rng;
/// A set of things that the user can select.
pub mod selection_set;
/// The data backing all project sequences.
pub mod sequence_repository;
/// Handles digital-audio, wall-clock, and musical time.
pub mod time;
/// Describes major system interfaces.
pub mod traits;
/// Common structures and constants used across the library.
pub mod types;
/// Unique identifiers.
pub mod uid;
/// Helper functions.
pub mod utils;
/// Scaffolding for managing multiple voices.
pub mod voices;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        composition::prelude::*, control::prelude::*, generators::prelude::*, midi::prelude::*,
        time::prelude::*, traits::prelude::*, types::prelude::*, uid::prelude::*,
    };
}
