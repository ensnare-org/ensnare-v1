// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! The `ensnare` crate helps make digital music.

pub use all_entities::EnsnareEntities;
pub use automation::Automator;
pub use composition::Composer;
pub use orchestration::Orchestrator;
pub use version::app_version;

pub mod automation;
pub mod composition;
pub mod core;
pub mod cores;
pub mod egui;
pub mod elements;
pub mod entities;
pub mod midi;
pub mod orchestration;
pub mod project;
pub mod selection_set;
pub mod services;
pub mod time;
pub mod traits;
pub mod types;
pub mod uid;
pub mod utils;

mod all_entities;
mod version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        automation::prelude::*, composition::prelude::*, core::prelude::*, egui::prelude::*,
        elements::prelude::*, entities::prelude::*, midi::prelude::*, orchestration::prelude::*,
        project::prelude::*, services::prelude::*, time::prelude::*, traits::prelude::*,
        types::prelude::*, utils::prelude::*, EnsnareEntities,
    };
}
