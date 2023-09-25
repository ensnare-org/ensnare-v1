// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Ensnare is a library for generating digital audio.

/// Wraps the [cpal] audio interface and makes it easy to address with a
/// crossbeam channel.
pub mod audio;
/// Handles automation, or real-time automatic control of one entity's
/// parameters by another entity's output.
pub mod control;
/// Core controllers.
pub mod controllers;
/// Common structures and constants used across the library.
pub mod core;
/// Helps coodrinate systemwide drag-and-drop activity.
pub mod drag_drop;
// /// Infrastructure for managing [Entities](Entity).
// pub mod entities;
/// A very simple sequencer.
pub mod even_smaller_sequencer;
/// Building blocks for signal generation.
pub mod generators;
/// Scaffolding for implementing instruments.
pub mod instruments;
/// MIDI-related functionality.
pub mod midi;
/// Talking to external MIDI devices.
pub mod midi_interface;
/// Yet another sequencer.
pub mod mini_sequencer;
/// Building blocks for signal modulation.
pub mod modulators;
/// Infrastructure that coordinates [Entities](Entity).
pub mod orchestration;
/// Visual composition of patterns.
pub mod piano_roll;
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
/// Helper functions.
pub mod utils;
/// Scaffolding for managing multiple voices.
pub mod voices;

/// Large GUI components;
pub mod panels;
/// Drawing components.
pub mod widgets;

mod humidifier;
mod midi_router;

pub mod entities;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::control::{ControlIndex, ControlName, ControlValue};
    pub use super::core::{
        BipolarNormal, FrequencyHz, Normal, ParameterType, Ratio, Sample, SampleType, SignalType,
        StereoSample,
    };
    // pub use super::entities::{EntityFactory, EntityKey};
    // pub use super::orchestration::{Orchestrator, OrchestratorBuilder};
    pub use super::time::{BeatValue, MusicalTime, SampleRate, Tempo, TimeSignature};
    pub use super::uid::Uid;
}
