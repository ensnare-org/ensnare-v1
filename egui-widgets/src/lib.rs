// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui widgets that might be useful outside this project.

pub use audio::{analyze_spectrum, frequency, frequency_domain, time_domain, waveform};
pub use control_bar::{ControlBar, ControlBarAction, ControlBarWidget};
pub use core::drag_normal;
pub use generators::{envelope, oscillator};
pub use indicators::{activity_indicator, level_indicator};
pub use misc::ObliqueStrategiesWidget;
pub use placeholder::wiggler;
pub use util::fill_remaining_ui_space;

mod audio;
mod control_bar;
mod core;
mod generators;
mod indicators;
mod misc;
mod placeholder;
mod util;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{activity_indicator, drag_normal, fill_remaining_ui_space, level_indicator};
}
