// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::cores::{ToyInstrumentCore, ToySynthCore};
use ensnare::{
    egui::{DcaWidget, DcaWidgetAction, EnvelopeWidget, OscillatorWidget},
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct ToyInstrument {
    uid: Uid,
    inner: ToyInstrumentCore,

    #[serde(skip)]
    dca_widget_action: Option<DcaWidgetAction>,
}
impl Displays for ToyInstrument {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(OscillatorWidget::widget(&mut self.inner.oscillator))
            | ui.add(DcaWidget::widget(
                &mut self.inner.dca,
                &mut self.dca_widget_action,
            ))
    }
}
impl ToyInstrument {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: ToyInstrumentCore::new(),
            dca_widget_action: Default::default(),
        }
    }
}

#[derive(
    Debug,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct ToySynth {
    uid: Uid,
    inner: ToySynthCore,
}
impl Default for ToySynth {
    fn default() -> Self {
        Self {
            uid: Default::default(),
            inner: ToySynthCore::new_with(
                Oscillator::default(),
                Envelope::default(),
                Dca::default(),
            ),
        }
    }
}
impl Displays for ToySynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            let oscillator_response = self.ui_oscillator(ui);
            let envelope_response = self.ui_envelope(ui);
            let dca_response = self.ui_dca(ui);
            oscillator_response | envelope_response | dca_response
        })
        .inner
    }
}
impl ToySynth {
    fn ui_oscillator(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(OscillatorWidget::widget(&mut self.inner.oscillator));
        if response.changed() {
            self.inner.notify_change_oscillator();
        }
        response
    }

    fn ui_envelope(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(EnvelopeWidget::widget(&mut self.inner.envelope));
        if response.changed() {
            self.inner.notify_change_envelope();
        }
        response
    }

    fn ui_dca(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut action = None;
        let response = ui.add(DcaWidget::widget(&mut self.inner.dca, &mut action));
        if response.changed() {
            self.inner.notify_change_dca();
        }
        response
    }

    pub fn new_with(uid: Uid, oscillator: Oscillator, envelope: Envelope, dca: Dca) -> Self {
        Self {
            uid,
            inner: ToySynthCore::new_with(oscillator, envelope, dca),
        }
    }
}
