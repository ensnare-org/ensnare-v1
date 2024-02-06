// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Creation and representation of music scores.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        sequencers::{PatternSequencer, PatternSequencerBuilder},
        ArrangementUid, Composer, Note, Pattern, PatternBuilder, PatternUid, PatternUidFactory,
    };
}

pub use arrangement::{ArrangementUid, ArrangementUidFactory};
pub use composer::Composer;
pub use note::Note;
pub use pattern::{Pattern, PatternBuilder, PatternUid, PatternUidFactory};
pub use sequencers::{
    MidiSequencer, MidiSequencerBuilder, NoteSequencer, NoteSequencerBuilder, PatternSequencer,
    PatternSequencerBuilder,
};

mod arrangement;
mod composer;
mod note;
mod pattern;
mod sequencers;
