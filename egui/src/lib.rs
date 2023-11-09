// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui logic for drawing ensnare entities.

pub mod controllers;
pub mod drag_drop;
pub mod effects;
pub mod generators;
pub mod instruments;
pub mod modulators;
pub mod orchestration;
pub mod toys;
pub mod track;
pub mod transport;
pub mod widgets;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        controllers::{
            live_pattern_sequencer_widget, trip, Arpeggiator, LfoController,
            SignalPassthroughController,
        },
        drag_drop::{DragDropManager, DragSource, DropTarget},
        effects::{
            BiQuadFilterLowPass24db, Bitcrusher, Chorus, Compressor, Gain, Limiter, Mixer, Reverb,
        },
        instruments::{fm::FmSynth, Drumkit, Sampler, WelshSynth},
        orchestration::{old_orchestrator, orchestrator},
        toys::{
            ToyController, ToyControllerAlwaysSendsMidiMessage, ToyEffect, ToyInstrument, ToySynth,
        },
        track::{signal_chain, track_widget, SignalChainWidgetAction},
        transport::transport,
    };
}

use eframe::epaint::vec2;
use ensnare_core::{traits::Displays, uid::Uid};
use widgets::pattern::{self, grid};

pub struct PianoRoll {
    uid: Uid,
    inner: ensnare_core::piano_roll::PianoRoll,
}
impl Displays for PianoRoll {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(piano_roll(self))
    }
}

/// Wraps a [PianoRollWidget] as a [Widget](eframe::egui::Widget).
pub fn piano_roll<'a>(entity: &'a mut PianoRoll) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| PianoRollWidget::new(entity).ui(ui)
}

struct PianoRollWidget<'a> {
    entity: &'a mut PianoRoll,
}
impl<'a> PianoRollWidget<'a> {
    pub fn new(entity: &'a mut PianoRoll) -> Self {
        Self { entity }
    }

    fn ui_pattern_edit(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if let Some(pattern_uid) = self.entity.inner.pattern_selection_set.single_selection() {
            ui.set_min_height(192.0);
            if let Some(pattern) = self.entity.inner.uids_to_patterns.get_mut(pattern_uid) {
                let desired_size = vec2(ui.available_width(), 96.0);
                let (_id, rect) = ui.allocate_space(desired_size);
                ui.add_enabled_ui(false, |ui| {
                    ui.allocate_ui_at_rect(rect, |ui| ui.add(grid(pattern.duration)))
                        .inner
                });
                return ui.allocate_ui_at_rect(rect, |ui| pattern.ui(ui)).inner;
            }
        }

        ui.set_min_height(0.0);
        // This is here so that we can return a Response. I don't know of a
        // better way to do it.
        ui.add_visible_ui(false, |_| {}).response
    }
}
impl<'a> Displays for PianoRollWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            ui.add(pattern::carousel(
                &self.entity.inner.ordered_pattern_uids,
                &self.entity.inner.uids_to_patterns,
                &mut self.entity.inner.pattern_selection_set,
            ));
            self.ui_pattern_edit(ui);
        })
        .response
    }
}

pub struct Track {}
impl Track {
    /// The [TitleBar] widget needs a Galley so that it can display the title
    /// sideways. But widgets live for only a frame, so it can't cache anything.
    /// Caller to the rescue! We generate the Galley and save it.
    ///
    /// TODO: when we allow title editing, we should set the galley to None so
    /// it can be rebuilt on the next frame.
    pub fn update_font_galley(&mut self, _ui: &mut eframe::egui::Ui) {
        // if self.e.title_font_galley.is_none() && !self.title.0.is_empty() {
        //     self.e.title_font_galley = Some(make_title_bar_galley(ui, &self.title));
        // }
        todo!()
    }
}
