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
    pub use crate::core::uid::TrackUidFactory;

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
        // pub use ensnare_entities::BuiltInEntities;
        // pub use ensnare_toys::ToyEntities;
    }
}

pub mod traits {
    //! Common behaviors of system components.
    pub use crate::core::traits::*;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            Configurable, ControlEventsFn, Controllable, Controls, Generates, HandlesMidi,
            HasExtent, HasSettings, MidiMessagesFn, Sequences, Serializable, WorkEvent,
        };
    }
}

pub mod transport {
    //! Time management.
    pub use crate::core::time::Transport;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::Transport;
    }
}

pub mod types {
    //! Common data types used throughout the system.
    pub use crate::core::{
        time::{MusicalTime, SampleRate, Tempo, TimeRange, TimeSignature, ViewRange},
        types::*,
        uid::{TrackUid, TrackUidFactory, UidFactory},
    };
    pub use crate::{types_future::*, uid::Uid};

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::{
            BipolarNormal, ChannelPair, FrequencyHz, MusicalTime, Normal, Ratio, Sample,
            SampleRate, StereoSample, Tempo, TimeRange, TimeSignature, TrackTitle, TrackUid,
            TrackUidFactory, Uid, UidFactory, ViewRange,
        };
    }
}

pub mod utils {
    //! Various helpers.
    pub use crate::core::utils::Paths;

    /// The most commonly used imports.
    pub mod prelude {
        pub use super::Paths;
    }
}

pub use all_entities::EnsnareEntities;
pub use version::app_version;

mod all_entities;
mod version;

pub mod automation;
pub mod composition;
pub mod core;
pub mod cores;
pub mod egui;
pub mod elements;
pub mod entities_future;
pub mod midi;
pub mod orchestration;
pub mod project;
pub mod services;
pub mod traits_future;
pub mod types_future;
pub mod uid;

pub mod time {
    pub use crate::core::time::*;
}

//pub use project::{ProjectAction, ProjectWidget};
// pub use track::{make_title_bar_galley, TitleBarWidget, TrackWidget};

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        automation::prelude::*, composition::prelude::*, core::prelude::*, egui::prelude::*,
        elements::prelude::*, entities::prelude::*, entities_future::prelude::*, midi::prelude::*,
        orchestration::prelude::*, project::prelude::*, services::prelude::*, traits::prelude::*,
        traits_future::prelude::*, transport::prelude::*, types::prelude::*, utils::prelude::*,
        EnsnareEntities,
    };
}
