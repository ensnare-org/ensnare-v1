// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub mod prelude {
    pub use super::factory::{EntityFactory, EntityKey, EntityStore};
    pub use super::traits::{ControlProxyEventsFn, ControlsAsProxy, Displays, EntityBounds};
    pub use super::{EntityUidFactory, Uid};
}

pub use uid::{EntityUidFactory, Uid};

pub mod factory;
pub mod traits;
mod uid;
