// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::{effects::BiQuadFilterLowPass24dbWidget, modulators::DcaWidget};
use crate::{
    egui::unfiled::{EnvelopeWidget, OscillatorWidget},
    prelude::*,
};
use eframe::egui::{CollapsingHeader, Slider, Widget};

#[derive(Debug)]
pub struct SamplerWidget<'a> {
    inner: &'a mut crate::cores::instruments::Sampler,
}
impl<'a> SamplerWidget<'a> {
    fn new(inner: &'a mut crate::cores::instruments::Sampler) -> Self {
        Self { inner }
    }

    pub fn widget(
        inner: &'a mut crate::cores::instruments::Sampler,
    ) -> impl eframe::egui::Widget + '_ {
        move |ui: &mut eframe::egui::Ui| SamplerWidget::new(inner).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for SamplerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(format!("Filename: {:?}", self.inner.path()))
    }
}

#[derive(Debug)]
pub struct WelshWidget<'a> {
    uid: Uid,
    inner: &'a mut crate::cores::instruments::WelshSynth,
}
impl<'a> WelshWidget<'a> {
    fn new(uid: Uid, inner: &'a mut crate::cores::instruments::WelshSynth) -> Self {
        Self { uid, inner }
    }

    pub fn widget(
        uid: Uid,
        inner: &'a mut crate::cores::instruments::WelshSynth,
    ) -> impl eframe::egui::Widget + '_ {
        move |ui: &mut eframe::egui::Ui| WelshWidget::new(uid, inner).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for WelshWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut response = CollapsingHeader::new("Oscillator 1")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(OscillatorWidget::widget(&mut self.inner.oscillator_1))
                    .changed()
                {
                    self.inner.notify_change_oscillator_1();
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Oscillator 2")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(OscillatorWidget::widget(&mut self.inner.oscillator_2))
                    .changed()
                {
                    self.inner.notify_change_oscillator_2();
                }
            })
            .header_response;
        let mut oscillator_mix = self.inner.oscillator_mix.0;
        if ui
            .add(Slider::new(&mut oscillator_mix, 0.0..=1.0).text("Osc Blend"))
            .changed()
        {
            self.inner.set_oscillator_mix(oscillator_mix.into());
        }

        response |= CollapsingHeader::new("DCA")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(DcaWidget::widget(&mut self.inner.dca, self.uid))
                    .changed()
                {
                    self.inner.notify_change_dca();
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Amplitude")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(EnvelopeWidget::widget(&mut self.inner.amp_envelope))
                    .changed()
                {
                    self.inner.notify_change_amp_envelope();
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Low-Pass Filter")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show(ui, |ui| {
                if ui
                    .add(BiQuadFilterLowPass24dbWidget::widget(
                        &mut self.inner.filter,
                    ))
                    .changed()
                {
                    self.inner.notify_change_filter();
                }
                if ui
                    .add(EnvelopeWidget::widget(&mut self.inner.filter_envelope))
                    .changed()
                {
                    self.inner.notify_change_filter_envelope();
                }
            })
            .header_response;
        response
    }
}
