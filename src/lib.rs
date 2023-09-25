// Copyright (c) 2023 Mike Tsao. All rights reserved.

/// Core instruments, controllers, and effects.
pub mod entities;

/// Provides a version string that crates/apps can use.
pub mod version;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::entities::register_factory_entities;
    pub use ensnare_core::{
        control::{
            ControlIndex, ControlStepBuilder, ControlTripBuilder, ControlTripPath, ControlValue,
        },
        core::{BipolarNormal, FrequencyHz, Normal, Ratio, Sample, StereoSample},
        entities::{EntityFactory, EntityKey, EntityStore},
        generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
        midi::{u4, u7, MidiChannel, MidiMessage, MidiMessagesFn, MidiNote},
        orchestration::{Orchestrator, OrchestratorBuilder},
        piano_roll::{Note, PatternBuilder},
        time::{MusicalTime, SampleRate, Tempo, TimeSignature},
        traits::{
            Configurable, ControlEventsFn, Controllable, Controls, Displays, DisplaysInTimeline,
            Entity, HandlesMidi, HasSettings, HasUid, IsController, IsEffect, IsInstrument,
        },
        uid::Uid,
    };
    pub use ensnare_midi_interface::{MidiInterfaceEvent, MidiInterfaceInput, MidiPortDescriptor};
    pub use ensnare_toys::{ToyController, ToyEffect, ToyInstrument};
}
pub use ensnare_core::core::*;
pub use ensnare_core::orchestration::*;
pub use ensnare_core::traits::*;
pub use ensnare_midi_interface::*;
pub use ensnare_not_core::*;
