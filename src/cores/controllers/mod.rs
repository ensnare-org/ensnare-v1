// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Controllers are musical devices that emit control events rather than audio.
//! A good example is an arpeggiator, which produces MIDI messages.

pub use arpeggiator::{ArpeggiatorCore, ArpeggiatorCoreBuilder, ArpeggioMode};
pub use lfo::{LfoControllerCore, LfoControllerCoreBuilder};
pub use passthrough::{SignalPassthroughControllerCore, SignalPassthroughType};

mod arpeggiator;
mod lfo;
mod passthrough;
mod util;
