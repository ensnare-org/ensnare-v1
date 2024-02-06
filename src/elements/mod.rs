// Copyright (c) 2024 Mike Tsao. All rights reserved.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        generators::{Envelope, Oscillator, PathUid, PathUidFactory, SignalPath, Waveform},
        modulators::Dca,
        synthesizers::Synthesizer,
        voices::{StealingVoiceStore, VoiceCount, VoiceStore},
    };
}

pub use generators::{Envelope, Oscillator, PathUid, PathUidFactory, SignalPath, Waveform};
pub use modulators::Dca;
pub use synthesizers::Synthesizer;
pub use voices::{StealingVoiceStore, VoiceCount, VoicePerNoteStore, VoiceStore};

/// Building blocks for signal generation.
mod generators;

/// Building blocks for signal modulation.
mod modulators;

/// Scaffolding for building synthesizers.
mod synthesizers;

/// Scaffolding for managing multiple voices.
mod voices;
