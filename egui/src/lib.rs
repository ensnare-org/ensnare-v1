// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui logic for drawing ensnare entities.

pub mod control;
pub mod controllers;
pub mod effects;
pub mod generators;
pub mod instruments;
pub mod piano_roll;
pub mod test;
pub mod toys;
pub mod transport;
pub mod widgets;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{
        controllers::{
            live_pattern_sequencer_widget, trip, Arpeggiator, LfoController,
            SignalPassthroughController, Timer, Trigger,
        },
        effects::{
            BiQuadFilterLowPass24db, Bitcrusher, Chorus, Compressor, Gain, Limiter, Mixer, Reverb,
        },
        instruments::{fm::FmSynth, Drumkit, Sampler, WelshSynth},
        transport::transport,
    };
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
