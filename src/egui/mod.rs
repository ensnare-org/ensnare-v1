// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Widgets that work with the [egui](https://www.egui.rs/) GUI library.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        ComposerWidget, EntityPaletteWidget, ProjectAction, ProjectWidget, WaveformWidget,
    };
}

pub use audio::{
    analyze_spectrum, FrequencyDomainWidget, FrequencyWidget, TimeDomainWidget, WaveformWidget,
};
pub use automation::{ControlLinkSource, ControlTripWidget};
pub use composition::ComposerWidget;
pub use controllers::{
    ArpeggiatorWidget, LfoControllerWidget, NoteSequencerWidget, PatternSequencerWidget, TripWidget,
};
pub use cursor::CursorWidget;
pub use effects::{
    BiQuadFilterAllPassWidget, BiQuadFilterBandPassWidget, BiQuadFilterBandStopWidget,
    BiQuadFilterHighPassWidget, BiQuadFilterLowPass24dbWidget,
};
pub use entities::EntityPaletteWidget;
pub use fm::{FmSynthWidget, FmSynthWidgetAction};
pub use grid::GridWidget;
pub use instruments::{
    DrumkitWidget, DrumkitWidgetAction, SamplerWidget, SamplerWidgetAction, WelshWidget,
    WelshWidgetAction,
};
pub use legend::LegendWidget;
pub use modulators::{DcaWidget, DcaWidgetAction};
pub use project::{ProjectAction, ProjectWidget};
pub use settings::{AudioSettingsWidget, MidiSettingsWidget};
pub use signal_chain::SignalChainItem;
pub use timeline::{TimelineIconStripAction, TimelineIconStripWidget};
pub use track::{make_title_bar_galley, TitleBarWidget, TrackWidget};
pub use transport::TransportWidget;
pub use unfiled::*;

mod audio;
mod automation;
mod colors;
mod composition;
mod controllers;
mod cursor;
mod effects;
mod entities;
mod fm;
mod grid;
mod instruments;
mod legend;
mod midi;
mod modulators;
mod orchestration;
mod project;
mod settings;
mod signal_chain;
mod timeline;
mod track;
mod transport;
mod unfiled;
