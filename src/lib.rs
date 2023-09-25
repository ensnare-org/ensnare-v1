// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! TODO

/// Provides a version string that crates/apps can use.
pub mod version;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use ensnare_core::{
        control::{
            ControlIndex, ControlStepBuilder, ControlTripBuilder, ControlTripPath, ControlValue,
        },
        core::{BipolarNormal, FrequencyHz, Normal, Ratio, Sample, StereoSample},
        entities::{
            register_factory_entities,
            toys::{ToyController, ToyEffect, ToyInstrument},
            EntityFactory, EntityKey, EntityStore,
        },
        generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
        midi::{u4, u7, MidiChannel, MidiMessage, MidiMessagesFn, MidiNote},
        midi_interface::{MidiInterfaceEvent, MidiInterfaceInput, MidiPortDescriptor},
        orchestration::{Orchestrator, OrchestratorBuilder},
        piano_roll::{Note, PatternBuilder},
        time::{MusicalTime, SampleRate, Tempo, TimeSignature},
        traits::{
            Configurable, ControlEventsFn, Controllable, Controls, Displays, DisplaysInTimeline,
            Entity, HandlesMidi, HasSettings, HasUid, IsController, IsEffect, IsInstrument,
        },
        uid::Uid,
    };
}
