// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Controllers are musical devices that emit control events rather than audio.
//! A good example is an arpeggiator, which produces MIDI messages.
//!
//! Controllers implement the [Controls](crate::traits::Controls) trait.
pub use arpeggiator::*;
pub use lfo::{LfoControllerCore, LfoControllerCoreBuilder};
pub use passthrough::*;

mod arpeggiator;
mod lfo;
mod passthrough;
mod util;
