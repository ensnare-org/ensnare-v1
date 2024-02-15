// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::{
    effects::BiQuadFilterLowPass24dbWidget,
    modulators::{DcaWidget, DcaWidgetAction},
};
use crate::{
    cores::instruments::{DrumkitCore, SamplerCore, WelshSynthCore},
    egui::unfiled::{EnvelopeWidget, OscillatorWidget},
    prelude::*,
};
use eframe::egui::{CollapsingHeader, Slider, Widget};
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum SamplerWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

#[derive(Debug)]
pub struct SamplerWidget<'a> {
    inner: &'a mut SamplerCore,
    action: &'a mut Option<SamplerWidgetAction>,
}
impl<'a> SamplerWidget<'a> {
    fn new(inner: &'a mut SamplerCore, action: &'a mut Option<SamplerWidgetAction>) -> Self {
        Self { inner, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        inner: &'a mut SamplerCore,
        action: &'a mut Option<SamplerWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| SamplerWidget::new(inner, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for SamplerWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(format!("Filename: {:?}", self.inner.path()))
    }
}

#[derive(Debug, Display)]
pub enum DrumkitWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

#[derive(Debug)]
pub struct DrumkitWidget<'a> {
    inner: &'a mut DrumkitCore,
    action: &'a mut Option<DrumkitWidgetAction>,
}
impl<'a> DrumkitWidget<'a> {
    fn new(inner: &'a mut DrumkitCore, action: &'a mut Option<DrumkitWidgetAction>) -> Self {
        Self { inner, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        inner: &'a mut DrumkitCore,
        action: &'a mut Option<DrumkitWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| DrumkitWidget::new(inner, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for DrumkitWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(format!("Name: {:?}", self.inner.name()))
    }
}

#[derive(Debug, Display)]
pub enum WelshWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

#[derive(Debug)]
pub struct WelshWidget<'a> {
    inner: &'a mut WelshSynthCore,
    action: &'a mut Option<WelshWidgetAction>,
}
impl<'a> WelshWidget<'a> {
    fn new(inner: &'a mut WelshSynthCore, action: &'a mut Option<WelshWidgetAction>) -> Self {
        Self { inner, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        inner: &'a mut WelshSynthCore,
        action: &'a mut Option<WelshWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| WelshWidget::new(inner, action).ui(ui)
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
                let mut action = None;
                if ui
                    .add(DcaWidget::widget(&mut self.inner.dca, &mut action))
                    .changed()
                {
                    self.inner.notify_change_dca();
                }
                if let Some(action) = action {
                    match action {
                        DcaWidgetAction::Link(source, index) => {
                            *self.action = Some(WelshWidgetAction::Link(
                                source,
                                index + WelshSynthCore::DCA_INDEX,
                            ))
                        }
                    }
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
