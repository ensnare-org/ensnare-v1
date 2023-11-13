// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub mod controllers;
pub mod effects;
mod factory;
pub mod instruments;

pub use factory::register_test_entities;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::register_test_entities;
}
