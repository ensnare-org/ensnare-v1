// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! The `ensnare` crate helps make digital music.

pub mod arrangement {
    //! Organization of musical elements.

    pub use super::project::{Project, ProjectTitle};
    pub use ensnare_core::time::Transport;
    pub use ensnare_orchestration::orchestration::{
        OldOrchestrator, Orchestrator, OrchestratorHelper,
    };

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{OldOrchestrator, Orchestrator, OrchestratorHelper, Transport};
    }
}

pub mod composition {
    //! Creation of musical elements.
    pub use ensnare_core::piano_roll::{Note, PatternBuilder, PatternUid, PianoRoll};

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{Note, PatternBuilder, PatternUid, PianoRoll};
    }
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

    pub use ensnare_core::control::{ControlIndex, ControlName, ControlValue};
    pub use ensnare_egui::control::ControlRouter;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{ControlIndex, ControlName, ControlRouter, ControlValue};
    }
}

pub mod devices {
    //! The core business logic that powers musical instruments.
    pub use ensnare_cores::controllers::sequencers::pattern::{
        PatternSequencer, PatternSequencerBuilder,
    };
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

    pub use ensnare_entity::factory::{EntityFactory, EntityKey, EntityStore};

    pub mod controllers {
        //! Built-in controllers. Controllers control other devices by generating MIDI and
        //! control events. Examples are sequencers, which generate MIDI, and LFOs,
        //! which generate control signals. Controllers implement
        //! [IsController](crate::traits::IsController).
        pub use ensnare_core::controllers::{
            ControlStepBuilder, ControlTrip, ControlTripBuilder, ControlTripPath,
        };
        pub use ensnare_entity::test_entities::TestController;
        pub use ensnare_factory_entities::controllers::{
            LivePatternSequencer, MidiSequencer, NoteSequencer, PatternSequencer,
        };
    }
    pub mod effects {
        //! Built-in effects. Effects transform audio. Examples are reverb and
        //! delay. Effects implement [IsEffect](crate::traits::IsEffect).
        pub use ensnare_factory_entities::effects::*;
    }

    pub mod instruments {
        //! Instruments produce audio in response to MIDI messages. All synthesizers
        //! are instruments. They implement the
        //! [IsInstrument](crate::traits::IsInstrument) interface.
        pub use ensnare_core::{
            instruments::Synthesizer,
            voices::{StealingVoiceStore, VoicePerNoteStore, VoiceStore},
        };
        pub use ensnare_factory_entities::instruments::*;
    }

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            controllers::{ControlStepBuilder, ControlTripBuilder, ControlTripPath},
            EntityFactory, EntityKey, EntityStore,
        };
        pub use ensnare_factory_entities::prelude::register_factory_entities;
        pub use ensnare_toy_entities::prelude::register_toy_entities;
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

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            interface::{MidiInterfaceEvent, MidiInterfaceInput, MidiPortDescriptor},
            u4, u7, MidiChannel, MidiMessage, MidiNote,
        };
    }
}

pub mod modulators {
    //! Infrastructure for transforming audio. An example is [Dca], or the
    //! digitally-controlled amplifier, which many instruments use to control
    //! signal amplitude and stereo position.
    pub use ensnare_core::modulators::{Dca, DcaParams};
}

pub mod systems {
    //! Subsystems that typically run in their own thread and use crossbeam
    //! channels for communication. They also generally implement
    //! [Displays](crate::traits::Displays), so they also provide a UI panel
    //! that helps visualize and manage the subsystem.

    /// `use ensnare::systems::prelude::*;` when working with systems.
    pub mod prelude {
        pub use super::{
            AudioPanel, AudioPanelEvent, AudioSettings, ControlPanel, ControlPanelAction,
            MidiPanel, MidiPanelEvent, MidiSettings, NeedsAudioFn, OrchestratorEvent,
            OrchestratorInput, OrchestratorPanel, PalettePanel,
        };
    }

    pub use ensnare_systems::{
        AudioPanel, AudioPanelEvent, AudioSettings, ControlPanel, ControlPanelAction, MidiPanel,
        MidiPanelEvent, MidiSettings, NeedsAudioFn, OrchestratorEvent, OrchestratorInput,
        OrchestratorPanel, PalettePanel,
    };
}

pub mod traits {
    //! Common behaviors of system components.
    pub use ensnare_core::traits::*;
    pub use ensnare_entity::traits::*;
    pub use ensnare_orchestration::traits::*;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            Acts, Configurable, ControlEventsFn, Controllable, Controls, Displays, Entity,
            EntityEvent, Generates, GeneratesToInternalBuffer, HandlesMidi, HasMetadata,
            HasSettings, IsController, IsEffect, IsInstrument, MidiMessagesFn, Orchestrates,
            Sequences, Serializable, Ticks,
        };
    }
}

pub mod types {
    //! Common data types used throughout the system.
    pub use ensnare_core::{
        time::{MusicalTime, SampleRate, Tempo, TimeSignature, ViewRange},
        types::*,
        uid::{Uid, UidFactory},
    };

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            BipolarNormal, ChannelPair, FrequencyHz, MusicalTime, Normal, Ratio, Sample,
            SampleRate, StereoSample, Tempo, TimeSignature, Uid, UidFactory, ViewRange,
        };
    }
}

pub mod ui {
    //! Components that provide and coordinate the user interface.
    pub use ensnare_egui::widgets::audio::CircularSampleBuffer;
    pub mod widgets {
        //! `widgets` contains egui `Widget`s that help draw things.
        pub use ensnare_egui::widgets::{audio, pattern, placeholder, timeline, track};
        pub use ensnare_systems::{audio_settings, midi_settings};
    }
    pub use ensnare_drag_drop::{DragDropManager, DragSource, DropTarget};

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{DragDropManager, DragSource, DropTarget};
    }
}

pub mod utils {
    //! Various helpers.
    pub use ensnare_core::utils::Paths;
}

pub use project::Project;
pub use version::app_version;

mod project;
mod version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::arrangement::prelude::*;
    pub use super::composition::prelude::*;
    pub use super::control::prelude::*;
    pub use super::entities::prelude::*;
    pub use super::midi::prelude::*;
    pub use super::systems::prelude::*;
    pub use super::traits::prelude::*;
    pub use super::types::prelude::*;
    pub use super::ui::prelude::*;
    // pub use super::{
    //     generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
    //     modulators::{Dca, DcaParams},
    //     project::{Project, ProjectTitle},
    // };
    pub use ensnare_egui::prelude::*;
    pub use ensnare_orchestration::prelude::*;
}
