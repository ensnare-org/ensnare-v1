// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{
    cores::instruments::WelshSynthCore,
    egui::{
        BiQuadFilterLowPass24dbWidget, BiQuadFilterWidgetAction, DcaWidget, DcaWidgetAction,
        EnvelopeWidget, OscillatorWidget,
    },
    prelude::*,
};
use eframe::egui::{CollapsingHeader, Slider, Widget};
use strum_macros::Display;

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
                let mut action = None;
                if ui
                    .add(BiQuadFilterLowPass24dbWidget::widget(
                        &mut self.inner.filter,
                        &mut action,
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
                if let Some(action) = action {
                    match action {
                        BiQuadFilterWidgetAction::Link(source, param) => {
                            *self.action = Some(WelshWidgetAction::Link(source, param));
                        }
                    }
                }
            })
            .header_response;
        response
    }
}
