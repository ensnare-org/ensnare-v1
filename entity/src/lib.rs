// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub mod factory;
pub mod traits;

pub mod prelude {
    pub use super::factory::{EntityFactory, EntityKey, EntityStore};
    pub use super::traits::{
        Displays, Entity2, EntityBounds, IsController, IsEffect, IsInstrument,
    };
}
