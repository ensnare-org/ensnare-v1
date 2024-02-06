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
/// MIDI-related functionality.
pub mod midi;
/// Provides a random-number generator for debugging and testing.
pub mod rng;
/// The data backing all project sequences.
pub mod sequence_repository;
/// Unique identifiers.
pub mod uid;
/// Helper functions.
pub mod utils;

pub use rng::Rng;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        composition::prelude::*, control::prelude::*, midi::prelude::*, uid::prelude::*,
    };
}
