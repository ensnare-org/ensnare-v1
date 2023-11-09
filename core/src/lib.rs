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
/// Built-in musical devices.
pub mod entities;
// /// Infrastructure for managing [Entities](Entity).
// pub mod entities;
/// Building blocks for signal generation.
pub mod generators;
/// Scaffolding for implementing instruments.
pub mod instruments;
/// MIDI-related functionality.
pub mod midi;
/// Talking to external MIDI devices.
pub mod midi_interface;
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
/// Common structures and constants used across the library.
pub mod types;
/// Unique identifiers.
pub mod uid;
/// Helper functions.
pub mod utils;
/// Scaffolding for managing multiple voices.
pub mod voices;

mod bus_route;
mod humidifier;
mod midi_router;

pub mod stuff;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::traits::prelude::*;
    pub use super::{
        control::{ControlIndex, ControlName, ControlValue},
        entities::factory::{EntityFactory, EntityKey, EntityStore},
        midi::{MidiChannel, MidiMessage},
        orchestration::{OldOrchestrator, Orchestrator},
        time::{BeatValue, MusicalTime, SampleRate, Tempo, TimeSignature, ViewRange},
        track::{Track, TrackUid},
        types::{
            BipolarNormal, ChannelPair, FrequencyHz, Normal, ParameterType, Ratio, Sample,
            SampleType, SignalType, StereoSample,
        },
        uid::Uid,
    };
}
