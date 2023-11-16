// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crossbeam_channel::Sender;
use ensnare_core::{
    controllers::{TimerParams, TriggerParams},
    piano_roll::{Pattern, PatternUid, PianoRoll},
    prelude::*,
    time::MusicalTime,
    traits::{Configurable, Controls, Sequences, Serializable},
    uid::Uid,
};
use ensnare_cores::{ArpeggiatorParams, LfoControllerParams};
use ensnare_cores_egui::controllers::{
    arpeggiator, lfo_controller, live_pattern_sequencer_widget, note_sequencer_widget,
    pattern_sequencer_widget, trip,
};
use ensnare_egui_widgets::ViewRange;
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable,
    InnerTransformsAudio, IsEntity, Metadata,
};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub enum SequencerInput {
    AddPattern(PatternUid, MusicalTime),
}

#[derive(Debug, Default, Control, InnerHandlesMidi, IsEntity, Metadata)]
#[entity("controller")]
pub struct Arpeggiator {
    uid: Uid,
    inner: ensnare_cores::Arpeggiator,
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
            inner: ensnare_cores::Arpeggiator::new_with(&params, MidiChannel::default()),
        }
    }
}

#[derive(Debug, Default, InnerControls, IsEntity, Metadata)]
#[entity("controller")]
pub struct PatternSequencer {
    uid: Uid,
    inner: ensnare_cores::PatternSequencer,
    view_range: ViewRange,
}
impl Displays for PatternSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(pattern_sequencer_widget(&mut self.inner, &self.view_range))
    }

    fn set_view_range(&mut self, view_range: &ViewRange) {
        self.view_range = view_range.clone();
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

#[derive(
    Debug, Default, InnerConfigurable, InnerHandlesMidi, InnerSerializable, IsEntity, Metadata,
)]
#[entity("controller", "timeline")]
pub struct LivePatternSequencer {
    uid: Uid,
    inner: ensnare_cores::LivePatternSequencer,
    channel: ChannelPair<SequencerInput>,
    view_range: ViewRange,
}
impl Displays for LivePatternSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(live_pattern_sequencer_widget(
            &mut self.inner,
            &self.view_range,
        ))
    }

    fn set_view_range(&mut self, view_range: &ViewRange) {
        self.view_range = view_range.clone();
    }
}
impl LivePatternSequencer {
    pub fn new_with(uid: Uid, piano_roll: Arc<RwLock<PianoRoll>>) -> Self {
        Self {
            uid,
            inner: ensnare_cores::LivePatternSequencer::new_with(piano_roll),
            view_range: Default::default(),
            channel: Default::default(),
        }
    }

    pub fn sender(&self) -> &Sender<SequencerInput> {
        &self.channel.sender
    }
}
impl Controls for LivePatternSequencer {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.inner.update_time_range(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        while let Ok(input) = self.channel.receiver.try_recv() {
            match input {
                SequencerInput::AddPattern(pattern_uid, position) => {
                    let _ = self
                        .inner
                        .record(MidiChannel::default(), &pattern_uid, position);
                }
            }
        }
        self.inner.work(control_events_fn)
    }

    fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }

    fn play(&mut self) {
        self.inner.play()
    }

    fn stop(&mut self) {
        self.inner.stop()
    }

    fn skip_to_start(&mut self) {
        self.inner.skip_to_start()
    }

    fn is_performing(&self) -> bool {
        self.inner.is_performing()
    }
}

#[derive(Debug, Default, InnerControls, IsEntity, Metadata)]
#[entity("controller")]
pub struct NoteSequencer {
    uid: Uid,
    inner: ensnare_cores::NoteSequencer,
    view_range: ViewRange,
}
impl Displays for NoteSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(note_sequencer_widget(&mut self.inner, &self.view_range))
    }
}
impl Configurable for NoteSequencer {}
impl HandlesMidi for NoteSequencer {}
impl Serializable for NoteSequencer {}
impl NoteSequencer {
    pub fn new_with_inner(uid: Uid, inner: ensnare_cores::NoteSequencer) -> Self {
        Self {
            uid,
            inner,
            view_range: Default::default(),
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
)]
#[entity("controller")]
pub struct LfoController {
    uid: Uid,
    inner: ensnare_cores::LfoController,
}
impl LfoController {
    pub fn new_with(uid: Uid, params: &LfoControllerParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::LfoController::new_with(params),
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
    InnerTransformsAudio,
    IsEntity,
    Metadata,
)]
#[entity("controller", "effect")]

pub struct SignalPassthroughController {
    uid: Uid,
    inner: ensnare_cores::controllers::SignalPassthroughController,
}
impl Displays for SignalPassthroughController {}
impl SignalPassthroughController {
    pub fn new(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_cores::controllers::SignalPassthroughController::new(),
        }
    }

    pub fn new_amplitude_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_cores::controllers::SignalPassthroughController::new_amplitude_passthrough_type(),
        }
    }

    pub fn new_amplitude_inverted_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: ensnare_cores::controllers::SignalPassthroughController::new_amplitude_inverted_passthrough_type(),
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
)]
#[entity("controller")]
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
    IsEntity,
    Metadata,
)]
#[entity("controller")]
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
    IsEntity,
    Metadata,
)]
#[entity("controller", "timeline")]
pub struct ControlTrip {
    uid: Uid,
    inner: ensnare_core::controllers::ControlTrip,
    view_range: ViewRange,
}
impl Displays for ControlTrip {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(trip(
            self.uid,
            &mut self.inner,
            None,
            self.view_range.clone(),
        ))
    }

    fn set_view_range(&mut self, view_range: &ViewRange) {
        self.view_range = view_range.clone();
    }
}
impl ControlTrip {
    pub fn new_with(uid: Uid, inner: ensnare_core::controllers::ControlTrip) -> Self {
        Self {
            uid,
            inner,
            view_range: Default::default(),
        }
    }
}
