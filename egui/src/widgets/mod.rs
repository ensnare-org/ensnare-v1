// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub mod prelude {
    pub use super::{audio, core, parts::*, pattern, placeholder, timeline, track};
}

/// Contains widgets that help visualize audio.
pub mod audio;

/// Various widgets used throughout the system.
pub mod core;

/// General-purpose widgets.
pub mod misc;

/// Constants, structs, and enums for widgets.
pub mod parts;

/// Contains widgets related to [Pattern](crate::mini::piano_roll::Pattern)s and
/// [PianoRoll](crate::mini::piano_roll::PianoRoll).
pub mod pattern;

/// Contains widgets that are useful as placeholders during development.
pub mod placeholder;

/// Contains widgets that help draw timeline views.
pub mod timeline;

/// Contains widgets that help draw tracks.
pub mod track;
