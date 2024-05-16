// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Useful things that don't have anything to do with digital audio.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        init_sample_libraries, BoundedCrossbeamChannel, CrossbeamChannel, KitIndex, KitItem,
        KitLibrary, ModSerial, Paths, SampleIndex, SampleLibrary, SampleSource,
    };
}

pub use channels::{BoundedCrossbeamChannel, CrossbeamChannel};
pub use rng::Rng;
pub use selection_set::SelectionSet;

mod channels;
pub mod rng;
pub mod selection_set;

#[cfg(feature = "std")]
pub use mod_serial::ModSerial;
#[cfg(feature = "std")]
mod mod_serial;

#[cfg(feature = "std")]
pub use paths::{FileType, Paths};
#[cfg(feature = "std")]
pub mod paths;

pub use library::{
    init_sample_libraries, KitIndex, KitItem, KitLibrary, SampleIndex, SampleLibrary, SampleSource,
};
pub mod library;
