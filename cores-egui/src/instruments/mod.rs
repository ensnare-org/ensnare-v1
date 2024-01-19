// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{effects::bi_quad_filter_low_pass_24db, modulators::dca};
use eframe::egui::{CollapsingHeader, Slider, Widget};
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
impl<'a> eframe::egui::Widget for SamplerWidget<'a> {
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
impl<'a> eframe::egui::Widget for WelshWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut response = CollapsingHeader::new("Oscillator 1")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui.add(oscillator(&mut self.inner.oscillator_1)).changed() {
                    self.inner.notify_change_oscillator_1();
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Oscillator 2")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui.add(oscillator(&mut self.inner.oscillator_2)).changed() {
                    self.inner.notify_change_oscillator_2();
                }
            })
            .header_response;
        let mut oscillator_mix = self.inner.oscillator_mix.0;
        if ui
            .add(Slider::new(&mut oscillator_mix, 0.0..=1.0))
            .changed()
        {
            self.inner.set_oscillator_mix(oscillator_mix.into());
        }

        response |= CollapsingHeader::new("DCA")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui.add(dca(&mut self.inner.dca, self.uid)).changed() {
                    self.inner.notify_change_dca();
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Amplitude")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui.add(envelope(&mut self.inner.amp_envelope)).changed() {
                    self.inner.notify_change_amp_envelope();
                }
            })
            .header_response;
        response |= CollapsingHeader::new("LPF")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(bi_quad_filter_low_pass_24db(&mut self.inner.filter))
                    .changed()
                {
                    self.inner.notify_change_filter();
                }
                if ui.add(envelope(&mut self.inner.filter_envelope)).changed() {
                    self.inner.notify_change_filter_envelope();
                }
            })
            .header_response;
        response
    }
}
