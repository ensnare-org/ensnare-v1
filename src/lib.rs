// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! Ensnare helps make music.

pub mod traits {
    //! `traits` specifies many of the common behaviors of system components.
    //! Important if you want to create more musical instruments.
    pub use ensnare_core::traits::*;
}

pub mod types {
    //! `types` specifies the common data types used throughout the system.
    pub use ensnare_core::{
        time::{MusicalTime, SampleRate, Tempo, TimeSignature},
        types::*,
        uid::Uid,
    };
}

pub mod controllers {
    //! Controllers control other devices by generating MIDI and control events.
    //! Examples are sequencers and arpeggiators. They implement the
    //! [IsController](crate::traits::IsController) interface.
    pub use ensnare_core::{
        controllers::{
            ControlAtlas, ControlAtlasBuilder, ControlStepBuilder, ControlTripBuilder,
            ControlTripPath,
        },
        entities::factory::test_entities::TestController,
        entities::{
            controllers::*,
            toys::{ToyController, ToyControllerParams},
        },
        even_smaller_sequencer::{ESSequencer, ESSequencerBuilder},
        mini_sequencer::Sequencer,
    };
}

pub mod effects {
    //! Effects transform audio. Examples are reverb and delay. They implement
    //! the [IsEffect](crate::traits::IsEffect) interface.
    pub use ensnare_core::entities::{
        effects::*,
        factory::test_entities::{TestEffect, TestEffectNegatesInput},
        toys::{ToyEffect, ToyEffectParams},
    };
}

pub mod instruments {
    //! Instruments produce audio in response to MIDI messages. All synthesizers
    //! are instruments. They implement the
    //! [IsInstrument](crate::traits::IsInstrument) interface.
    pub use ensnare_core::entities::{
        factory::test_entities::{TestInstrument, TestInstrumentCountsMidiMessages},
        instruments::*,
        toys::{ToyInstrument, ToyInstrumentParams, ToySynth, ToySynthParams},
    };
}

pub mod entity {
    //! The `entity` infrastructure supports the [Entity](crate::traits::Entity)
    //! trait. All Ensnare musical instruments, effects, and controllers are
    //! [Entities](crate::traits::Entity).
    pub use ensnare_core::entities::factory::{
        register_factory_entities, EntityFactory, EntityKey, EntityStore,
    };
}

pub mod control {
    //! `control` is the infrastructure enabling automation, or automatic
    //! control of device parameters.
    //!
    //! For example, an LFO might emits a [ControlValue] each time its value
    //! changes. If a synthesizer's pan parameter is linked to that
    //! [ControlValue], then the synth pan changes with the LFO output.
    //!
    //! Entities that control others implement the
    //! [Controls](crate::traits::Controls) trait.
    //!
    //! Controllable entities have one or more parameters that are addressable
    //! by [ControlName] or [ControlIndex], which are discoverable through the
    //! [Controllable](crate::traits::Controllable) trait.
    //!
    //! [ControlRouter] manages the relationships between controllers and
    //! controlled entities.
    //!
    //! All values that pass through the control subsystem are normalized to
    //! [ControlValue]s, which range from 0..=1.0. Sensible mappings exist for
    //! all applicable types in the system.
    //!
    //! Alert! [ControlAtlas](crate::controllers::ControlAtlas) might seem like
    //! it should be here, but it is actually in the `controllers` module. This
    //! is because it is a device rather than part of the infrastructure.

    pub use ensnare_core::control::{ControlIndex, ControlName, ControlRouter, ControlValue};
}

pub mod midi {
    //! `midi` manages all MIDI-related information that flows within the
    //! system. If you're looking for MIDI that enters or leaves the system
    //! (e.g., MIDI interfaces or MIDI keyboards plugged in through USB), see
    //! [midi_interface](crate::midi_interface) instead.
    pub use ensnare_core::midi::{u4, u7, MidiChannel, MidiMessage, MidiNote};
}

pub mod generators {
    //! `generators` generate signals, and are some of the building blocks of
    //! many digital instruments. Examples are envelopes and oscillators.
    pub use ensnare_core::generators::{
        Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform,
    };
}

pub mod midi_interface {
    //! `midi_interface` manages external MIDI interfaces.
    pub use ensnare_core::midi_interface::{
        MidiInterfaceEvent, MidiInterfaceInput, MidiInterfaceService, MidiPortDescriptor,
    };
}

pub mod arrangement {
    //! `arrangement` helps organize devices musically.
    pub use ensnare_core::{
        orchestration::{Orchestrator, OrchestratorBuilder},
        track::{DeviceChain, DeviceChainAction, Track, TrackTitle, TrackUid},
    };
}

pub mod composition {
    //! `composition` contains components useful for composing music.
    pub use ensnare_core::piano_roll::{Note, PatternBuilder, PatternUid, PianoRoll};
}

pub mod modulation {
    //! `modulation` infrastructure transforms audio.
    pub use ensnare_core::modulators::{Dca, DcaParams};
}

pub mod ui {
    //! `ui` contains components that help provide the user interface.
    pub use ensnare_core::drag_drop::{DragDropEvent, DragDropManager, DragDropSource};
    pub use ensnare_core::widgets::audio::CircularSampleBuffer;
    pub mod widgets {
        //! `widgets` contains egui `Widget`s that help draw things.
        pub use ensnare_core::panels::{audio_settings, midi_settings};
        pub use ensnare_core::widgets::{
            audio, control, controllers, core, generators, pattern, placeholder, timeline, track,
        };
    }
}

pub mod panels {
    //! `panels` are subsystems that typically run in their own thread and use
    //! crossbeam channels for communication. They also generally implement
    //! [Displays](crate::traits::Displays), so they also provide a UI panel
    //! that helps visualize and manage the subsystem.

    /// `use ensnare::panels::prelude::*;` when working with panels.
    pub mod prelude {
        pub use super::{
            AudioPanel, AudioPanelEvent, AudioSettings, ControlPanel, ControlPanelAction,
            MidiPanel, MidiPanelEvent, MidiSettings, NeedsAudioFn, OrchestratorEvent,
            OrchestratorInput, OrchestratorPanel, PalettePanel,
        };
    }
    pub use ensnare_core::panels::{
        AudioPanel, AudioPanelEvent, AudioSettings, ControlPanel, ControlPanelAction, MidiPanel,
        MidiPanelEvent, MidiSettings, NeedsAudioFn, OrchestratorEvent, OrchestratorInput,
        OrchestratorPanel, PalettePanel,
    };
}

pub mod util {
    //! Various helpers.
}

pub mod version;

/// `use ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        arrangement::{
            DeviceChain, DeviceChainAction, Orchestrator, OrchestratorBuilder, Track, TrackTitle,
            TrackUid,
        },
        composition::{Note, PatternBuilder, PatternUid, PianoRoll},
        control::{ControlIndex, ControlName, ControlRouter, ControlValue},
        controllers::{
            lfo::{LfoController, LfoControllerParams},
            ControlAtlas, ControlAtlasBuilder, ControlStepBuilder, ControlTripBuilder,
            ControlTripPath, ESSequencer, ESSequencerBuilder, Sequencer,
        },
        entity::{register_factory_entities, EntityFactory, EntityKey, EntityStore},
        generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
        midi::{u4, u7, MidiChannel, MidiMessage, MidiNote},
        midi_interface::{MidiInterfaceEvent, MidiInterfaceInput, MidiPortDescriptor},
        modulation::{Dca, DcaParams},
        traits::{
            Acts, Configurable, ControlEventsFn, Controllable, Controls, Displays,
            DisplaysInTimeline, Entity, EntityEvent, HandlesMidi, HasSettings, HasUid,
            IsController, IsEffect, IsInstrument, MidiMessagesFn, Orchestrates,
        },
        types::{
            BipolarNormal, FrequencyHz, MusicalTime, Normal, Ratio, Sample, SampleRate,
            StereoSample, Tempo, TimeSignature, Uid,
        },
        ui::DragDropManager,
    };
}
