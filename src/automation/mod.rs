// Copyright (c) 2024 Mike Tsao. All rights reserved.

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

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        ControlEventsFn, ControlIndex, ControlLink, ControlName, ControlProxyEventsFn, ControlStep,
        ControlStepBuilder, ControlTrip, ControlTripBuilder, ControlTripPath, ControlValue,
        Controllable, Controls, ControlsAsProxy,
    };
}

pub use automator::Automator;
pub use controllers::{
    ControlStep, ControlStepBuilder, ControlTrip, ControlTripBuilder, ControlTripPath, Timer,
    Trigger,
};
pub use traits::{ControlEventsFn, ControlProxyEventsFn, Controllable, Controls, ControlsAsProxy};
pub use types::{ControlIndex, ControlLink, ControlName, ControlValue};

mod automator;
mod controllers;
mod traits;
mod types;
