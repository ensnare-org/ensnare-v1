// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    cores::controllers,
    egui::{ArpeggiatorWidget, LfoControllerWidget},
    prelude::*,
};
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable,
    InnerTransformsAudio, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    Control,
    InnerConfigurable,
    InnerHandlesMidi,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(Controls, GeneratesStereoSample, Serializable, Ticks, TransformsAudio)]
pub struct Arpeggiator {
    uid: Uid,
    inner: controllers::Arpeggiator,
}
impl Displays for Arpeggiator {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(ArpeggiatorWidget::widget(&mut self.inner))
    }
}
impl Arpeggiator {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: controllers::Arpeggiator::new_with(MidiChannel::default()),
        }
    }
}

#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct LfoController {
    uid: Uid,
    inner: controllers::LfoController,
}
impl LfoController {
    pub fn new_with(uid: Uid, oscillator: Oscillator) -> Self {
        Self {
            uid,
            inner: controllers::LfoController::new_with(oscillator),
        }
    }
}
impl Displays for LfoController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(LfoControllerWidget::widget(
            &mut self.inner.oscillator.waveform,
            &mut self.inner.oscillator.frequency,
        ));
        if response.changed() {
            self.inner.notify_change_oscillator();
        }
        response
    }
}

#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    InnerTransformsAudio,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(GeneratesStereoSample, Ticks)]
pub struct SignalPassthroughController {
    uid: Uid,
    inner: controllers::SignalPassthroughController,
}
impl Displays for SignalPassthroughController {}
impl SignalPassthroughController {
    #[allow(unused_variables)]
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: controllers::SignalPassthroughController::new(),
        }
    }

    pub fn new_amplitude_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: controllers::SignalPassthroughController::new_amplitude_passthrough_type(),
        }
    }

    pub fn new_amplitude_inverted_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner:
                controllers::SignalPassthroughController::new_amplitude_inverted_passthrough_type(),
        }
    }
}

#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct Timer {
    uid: Uid,
    inner: crate::automation::Timer,
}
impl Displays for Timer {}
impl Timer {
    pub fn new_with(uid: Uid, duration: MusicalTime) -> Self {
        Self {
            uid,
            inner: crate::automation::Timer::new_with(duration),
        }
    }
}

#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct Trigger {
    uid: Uid,
    inner: crate::automation::Trigger,
}
impl Displays for Trigger {}
impl Trigger {
    pub fn new_with(uid: Uid, timer: crate::automation::Timer, value: ControlValue) -> Self {
        Self {
            uid,
            inner: crate::automation::Trigger::new_with(timer, value),
        }
    }
}
