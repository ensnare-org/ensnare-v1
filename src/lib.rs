// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! The `ensnare` crate helps make digital music.

pub mod automation {
    //! Automation lets a project change the parameters of instruments and
    //! effects over time in a programmatic, reproducible way.
    //!
    //! For example, suppose a producer wants a pan effect going
    //! left-right-left-right throughout the whole song. This could be done by
    //! manually turning a knob back and forth, but that's tedious, and it
    //! especially won't work when rendering the final output to a song file.
    //!
    //! Using automation, the producer can instead configure an LFO to emit a
    //! [ControlValue] each time its value changes, and then link that output to
    //! a synthesizer's pan parameter. Then the synth's pan changes with the LFO
    //! output, automatically and identically for each performance of the song.
    //!
    //! Controllable entities have one or more parameters that are addressable
    //! by [ControlName] or [ControlIndex], which are discoverable through the
    //! [Controllable](crate::traits::Controllable) trait. The
    //! [Control](ensnare_proc_macros::Control) derive macro, with #[control]
    //! derive parameters, usually implements this trait.
    //!
    //! All values that pass through the automation subsystem are normalized to
    //! [ControlValue]s, which range from 0..=1.0. Sensible mappings exist for
    //! all applicable types in the system.

    pub use ensnare_core::{
        control::{ControlIndex, ControlName, ControlValue},
        controllers::{
            ControlStep, ControlStepBuilder, ControlTrip, ControlTripBuilder, ControlTripPath,
        },
        traits::{ControlEventsFn, Controllable, Controls},
    };

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{ControlIndex, ControlName, ControlValue};
    }
}

pub mod composition {
    //! Creation and representation of music scores.

    pub use ensnare_core::composition::{Note, Pattern, PatternBuilder, PatternUid};
    pub use ensnare_new_stuff::Composer;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{Composer, Note, PatternBuilder, PatternUid};
    }
}

pub mod elements {
    //! Building blocks that make up musical instruments and effects.

    pub use ensnare_core::{
        generators::{Envelope, Oscillator, Waveform},
        instruments::Synthesizer,
        modulators::Dca,
        voices::{StealingVoiceStore, VoicePerNoteStore, VoiceStore},
    };

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{Dca, Envelope, Oscillator, StealingVoiceStore, Synthesizer, Waveform};
    }
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
    pub use ensnare_core::uid::TrackUidFactory;
    pub use ensnare_entity::{
        factory::{EntityFactory, EntityKey, EntityStore},
        EntityUidFactory,
    };

    pub mod controllers {
        //! Controllers control other devices. An example of a controller is a
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
        //! Effects transform audio. They don't produce their own audio, and
        //! while they don't produce control signals, most of them do respond to
        //! controls. Examples of effects are [Compressor] and [Reverb].
        pub use ensnare_entities::effects::*;
    }

    pub mod instruments {
        //! Instruments play sounds. They respond to MIDI and produce
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

pub mod services {
    //! Services typically run in their own thread and use crossbeam channels
    //! for communication. Some also provide a UI panel that helps visualize and
    //! manage the subsystem, which means that they implement
    //! [Displays](crate::traits::Displays).

    pub use ensnare_services::{
        AudioService, AudioServiceEvent, AudioServiceInput, AudioSettings, MidiService,
        MidiServiceEvent, MidiServiceInput, MidiSettings, ProjectService, ProjectServiceEvent,
        ProjectServiceInput,
    };

    /// `use ensnare::systems::prelude::*;` when working with services.
    pub mod prelude {
        pub use super::{
            AudioService, AudioServiceEvent, AudioServiceInput, AudioSettings, MidiService,
            MidiServiceEvent, MidiServiceInput, MidiSettings, ProjectService, ProjectServiceEvent,
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
            Generates, HandlesMidi, HasExtent, HasMetadata, HasSettings, MidiMessagesFn, Sequences,
            Serializable, WorkEvent,
        };
    }
}

pub mod transport {
    //! Time management.
    pub use ensnare_core::time::Transport;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::Transport;
    }
}

pub mod types {
    //! Common data types used throughout the system.
    pub use ensnare_core::{
        time::{MusicalTime, SampleRate, Tempo, TimeRange, TimeSignature, ViewRange},
        types::*,
        uid::{TrackUid, TrackUidFactory, UidFactory},
    };
    pub use ensnare_entity::Uid;

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
        pub use ensnare_cores_egui::{prelude::*, widgets::pattern};
        pub use ensnare_egui_widgets::{
            ControlBar, ControlBarAction, ControlBarWidget, ObliqueStrategiesWidget,
        };
        pub use ensnare_new_stuff::egui::*;
        pub use ensnare_services::{AudioSettingsWidget, MidiSettingsWidget};
    }
    pub use ensnare_drag_drop::{DragDropManager, DragSource, DropTarget};

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            widgets::{
                AudioSettingsWidget, ControlBar, ControlBarAction, ControlBarWidget,
                EntityPaletteWidget, MidiSettingsWidget, ProjectAction, TimelineIconStripAction,
                TimelineIconStripWidget,
            },
            DragDropManager, DragSource, DropTarget,
        };
    }
}

pub mod utils {
    //! Various helpers.
    pub use ensnare_core::utils::Paths;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::Paths;
    }
}

pub use all_entities::EnsnareEntities;
pub use version::app_version;

mod all_entities;
mod version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        automation::prelude::*, composition::prelude::*, elements::prelude::*,
        entities::prelude::*, midi::prelude::*, services::prelude::*, traits::prelude::*,
        transport::prelude::*, types::prelude::*, ui::prelude::*, utils::prelude::*,
        EnsnareEntities,
    };
    pub use ensnare_cores_egui::prelude::*;
    pub use ensnare_new_stuff::project::Project;
}
