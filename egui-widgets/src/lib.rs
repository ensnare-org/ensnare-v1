// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui widgets that might be useful outside this project.

pub use audio::{
    analyze_spectrum, FrequencyDomainWidget, FrequencyWidget, TimeDomainWidget, WaveformWidget,
};
pub use control_bar::{ControlBar, ControlBarAction, ControlBarWidget};
pub use core::DragNormalWidget;
pub use generators::{EnvelopeWidget, OscillatorWidget};
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
    pub use super::{
        activity_indicator, fill_remaining_ui_space, level_indicator, DragNormalWidget,
        WaveformWidget,
    };
}
