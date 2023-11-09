// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Ensnare structs that implement the Entity trait.

mod controllers;
mod factory;

pub use factory::register_factory_entities;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::factory::register_factory_entities;
}
