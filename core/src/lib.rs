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
/// Visual composition of patterns.
pub mod piano_roll;
/// Provides a random-number generator for debugging and testing.
pub mod rng;
/// A set of things that the user can select.
pub mod selection_set;
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
    pub use super::traits::prelude::*;
    pub use super::{
        control::{ControlIndex, ControlName, ControlValue},
        midi::prelude::*,
        time::{BeatValue, MusicalTime, SampleRate, Tempo, TimeSignature},
        types::{
            BipolarNormal, ChannelPair, FrequencyHz, Normal, ParameterType, Ratio, Sample,
            SampleType, SignalType, StereoSample,
        },
        uid::{TrackUid, TrackUidFactory, Uid, UidFactory},
    };
}
