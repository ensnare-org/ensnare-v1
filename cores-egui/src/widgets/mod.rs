// Copyright (c) 2023 Mike Tsao. All rights reserved.

/// Constants, structs, and enums for widgets.
//pub mod parts;

/// Contains widgets related to [Pattern](crate::mini::piano_roll::Pattern)s and
/// [PianoRoll](crate::mini::piano_roll::PianoRoll).
pub mod pattern;

pub mod prelude {
    pub use super::pattern;
}
