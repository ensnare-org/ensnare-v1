// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Toy Entities that wrap the corresponding toy devices in ensnare_core::toys.

pub mod controllers;
pub mod effects;
pub mod factory;
pub mod instruments;

pub use factory::ToyEntities;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::controllers::*;
    pub use super::effects::*;
    pub use super::instruments::*;
    pub use super::ToyEntities;
}
