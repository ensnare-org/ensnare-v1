// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui widgets that might be useful outside this project.

pub use audio::{analyze_spectrum, frequency, frequency_domain, time_domain, waveform};
pub use control_bar::{control_bar_widget, ControlBar, ControlBarAction};
pub use core::drag_normal;
pub use generators::{envelope, oscillator};
pub use indicators::{activity_indicator, level_indicator};
pub use misc::{oblique_strategies, ObliqueStrategiesManager};
pub use placeholder::wiggler;

mod audio;
mod control_bar;
mod core;
mod generators;
mod indicators;
mod misc;
mod placeholder;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::core::drag_normal;
    pub use super::indicators::{activity_indicator, level_indicator};
}
