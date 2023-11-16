// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui widgets that might be useful outside this project.

pub use audio::{frequency, waveform};
pub use core::drag_normal;
pub use generators::{envelope, oscillator};
pub use indicators::{activity_indicator, level_indicator};
pub use misc::{oblique_strategies, ObliqueStrategiesManager};
pub use types::ViewRange;

mod audio;
mod core;
mod generators;
mod indicators;
mod misc;
mod types;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::core::drag_normal;
    pub use super::indicators::{activity_indicator, level_indicator};
}
