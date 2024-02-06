// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Toy Entities that wrap the corresponding toy devices in ensnare_core::toys.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::controllers::*;
    pub use super::effects::*;
    pub use super::instruments::*;
    pub use super::ToyEntities;
}

pub use crate::controllers::*;
pub use crate::effects::*;
pub use crate::instruments::*;
pub use factory::ToyEntities;

mod controllers;
mod cores;
mod effects;
mod factory;
mod instruments;
