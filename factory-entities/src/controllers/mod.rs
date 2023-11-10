// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub mod keyboard;

use ensnare_core::{
    controllers::{TimerParams, TriggerParams},
    piano_roll::Pattern,
    prelude::*,
    stuff::{arpeggiator::ArpeggiatorParams, lfo::LfoControllerParams},
    time::MusicalTime,
    traits::{Configurable, Controls, Sequences, Serializable},
    uid::Uid,
};
use ensnare_egui::controllers::{
    arpeggiator, lfo_controller, note_sequencer_widget, pattern_sequencer_widget,
};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable, IsController,
    Metadata,
};

#[derive(Debug, Default, Control, InnerHandlesMidi, IsController, Metadata)]
pub struct Arpeggiator {
    uid: Uid,
    inner: ensnare_core::stuff::arpeggiator::Arpeggiator,
}
impl Displays for Arpeggiator {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(arpeggiator(&mut self.inner))
    }
}
impl Controls for Arpeggiator {}
impl Serializable for Arpeggiator {}
impl Configurable for Arpeggiator {}
impl Arpeggiator {
    pub fn new_with(uid: Uid, params: &ArpeggiatorParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::arpeggiator::Arpeggiator::new_with(
                &params,
                MidiChannel::default(),
            ),
        }
    }
}

#[derive(Debug, Default, InnerControls, IsController, Metadata)]
pub struct PatternSequencer {
    uid: Uid,
    inner: ensnare_core::entities::controllers::sequencers::PatternSequencer,
}
impl Displays for PatternSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let range = self.inner.inner.time_range.clone();
        ui.add(pattern_sequencer_widget(&mut self.inner, &range))
    }
}
impl Configurable for PatternSequencer {}
impl HandlesMidi for PatternSequencer {}
impl Serializable for PatternSequencer {}
impl Sequences for PatternSequencer {
    type MU = Pattern;

    fn record(
        &mut self,
        channel: MidiChannel,
        unit: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        self.inner.record(channel, unit, position)
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        unit: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        self.inner.remove(channel, unit, position)
    }

    fn clear(&mut self) {
        self.inner.clear()
    }
}

#[derive(Debug, Default, InnerControls, IsController, Metadata)]
pub struct NoteSequencer {
    uid: Uid,
    inner: ensnare_core::entities::controllers::sequencers::NoteSequencer,
}
impl Displays for NoteSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let range = self.inner.inner.time_range.clone();
        ui.add(note_sequencer_widget(&mut self.inner, &range))
    }
}
impl Configurable for NoteSequencer {}
impl HandlesMidi for NoteSequencer {}
impl Serializable for NoteSequencer {}
impl NoteSequencer {
    pub fn new_with_inner(
        uid: Uid,
        inner: ensnare_core::entities::controllers::sequencers::NoteSequencer,
    ) -> Self {
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
    IsController,
    Metadata,
)]

pub struct LfoController {
    uid: Uid,
    inner: ensnare_core::stuff::lfo::LfoController,
}
impl LfoController {
    pub fn new_with(uid: Uid, params: &LfoControllerParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::lfo::LfoController::new_with(params),
        }
    }
}
impl Displays for LfoController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut waveform = self.inner.waveform;
        let mut frequency = self.inner.frequency;
        let response = ui.add(lfo_controller(&mut waveform, &mut frequency));
        if response.changed() {
            self.inner.set_waveform(waveform);
            self.inner.set_frequency(frequency);
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
    IsController,
    Metadata,
)]
pub struct SignalPassthroughController {
    uid: Uid,
    inner: ensnare_core::entities::controllers::SignalPassthroughController,
}
impl Displays for SignalPassthroughController {}
impl SignalPassthroughController {
    pub fn new(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_core::entities::controllers::SignalPassthroughController::new(),
        }
    }

    pub fn new_amplitude_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_core::entities::controllers::SignalPassthroughController::new_amplitude_passthrough_type(),
        }
    }

    pub fn new_amplitude_inverted_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_core::entities::controllers::SignalPassthroughController::new_amplitude_inverted_passthrough_type(),
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
    IsController,
    Metadata,
)]
pub struct Timer {
    uid: Uid,
    inner: ensnare_core::controllers::Timer,
}
impl Displays for Timer {}
impl Timer {
    pub fn new_with(uid: Uid, params: &TimerParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::controllers::Timer::new_with(params),
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
    IsController,
    Metadata,
)]
pub struct Trigger {
    uid: Uid,
    inner: ensnare_core::controllers::Trigger,
}
impl Displays for Trigger {}
impl Trigger {
    pub fn new_with(uid: Uid, params: &TriggerParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::controllers::Trigger::new_with(params),
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
    IsController,
    Metadata,
)]
pub struct ControlTrip {
    uid: Uid,
    inner: ensnare_core::controllers::ControlTrip,
}
impl Displays for ControlTrip {}
impl ControlTrip {
    pub fn new_with(uid: Uid, inner: ensnare_core::controllers::ControlTrip) -> Self {
        Self { uid, inner }
    }
}
