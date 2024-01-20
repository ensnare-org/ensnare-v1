// Copyright (c) 2024 Mike Tsao. All rights reserved.

pub mod composer;

pub use composer::Composer;

pub mod prelude {
    pub use super::{
        Arrangement, Composer, Note, Pattern, PatternBuilder, PatternUid, PatternUidFactory,
    };
}
