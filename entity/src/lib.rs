// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub mod factory;
pub mod test_entities;
pub mod traits;

pub mod prelude {
    pub use super::factory::{EntityFactory, EntityKey, EntityStore};
    pub use super::traits::{
        Acts, Displays, Entity, IsAction, IsController, IsEffect, IsInstrument,
    };
}
