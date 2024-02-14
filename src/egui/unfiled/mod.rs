// Copyright (c) 2024 Mike Tsao. All rights reserved.

pub use control_bar::{ControlBar, ControlBarAction, ControlBarWidget};
pub use core::DragNormalWidget;
pub use generators::{EnvelopeWidget, OscillatorWidget};
pub use indicators::{activity_indicator, level_indicator};
pub use misc::ObliqueStrategiesWidget;
pub use placeholder::wiggler;
pub use unfiled_xx::{CarouselAction, CarouselWidget, DraggableIconWidget, IconWidget};
pub use util::fill_remaining_ui_space;

mod control_bar;
mod core;
mod generators;
mod indicators;
mod misc;
mod placeholder;
mod unfiled_xx;
mod util;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        activity_indicator, fill_remaining_ui_space, level_indicator, DragNormalWidget,
    };
}
