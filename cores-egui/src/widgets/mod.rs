// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub mod prelude {
    pub use super::{audio, parts::*, pattern, placeholder, timeline};
}

/// Contains widgets that help visualize audio.
pub mod audio;

/// Constants, structs, and enums for widgets.
pub mod parts;

/// Contains widgets related to [Pattern](crate::mini::piano_roll::Pattern)s and
/// [PianoRoll](crate::mini::piano_roll::PianoRoll).
pub mod pattern;

/// Contains widgets that are useful as placeholders during development.
pub mod placeholder;

/// Contains widgets that help draw timeline views.
pub mod timeline;