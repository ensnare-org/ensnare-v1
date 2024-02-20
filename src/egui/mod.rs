// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Widgets that work with the [egui](https://www.egui.rs/) GUI library.

// Public/reusable
pub use {
    composition::ComposerWidget,
    entities::EntityPaletteWidget,
    generators::{EnvelopeWidget, OscillatorWidget},
    modulators::{DcaWidget, DcaWidgetAction},
    project::{ProjectAction, ProjectWidget},
    settings::{AudioSettingsWidget, MidiSettingsWidget},
    timeline::{TimelineIconStripAction, TimelineIconStripWidget},
    transport::TransportWidget,
    unfiled::{
        ControlBar, ControlBarAction, ControlBarWidget, DragNormalWidget, ObliqueStrategiesWidget,
    },
};

/// Exported only for widget explorer.
// TODO maybe replace with a sneaky factory
pub mod widget_explorer {
    pub use super::{
        audio::{analyze_spectrum, FrequencyDomainWidget, TimeDomainWidget},
        controllers::{ArpeggiatorWidget, LfoControllerWidget, NoteSequencerWidget},
        grid::GridWidget,
        legend::LegendWidget,
        track::{make_title_bar_galley, TitleBarWidget},
        unfiled::wiggler,
    };
}

// Internal use only
pub(crate) use {
    controllers::{ArpeggiatorWidget, LfoControllerWidget},
    effects::{
        BiQuadFilterAllPassWidget, BiQuadFilterBandPassWidget, BiQuadFilterBandStopWidget,
        BiQuadFilterHighPassWidget, BiQuadFilterLowPass24dbWidget, BiQuadFilterWidgetAction,
    },
    fm::{FmSynthWidget, FmSynthWidgetAction},
    instruments::{
        DrumkitWidget, DrumkitWidgetAction, SamplerWidget, SamplerWidgetAction, WelshWidget,
        WelshWidgetAction,
    },
};

// Used only by other widgets
pub(in crate::egui) use {
    audio::{
        analyze_spectrum, FrequencyDomainWidget, FrequencyWidget, TimeDomainWidget, WaveformWidget,
    },
    grid::GridWidget,
    indicators::activity_indicator,
    legend::LegendWidget,
    util::{dnd_drop_zone_with_inner_response, fill_remaining_ui_space},
};

mod audio;
mod automation;
mod colors;
mod composition;
mod controllers;
mod cursor;
mod effects;
mod entities;
mod fm;
mod generators;
mod grid;
mod indicators;
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
mod util;
