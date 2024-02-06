// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Built-in musical instruments and supporting infrastructure.

/// The most commonly used imports.
pub mod prelude {
    #[cfg(feature = "test")]
    pub use super::register_test_entities;
    pub use super::{
        infra::{EntityFactory, EntityKey, EntityUidFactory},
        BuiltInEntities,
    };
}

pub use built_in::*;
#[cfg(feature = "test")]
pub use test_entities::*;

mod built_in;
mod infra;
mod test_entities;
