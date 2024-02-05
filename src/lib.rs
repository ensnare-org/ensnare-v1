// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! The `ensnare` crate helps make digital music.

pub use automation::Automator;
pub use composition::Composer;
pub use orchestration::Orchestrator;

// pub mod compositionx {
//     //! Creation and representation of music scores.

//     pub use ensnare_core::composition::{Note, Pattern, PatternBuilder, PatternUid};

//     /// The most commonly used imports.
//     pub mod prelude {
//         pub use super::{Note, PatternBuilder, PatternUid};
//     }
// }

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

    // pub mod controllers {
    //     //! Controllers control other devices. An example of a controller is a
    //     //! [PatternSequencer], which replays patterns of MIDI messages.
    //     //!
    //     //! Generally, controllers produce only control signals, and not audio.
    //     //! But adapters exist that change one kind of signal into another, such
    //     //! as [SignalPassthroughController], which is used in
    //     //! [sidechaining](https://en.wikipedia.org/wiki/Dynamic_range_compression#Side-chaining).
    //     //! In theory, a similar adapter could be used to change a control
    //     //! signal into an audio signal.
    //     pub use ensnare_entities::controllers::*;
    // }
    // pub mod effects {
    //     //! Effects transform audio. They don't produce their own audio, and
    //     //! while they don't produce control signals, most of them do respond to
    //     //! controls. Examples of effects are [Compressor] and [Reverb].
    //     pub use ensnare_entities::effects::*;
    // }

    // pub mod instruments {
    //     //! Instruments play sounds. They respond to MIDI and produce
    //     //! [StereoSamples](crate::types::StereoSample). Examples of instruments
    //     //! are [Sampler] and [WelshSynth].
    //     pub use ensnare_entities::instruments::*;
    // }

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            //     controllers::{Timer, Trigger},
            EntityFactory,
            EntityKey,
            EntityStore,
        };
        // pub use ensnare_entities::BuiltInEntities;
        // pub use ensnare_toys::ToyEntities;
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
    pub use crate::types_future::*;
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
        pub use super::Paths;
    }
}

pub use all_entities::EnsnareEntities;
pub use version::app_version;

pub use ensnare_drag_drop::{DragDropManager, DragSource, DropTarget};

mod all_entities;
mod version;

pub mod automation;
pub mod composition;
pub mod egui;
pub mod entities_future;
pub mod midi;
pub mod orchestration;
pub mod project;
pub mod services;
pub mod types_future;

//pub use project::{ProjectAction, ProjectWidget};
// pub use track::{make_title_bar_galley, TitleBarWidget, TrackWidget};

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        automation::prelude::*, composition::prelude::*, egui::prelude::*, elements::prelude::*,
        entities::prelude::*, entities_future::prelude::*, midi::prelude::*,
        orchestration::prelude::*, project::prelude::*, services::prelude::*, traits::prelude::*,
        transport::prelude::*, types::prelude::*, ui::prelude::*, utils::prelude::*,
        EnsnareEntities,
    };
    pub use ensnare_cores_egui::prelude::*;
}
