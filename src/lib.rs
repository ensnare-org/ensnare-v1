// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! The `ensnare` crate helps make digital music.

pub mod arrangement {
    //! Organization of musical elements.

    pub use ensnare_core::time::Transport;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::Transport;
    }
}

pub mod composition {
    //! Creation of musical elements.

    pub use ensnare_core::{
        piano_roll::{Note, Pattern, PatternBuilder, PatternUid, PianoRoll},
        sequence_repository::{Sequence, SequenceRepository, SequenceUid},
    };

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
    pub use ensnare_orchestration::ControlRouter;

    pub use ensnare_core::controllers::{
        ControlStep, ControlStepBuilder, ControlTrip, ControlTripBuilder, ControlTripParams,
        ControlTripPath,
    };

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{ControlIndex, ControlName, ControlRouter, ControlValue};
    }
}

pub mod cores {
    //! The core business logic that powers musical instruments.
    pub use ensnare_core::controllers::{TimerParams, TriggerParams};
    pub use ensnare_cores::{controllers::*, effects::*, instruments::*, toys::*};
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

    pub use ensnare_core::uid::{EntityUidFactory, TrackUidFactory};
    pub use ensnare_entity::factory::{EntityFactory, EntityKey, EntityStore};

    pub mod controllers {
        //! Controllers implement the
        //! [IsController](crate::traits::IsController) trait, which means that
        //! they control other devices. An example of a controller is a
        //! [PatternSequencer], which replays patterns of MIDI messages.
        //!
        //! Generally, controllers produce only control signals, and not audio.
        //! But adapters exist that change one kind of signal into another, such
        //! as [SignalPassthroughController], which is used in
        //! [sidechaining](https://en.wikipedia.org/wiki/Dynamic_range_compression#Side-chaining).
        //! In theory, a similar adapter could be used to change a control
        //! signal into an audio signal.
        pub use ensnare_entities::controllers::*;
    }
    pub mod effects {
        //! Effects implement the [IsEffect](crate::traits::IsEffect) trait,
        //! which means that they transform audio. They don't produce their own
        //! audio, and while they don't produce control signals, most of them do
        //! respond to controls. Examples of effects are [Compressor] and
        //! [Reverb].
        pub use ensnare_entities::effects::*;
    }

    pub mod instruments {
        //! Instruments play sounds. They implement the
        //! [IsInstrument](crate::traits::IsInstrument) trait, which means that
        //! they respond to MIDI and produce
        //! [StereoSamples](crate::types::StereoSample). Examples of instruments
        //! are [Sampler] and [WelshSynth].
        pub use ensnare_entities::instruments::*;
    }

    pub mod toys {
        //! Extremely simple implementations of various types of entities.
        pub use ensnare_entities_toy::{controllers::*, effects::*, instruments::*};
    }

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            controllers::{Timer, Trigger},
            EntityFactory, EntityKey, EntityStore,
        };
        pub use ensnare_entities::BuiltInEntities;
        pub use ensnare_entities_toy::ToyEntities;
    }
}

pub mod generators {
    //! Signal generators. These are some of the building blocks of many digital
    //! instruments. Examples are envelopes and oscillators.
    pub use ensnare_core::generators::{
        Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform,
    };

    /// The most commonly used imports.
    pub mod prelude {
        pub use ensnare_core::generators::{Envelope, Oscillator, Waveform};
    }
}

pub mod midi {
    //! Management of all MIDI-related information that flows within the system.
    pub use ensnare_core::midi::{u4, u7, MidiChannel, MidiMessage, MidiNote};

    pub mod interface {
        //! External MIDI hardware, such as MIDI interfaces or MIDI keyboards
        //! plugged in through USB).
        pub use ensnare_core::midi_interface::{
            MidiInterfaceService, MidiInterfaceServiceEvent, MidiInterfaceServiceInput,
            MidiPortDescriptor,
        };
    }

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            interface::{MidiInterfaceServiceEvent, MidiInterfaceServiceInput, MidiPortDescriptor},
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

pub mod synthesizer {
    //! Infrastructure for assembling components into polyphonic musical
    //! instruments.
    pub use ensnare_core::{
        instruments::Synthesizer,
        voices::{StealingVoiceStore, VoicePerNoteStore, VoiceStore},
    };
}

pub mod services {
    //! Services typically run in their own thread and use crossbeam channels
    //! for communication. Some also provide a UI panel that helps visualize and
    //! manage the subsystem, which means that they implement
    //! [Displays](crate::traits::Displays).

    pub use ensnare_services::{
        AudioService, AudioServiceEvent, AudioServiceInput, AudioSettings, ControlBar, MidiService,
        MidiServiceEvent, MidiServiceInput, MidiSettings, ProjectService, ProjectServiceEvent,
        ProjectServiceInput,
    };

    /// `use ensnare::systems::prelude::*;` when working with services.
    pub mod prelude {
        pub use super::{
            AudioService, AudioServiceEvent, AudioServiceInput, AudioSettings, ControlBar,
            MidiService, MidiServiceEvent, MidiSettings, ProjectService, ProjectServiceEvent,
            ProjectServiceInput,
        };
    }
}

pub mod traits {
    //! Common behaviors of system components.
    pub use ensnare_core::traits::*;
    pub use ensnare_entity::traits::*;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            Configurable, ControlEventsFn, Controllable, Controls, Displays, EntityBounds,
            Generates, GeneratesToInternalBuffer, HandlesMidi, HasMetadata, HasSettings,
            MidiMessagesFn, Sequences, Serializable, Ticks, WorkEvent,
        };
    }
}

pub mod types {
    //! Common data types used throughout the system.
    pub use ensnare_core::{
        time::{MusicalTime, SampleRate, Tempo, TimeRange, TimeSignature, ViewRange},
        types::*,
        uid::{TrackUid, TrackUidFactory, Uid, UidFactory},
    };

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            BipolarNormal, ChannelPair, FrequencyHz, MusicalTime, Normal, Ratio, Sample,
            SampleRate, StereoSample, Tempo, TimeRange, TimeSignature, TrackTitle, TrackUid,
            TrackUidFactory, Uid, UidFactory, ViewRange,
        };
    }
}

pub mod ui {
    //! Components that provide and coordinate the user interface.
    pub mod widgets {
        //! `widgets` contains egui `Widget`s that help draw things.
        pub use ensnare_cores_egui::{
            piano_roll::piano_roll,
            prelude::*,
            widgets::{audio, pattern, placeholder, timeline},
        };
        pub use ensnare_orchestration::egui::entity_palette;
        pub use ensnare_services::{
            audio_settings, control_bar_widget, midi_settings, ControlBarAction,
        };
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

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::*;
    }
}

pub use version::app_version;

mod version;

pub mod all_entities;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        arrangement::prelude::*, composition::prelude::*, control::prelude::*,
        entities::prelude::*, generators::prelude::*, midi::prelude::*, services::prelude::*,
        traits::prelude::*, types::prelude::*, ui::prelude::*, utils::prelude::*,
    };
    pub use ensnare_cores_egui::prelude::*;
    pub use ensnare_new_stuff::project::Project;
    pub use ensnare_orchestration::prelude::*;
}
