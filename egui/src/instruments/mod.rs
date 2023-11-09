// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::CollapsingHeader;
use ensnare_core::{
    entities::prelude::{DrumkitParams, SamplerParams},
    prelude::*,
    stuff::welsh::WelshSynthParams,
    utils::Paths,
};
use ensnare_egui_widgets::{dca, envelope, oscillator};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsInstrument, Metadata,
};

use crate::effects::bi_quad_filter_low_pass_24db;

pub mod fm;

#[derive(
    Debug,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
pub struct WelshSynth {
    uid: Uid,
    inner: ensnare_core::stuff::welsh::WelshSynth,
}
impl Displays for WelshSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // TODO: the set_waveform() calls don't capture the whole set of things
        // that the oscillator widget might change. We need to figure out how to
        // update the live oscillator parameters without doing things like
        // resetting the period.
        let mut response = CollapsingHeader::new("Oscillator 1")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(oscillator(&mut self.inner.voice.oscillator_1))
                    .changed()
                {
                    self.inner.inner_synth.voices_mut().for_each(|v| {
                        v.oscillator_1
                            .set_waveform(self.inner.voice.oscillator_1.waveform())
                    })
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Oscillator 2")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(oscillator(&mut self.inner.voice.oscillator_2))
                    .changed()
                {
                    self.inner.inner_synth.voices_mut().for_each(|v| {
                        v.oscillator_2
                            .set_waveform(self.inner.voice.oscillator_2.waveform())
                    })
                }
            })
            .header_response;

        // TODO: this doesn't get propagated to the voices, because the
        // single DCA will be responsible for turning mono voice output to
        // stereo.
        //
        // TODO: hmmm but it sure looks like we are propagating....
        response |= CollapsingHeader::new("DCA")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui.add(dca(&mut self.inner.dca, self.uid)).changed() {
                    self.inner.inner_synth.voices_mut().for_each(|v| {
                        v.dca.update_from_params(&self.inner.dca.to_params());
                    })
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Amplitude")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(envelope(&mut self.inner.voice.amp_envelope))
                    .changed()
                {
                    self.inner.inner_synth.voices_mut().for_each(|v| {
                        v.amp_envelope_mut()
                            .update_from_params(&self.inner.voice.amp_envelope.to_params());
                    })
                }
            })
            .header_response;
        response |= CollapsingHeader::new("LPF")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                let filter_changed = ui
                    .add(bi_quad_filter_low_pass_24db(&mut self.inner.voice.filter))
                    .changed();
                let filter_envelope_changed = ui
                    .add(envelope(&mut self.inner.voice.filter_envelope))
                    .changed();
                if filter_changed || filter_envelope_changed {
                    self.inner.inner_synth.voices_mut().for_each(|v| {
                        v.filter_mut()
                            .update_from_params(&self.inner.voice.filter.to_params());
                    })
                }
            })
            .header_response;
        response
        //   }
    }
}
impl WelshSynth {
    pub fn new_with(uid: Uid, params: &WelshSynthParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::welsh::WelshSynth::new_with(params),
        }
    }
}

#[derive(
    Debug,
    InnerControllable,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
pub struct Sampler {
    uid: Uid,
    inner: ensnare_core::stuff::sampler::Sampler,
}
impl Displays for Sampler {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Sampler {
    pub fn new_with(uid: Uid, params: &SamplerParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::sampler::Sampler::new_with(&params),
        }
    }

    pub fn load(&mut self, paths: &Paths) -> anyhow::Result<()> {
        self.inner.load(paths)
    }
}

#[derive(
    Debug,
    InnerControllable,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
pub struct Drumkit {
    uid: Uid,
    inner: ensnare_core::stuff::drumkit::Drumkit,
}
impl Displays for Drumkit {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Drumkit {
    pub fn new_with(uid: Uid, params: &DrumkitParams, paths: &Paths) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::drumkit::Drumkit::new_with(&params, paths),
        }
    }
}
