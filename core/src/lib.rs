// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! Ensnare is a library for generating digital audio.

/// Wraps the [cpal] audio interface and makes it easy to address with a
/// crossbeam channel.
pub mod audio;
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
pub mod drag_drop;
pub mod humidifier;
pub mod midi_router;
pub mod piano_roll;
pub mod widgets;

pub mod temp_impls;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::{
        control::{ControlIndex, ControlName, ControlValue},
        core::{
            BipolarNormal, FrequencyHz, Normal, ParameterType, Ratio, Sample, SampleType,
            SignalType, StereoSample,
        },
        entities::{EntityFactory, EntityKey},
        orchestration::{Orchestrator, OrchestratorBuilder},
        time::{BeatValue, MusicalTime, SampleRate, Tempo, TimeSignature},
        uid::Uid,
    };
}
