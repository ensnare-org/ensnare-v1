// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Creation and representation of music scores.

/// The most commonly used imports.
pub mod prelude {
    pub use super::Composer;
    pub use crate::core::composition::{Note, PatternBuilder, PatternUid};
}

pub use composer::Composer;

mod composer;
