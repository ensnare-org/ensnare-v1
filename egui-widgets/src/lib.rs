// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui widgets that might be useful outside this project.

/// Widgets that display state and are noninteractive.
pub use indicators::{activity_indicator, level_indicator};

pub use misc::{oblique_strategies, ObliqueStrategiesManager};

mod indicators;
mod misc;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::indicators::{activity_indicator, level_indicator};
}
