// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! The `ensnare` crate helps make digital music.

pub mod arrangement {
    //! Organization of musical elements.
    pub use ensnare_core::{
        orchestration::{Orchestrator, OrchestratorBuilder},
        time::{transport, Transport},
        track::{signal_chain, track_widget, Track, TrackAction, TrackTitle, TrackUid},
    };
}

pub mod composition {
    //! Creation of musical elements.
    pub use ensnare_core::piano_roll::{Note, PatternBuilder, PatternUid, PianoRoll};
}

pub mod control {
    //! [Automation](https://en.wikipedia.org/wiki/Mix_automation), or automatic
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

    pub use ensnare_core::control::{ControlIndex, ControlName, ControlRouter, ControlValue};
}

pub mod entities {
    //! Built-in musical instruments and supporting infrastructure.
    //!
    //! An [Entity](crate::traits::Entity) is a musical instrument. Entities
    //! generally fall into one of three classes: controllers, effects, and
    //! instruments. A controller controls other entities. An effect transforms
    //! audio. An instrument generates audio.
    //!
    //! Some entities are hybrids. For example, an arpeggiator responds to MIDI
    //! messages, so in that sense it acts like an instrument. But it also
    //! generates MIDI messages of its own, which makes it act like a
    //! controller.
    //!
    //! Every [Entity](crate::traits::Entity) must implement one of the `Is`
    //! traits: [IsController](crate::traits::IsController),
    //! [IsEffect](crate::traits::IsEffect),
    //! [IsInstrument](crate::traits::IsInstrument) (or one of the hybrids of
    //! these traits).

    pub use ensnare_core::entities::factory::{
        register_factory_entities, EntityFactory, EntityKey, EntityStore,
    };

    pub mod controllers {
        //! Built-in controllers. Controllers control other devices by generating MIDI and
        //! control events. Examples are sequencers, which generate MIDI, and LFOs,
        //! which generate control signals. Controllers implement
        //! [IsController](crate::traits::IsController).
        pub use ensnare_core::{
            controllers::{ControlStepBuilder, ControlTrip, ControlTripBuilder, ControlTripPath},
            entities::factory::test_entities::TestController,
            entities::{
                controllers::sequencers::{
                    live_pattern_sequencer_widget, LivePatternSequencer, MidiSequencer,
                    NoteSequencer, NoteSequencerBuilder, PatternSequencer, PatternSequencerBuilder,
                },
                controllers::*,
                toys::{ToyController, ToyControllerParams},
            },
        };
    }
    pub mod effects {
        //! Built-in effects. Effects transform audio. Examples are reverb and
        //! delay. Effects implement [IsEffect](crate::traits::IsEffect).
        pub use ensnare_core::entities::{
            effects::{gain::Gain, reverb::Reverb},
            factory::test_entities::{TestEffect, TestEffectNegatesInput},
            toys::{ToyEffect, ToyEffectParams},
        };
    }

    pub mod instruments {
        //! Instruments produce audio in response to MIDI messages. All synthesizers
        //! are instruments. They implement the
        //! [IsInstrument](crate::traits::IsInstrument) interface.
        pub use ensnare_core::{
            entities::{
                factory::test_entities::{TestInstrument, TestInstrumentCountsMidiMessages},
                instruments::*,
                toys::{ToyInstrument, ToyInstrumentParams, ToySynth, ToySynthParams},
            },
            instruments::Synthesizer,
            voices::{StealingVoiceStore, VoicePerNoteStore, VoiceStore},
        };
    }
}

pub mod generators {
    //! Signal generators. These are some of the building blocks of many digital
    //! instruments. Examples are envelopes and oscillators.
    pub use ensnare_core::generators::{
        Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform,
    };
}

pub mod midi {
    //! Management of all MIDI-related information that flows within the system.
    pub use ensnare_core::midi::{u4, u7, MidiChannel, MidiMessage, MidiNote};

    pub mod interface {
        //! External MIDI hardware, such as MIDI interfaces or MIDI keyboards
        //! plugged in through USB).
        pub use ensnare_core::midi_interface::{
            MidiInterfaceEvent, MidiInterfaceInput, MidiInterfaceService, MidiPortDescriptor,
        };
    }
}

pub mod modulators {
    //! Infrastructure for transforming audio. An example is [Dca], or the
    //! digitally-controlled amplifier, which many instruments use to control
    //! signal amplitude and stereo position.
    pub use ensnare_core::modulators::{Dca, DcaParams};
}

pub mod panels {
    //! Subsystems that typically run in their own thread and use crossbeam
    //! channels for communication. They also generally implement
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

pub mod traits {
    //! Common behaviors of system components.
    pub use ensnare_core::traits::*;
}

pub mod types {
    //! Common data types used throughout the system.
    pub use ensnare_core::{
        time::{MusicalTime, SampleRate, Tempo, TimeSignature, ViewRange},
        types::*,
        uid::{Uid, UidFactory},
    };
}

pub mod ui {
    //! Components that provide and coordinate the user interface.
    pub use ensnare_core::drag_drop::{DragDropManager, DragSource, DropTarget};
    pub use ensnare_core::widgets::audio::CircularSampleBuffer;
    pub mod widgets {
        //! `widgets` contains egui `Widget`s that help draw things.
        pub use ensnare_core::panels::{audio_settings, midi_settings};
        pub use ensnare_core::widgets::{
            audio, core, generators, pattern, placeholder, timeline, track,
        };
    }
}

pub mod utils {
    //! Various helpers.
    pub use ensnare_core::utils::Paths;
}

mod version;

pub use version::app_version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        arrangement::{Orchestrator, OrchestratorBuilder, Track, TrackTitle, TrackUid, Transport},
        composition::{Note, PatternBuilder, PatternUid, PianoRoll},
        control::{ControlIndex, ControlName, ControlRouter, ControlValue},
        entities::{
            controllers::{ControlStepBuilder, ControlTripBuilder, ControlTripPath},
            register_factory_entities, EntityFactory, EntityKey, EntityStore,
        },
        generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
        midi::{
            interface::{MidiInterfaceEvent, MidiInterfaceInput, MidiPortDescriptor},
            u4, u7, MidiChannel, MidiMessage, MidiNote,
        },
        modulators::{Dca, DcaParams},
        traits::{
            Acts, Configurable, ControlEventsFn, Controllable, Controls, Displays, Entity,
            EntityEvent, HandlesMidi, HasMetadata, HasSettings, IsAction, IsController, IsEffect,
            IsInstrument, MidiMessagesFn, Orchestrates,
        },
        types::{
            BipolarNormal, ChannelPair, FrequencyHz, MusicalTime, Normal, Ratio, Sample,
            SampleRate, StereoSample, Tempo, TimeSignature, Uid, UidFactory, ViewRange,
        },
        ui::DragDropManager,
    };
}
