// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! egui widgets for system components.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{ComposerWidget, EntityPaletteWidget, ProjectAction, ProjectWidget};
}

pub use audio::AudioSettingsWidget;
pub use composer::{ComposerEditorWidget, ComposerWidget};
pub use cursor::CursorWidget;
pub use entities::EntityPaletteWidget;
pub use grid::GridWidget;
pub use legend::LegendWidget;
pub use midi::MidiSettingsWidget;
pub use project::{ProjectAction, ProjectWidget};
pub use signal_chain::SignalChainItem;
pub use timeline::{TimelineIconStripAction, TimelineIconStripWidget};
pub use track::{make_title_bar_galley, TitleBarWidget, TrackWidget};

mod audio;
mod composer;
mod cursor;
mod entities;
mod grid;
mod legend;
mod midi;
mod project;
mod signal_chain;
mod timeline;
mod track;
