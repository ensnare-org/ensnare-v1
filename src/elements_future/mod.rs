// Copyright (c) 2024 Mike Tsao. All rights reserved.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        generators::{Envelope, Oscillator, PathUid, PathUidFactory, SignalPath, Waveform},
        modulators::Dca,
    };
}

pub use generators::{Envelope, Oscillator, PathUid, PathUidFactory, SignalPath, Waveform};
pub use modulators::Dca;

/// Building blocks for signal generation.
mod generators;

/// Building blocks for signal modulation.
mod modulators;
