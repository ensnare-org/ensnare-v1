// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! egui widgets for system components.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{ComposerWidget, EntityPaletteWidget, ProjectAction, ProjectWidget};
}

pub use audio::AudioSettingsWidget;
pub use composer::{ComposerEditorWidget, ComposerWidget};
pub use controllers::{
    ArpeggiatorWidget, LfoControllerWidget, NoteSequencerWidget, PatternSequencerWidget, TripWidget,
};
pub use cursor::CursorWidget;
pub use drag_drop::{DragDropManager, DragSource, DropTarget};
pub use effects::{
    BiQuadFilterAllPassWidget, BiQuadFilterBandPassWidget, BiQuadFilterBandStopWidget,
    BiQuadFilterHighPassWidget, BiQuadFilterLowPass24dbWidget,
};
pub use entities::EntityPaletteWidget;
pub use fm::FmSynthWidget;
pub use grid::GridWidget;
pub use instruments::{SamplerWidget, WelshWidget};
pub use legend::LegendWidget;
pub use midi::MidiSettingsWidget;
pub use modulators::DcaWidget;
pub use project::{ProjectAction, ProjectWidget};
pub use signal_chain::SignalChainItem;
pub use timeline::{TimelineIconStripAction, TimelineIconStripWidget};
pub use track::{make_title_bar_galley, TitleBarWidget, TrackWidget};
pub use transport::TransportWidget;
pub use unfiled::*;

mod audio;
mod colors;
mod composer;
mod controllers;
mod cursor;
mod drag_drop;
mod effects;
mod entities;
mod fm;
mod grid;
mod instruments;
mod legend;
mod midi;
mod modulators;
mod project;
mod signal_chain;
mod timeline;
mod track;
mod transport;
mod unfiled;
