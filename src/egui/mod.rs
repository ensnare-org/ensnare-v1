// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Widgets that work with the [egui](https://www.egui.rs/) GUI library.

// Public/reusable
pub use {
    chrome::{ControlBar, ControlBarAction, ControlBarWidget, TransportWidget},
    composition::ComposerWidget,
    entities::EntityPaletteWidget,
    generators::{EnvelopeWidget, OscillatorWidget},
    glue::DragNormalWidget,
    misc::ObliqueStrategiesWidget,
    modulators::{DcaWidget, DcaWidgetAction},
    project::{ProjectAction, ProjectWidget},
    settings::{AudioSettingsWidget, MidiSettingsWidget},
};

/// Exported only for widget explorer example.
// TODO maybe replace with a sneaky factory
pub mod widget_explorer {
    pub use super::{
        audio::{analyze_spectrum, FrequencyDomainWidget, TimeDomainWidget},
        automation::{SignalPathWidget, Target, TargetNode},
        controllers::{ArpeggiatorWidget, LfoControllerWidget, NoteSequencerWidget},
        grid::GridWidget,
        legend::LegendWidget,
        placeholders::Wiggler,
        track::{make_title_bar_galley, TitleBarWidget},
    };
}

// Internal use only
pub(crate) use {
    controllers::{ArpeggiatorWidget, LfoControllerWidget},
    effects::{
        BiQuadFilterAllPassWidget, BiQuadFilterBandPassWidget, BiQuadFilterBandStopWidget,
        BiQuadFilterHighPassWidget, BiQuadFilterLowPass24dbWidget, BiQuadFilterWidgetAction,
    },
    instruments::{
        DrumkitWidget, DrumkitWidgetAction, FmSynthWidget, FmSynthWidgetAction, SamplerWidget,
        SamplerWidgetAction, SubtractiveSynthWidget, SubtractiveSynthWidgetAction,
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
mod chrome;
mod colors;
mod composition;
mod controllers;
mod cursor;
mod effects;
mod entities;
mod generators;
mod glue;
mod grid;
mod indicators;
mod instruments;
mod legend;
mod midi;
mod misc;
mod modulators;
mod placeholders;
mod project;
mod settings;
mod signal_chain;
mod track;
mod util;
