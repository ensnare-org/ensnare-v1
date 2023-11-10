// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{effects::bi_quad_filter_low_pass_24db, modulators::dca};
use eframe::egui::{CollapsingHeader, Widget};
use ensnare_core::prelude::*;
use ensnare_egui_widgets::{envelope, oscillator};

pub mod fm;

/// Wraps a [SamplerWidget] as a [Widget](eframe::egui::Widget).
pub fn sampler<'a>(inner: &'a mut ensnare_cores::Sampler) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| SamplerWidget::new(inner).ui(ui)
}

#[derive(Debug)]
pub struct SamplerWidget<'a> {
    inner: &'a mut ensnare_cores::Sampler,
}
impl<'a> SamplerWidget<'a> {
    fn new(inner: &'a mut ensnare_cores::Sampler) -> Self {
        Self { inner }
    }
}
impl<'a> Widget for SamplerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(sampler(self.inner))
    }
}

/// Wraps a [WelshWidget] as a [Widget](eframe::egui::Widget).
pub fn welsh<'a>(
    uid: Uid,
    inner: &'a mut ensnare_cores::WelshSynth,
) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| WelshWidget::new(uid, inner).ui(ui)
}

#[derive(Debug)]
pub struct WelshWidget<'a> {
    uid: Uid,
    inner: &'a mut ensnare_cores::WelshSynth,
}
impl<'a> WelshWidget<'a> {
    fn new(uid: Uid, inner: &'a mut ensnare_cores::WelshSynth) -> Self {
        Self { uid, inner }
    }
}
impl<'a> Widget for WelshWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
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
    }
}
