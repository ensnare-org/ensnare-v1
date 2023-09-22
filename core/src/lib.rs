// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! Ensnare is a library for generating digital audio.

/// Handles automation, or real-time automatic control of one entity's
/// parameters by another entity's output.
pub mod control;
/// Common structures and constants used across the library.
pub mod core;
/// Infrastructure for managing [Entities](Entity).
pub mod entities;
/// Building blocks for signal generation.
pub mod generators;
/// Scaffolding for implementing instruments.
pub mod instruments;
/// MIDI-related functionality.
pub mod midi;
/// Building blocks for signal modulation.
pub mod modulators;
/// Infrastructure that coordinates [Entities](Entity).
pub mod orchestration;
/// Provides a random-number generator for debugging and testing.
pub mod rng;
/// A set of things that the user can select.
pub mod selection_set;
/// Handles digital-audio, wall-clock, and musical time.
pub mod time;
/// Groups [Entities](Entity) into tracks.
pub mod track;
/// Describes major system interfaces.
pub mod traits;
/// Unique identifiers.
pub mod uid;
/// Scaffolding for managing multiple voices.
pub mod voices;

// TEMP
mod drag_drop;
mod even_smaller_sequencer;
mod humidifier;
mod midi_router;
mod piano_roll;
mod widgets;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::control::{ControlIndex, ControlName, ControlValue};
    pub use crate::core::{
        BipolarNormal, FrequencyHz, Normal, ParameterType, Ratio, Sample, SampleType, SignalType,
        StereoSample,
    };
    pub use crate::entities::Key;
    pub use crate::time::{BeatValue, MusicalTime, SampleRate, Tempo, TimeSignature};
    pub use crate::uid::Uid;
}
