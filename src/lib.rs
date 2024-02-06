// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! The `ensnare` crate helps make digital music.

pub use all_entities::EnsnareEntities;
pub use automation::Automator;
pub use composition::Composer;
pub use orchestration::Orchestrator;
pub use project::Project;
pub use version::app_version;

pub mod automation;
pub mod composition;
pub mod cores;
#[cfg(feature = "egui")]
pub mod egui;
pub mod elements;
pub mod entities;
pub mod midi;
pub mod orchestration;
pub mod project;
pub mod services;
pub mod traits;
pub mod types;
pub mod util;

mod all_entities;
mod version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    #[cfg(feature = "egui")]
    pub use super::egui::prelude::*;
    pub use super::{
        automation::prelude::*, composition::prelude::*, elements::prelude::*,
        entities::prelude::*, midi::prelude::*, orchestration::prelude::*, project::prelude::*,
        services::prelude::*, traits::prelude::*, types::prelude::*, util::prelude::*,
        EnsnareEntities,
    };
}
