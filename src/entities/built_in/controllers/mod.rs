// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    automation::{TimerCore, TriggerCore},
    cores::controllers::{
        ArpeggiatorCore, LfoControllerCore, SignalPassthroughControllerCore,
        SignalPassthroughControllerCoreBuilder,
    },
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
#[entity(Controls, GeneratesStereoSample, Serializable, TransformsAudio)]
pub struct Arpeggiator {
    uid: Uid,
    inner: ArpeggiatorCore,
}
impl Arpeggiator {
    pub fn new_with(uid: Uid, inner: ArpeggiatorCore) -> Self {
        Self { uid, inner }
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
#[entity(GeneratesStereoSample, TransformsAudio)]
pub struct LfoController {
    uid: Uid,
    inner: LfoControllerCore,
}
impl LfoController {
    pub fn new_with(uid: Uid, inner: LfoControllerCore) -> Self {
        Self { uid, inner }
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
#[entity(GeneratesStereoSample)]
pub struct SignalPassthroughController {
    uid: Uid,
    inner: SignalPassthroughControllerCore,
}
impl SignalPassthroughController {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: SignalPassthroughControllerCoreBuilder::default()
                .build()
                .unwrap(),
        }
    }

    pub fn new_amplitude_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: SignalPassthroughControllerCoreBuilder::amplitude()
                .build()
                .unwrap(),
        }
    }

    pub fn new_amplitude_inverted_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: SignalPassthroughControllerCoreBuilder::amplitude_inverted()
                .build()
                .unwrap(),
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
#[entity(GeneratesStereoSample, TransformsAudio)]
pub struct Timer {
    uid: Uid,
    inner: TimerCore,
}
impl Timer {
    pub fn new_with(uid: Uid, duration: MusicalTime) -> Self {
        Self {
            uid,
            inner: TimerCore::new_with(duration),
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
#[entity(GeneratesStereoSample, TransformsAudio)]
pub struct Trigger {
    uid: Uid,
    inner: TriggerCore,
}
impl Trigger {
    pub fn new_with(uid: Uid, timer: TimerCore, value: ControlValue) -> Self {
        Self {
            uid,
            inner: TriggerCore::new_with(timer, value),
        }
    }
}

#[cfg(feature = "egui")]
mod egui {
    use super::*;
    use crate::egui::{ArpeggiatorWidget, LfoControllerWidget};

    impl Displays for Arpeggiator {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.add(ArpeggiatorWidget::widget(&mut self.inner))
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
    impl Displays for SignalPassthroughController {}
    impl Displays for Timer {}
    impl Displays for Trigger {}
}
