// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! This module assembles all the available entities so that an application can
//! use them.

use crate::prelude::*;

/// A wrapper that contains all the entities we know about.
pub struct MiniDawEntities {}
impl MiniDawEntities {
    /// Registers all the entities in this collection.
    pub fn register(
        mut factory: EntityFactory<dyn EntityBounds>,
    ) -> EntityFactory<dyn EntityBounds> {
        let factory = BuiltInEntities::register(factory);
        factory
    }
}
