// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crossbeam_channel::Sender;
use delegate::delegate;
use ensnare_core::{
    controllers::{ControlTripParams, TimerParams, TriggerParams},
    piano_roll::{Pattern, PatternUid, PianoRoll},
    prelude::*,
    sequence_repository::SequenceRepository,
    time::MusicalTime,
    traits::{Controls, Sequences},
    uid::Uid,
};
use ensnare_cores::{
    ArpeggiatorParams, LfoControllerParams, LivePatternSequencerParams,
    SignalPassthroughControllerParams, ThinSequencerParams,
};
use ensnare_cores_egui::controllers::{
    arpeggiator, lfo_controller, live_pattern_sequencer_widget, note_sequencer_widget,
    pattern_sequencer_widget, trip,
};
use ensnare_egui_widgets::ViewRange;
use ensnare_entity::prelude::*;
use ensnare_orchestration::ControlRouter;
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable,
    InnerTransformsAudio, IsEntity2, Metadata,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub enum SequencerInput {
    AddPattern(PatternUid, MusicalTime),
}

#[derive(
    Debug, Default, Control, InnerHandlesMidi, IsEntity2, Metadata, Serialize, Deserialize,
)]
#[entity2(
    Configurable,
    Controls,
    GeneratesStereoSample,
    Serializable,
    Ticks,
    TransformsAudio
)]
pub struct Arpeggiator {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Arpeggiator,
}
impl Displays for Arpeggiator {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(arpeggiator(&mut self.inner))
    }
}
impl Arpeggiator {
    pub fn new_with(uid: Uid, params: &ArpeggiatorParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Arpeggiator::new_with(&params, MidiChannel::default()),
        }
    }
}

#[derive(Debug, Default, InnerControls, IsEntity2, Metadata, Serialize, Deserialize)]
#[entity2(
    Configurable,
    Controllable,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    Ticks,
    TransformsAudio,
    SkipInner
)]
pub struct PatternSequencer {
    uid: Uid,
    inner: ensnare_cores::PatternSequencer,
    #[serde(skip)]
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
    Debug,
    Default,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controllable, GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct LivePatternSequencer {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::LivePatternSequencer,
    #[serde(skip)]
    channel: ChannelPair<SequencerInput>,
    #[serde(skip)]
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
impl Sequences for LivePatternSequencer {
    type MU = PatternUid;

    delegate! {
        to self.inner {
            fn record(&mut self, channel: MidiChannel, unit: &Self::MU, position: MusicalTime) -> anyhow::Result<()>;
            fn remove(&mut self, channel: MidiChannel, unit: &Self::MU, position: MusicalTime,) -> anyhow::Result<()>;
            fn clear(&mut self);
        }
    }
}
impl LivePatternSequencer {
    pub fn new_with(
        uid: Uid,
        params: &LivePatternSequencerParams,
        piano_roll: &Arc<RwLock<PianoRoll>>,
    ) -> Self {
        Self {
            uid,
            inner: ensnare_cores::LivePatternSequencer::new_with(params, piano_roll),
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
impl TryFrom<(Uid, &LivePatternSequencerParams, &Arc<RwLock<PianoRoll>>)> for LivePatternSequencer {
    type Error = anyhow::Error;

    fn try_from(
        value: (Uid, &LivePatternSequencerParams, &Arc<RwLock<PianoRoll>>),
    ) -> Result<Self, Self::Error> {
        Ok(Self::new_with(value.0, value.1, value.2))
    }
}

#[derive(Debug, Default, InnerControls, IsEntity2, Metadata, Serialize, Deserialize)]
#[entity2(
    Configurable,
    Controllable,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    Ticks,
    TransformsAudio,
    SkipInner
)]
pub struct NoteSequencer {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::NoteSequencer,
    #[serde(skip)]
    view_range: ViewRange,
}
impl Displays for NoteSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(note_sequencer_widget(&mut self.inner, &self.view_range))
    }
}
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
    Debug,
    Default,
    InnerConfigurable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(TransformsAudio, GeneratesStereoSample, Ticks, Controllable)]
pub struct ThinSequencer {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::ThinSequencer,
    #[serde(skip)]
    view_range: ViewRange,
}
impl Displays for ThinSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(live_pattern_sequencer_widget(
            &mut self.inner.inner,
            &self.view_range,
        ))
    }

    fn set_view_range(&mut self, view_range: &ViewRange) {
        self.view_range = view_range.clone();
    }
}
impl Sequences for ThinSequencer {
    type MU = PatternUid;

    delegate! {
        to self.inner {
            fn record(&mut self, channel: MidiChannel, unit: &Self::MU, position: MusicalTime) -> anyhow::Result<()>;
            fn remove(&mut self, channel: MidiChannel, unit: &Self::MU, position: MusicalTime,) -> anyhow::Result<()>;
            fn clear(&mut self);
        }
    }
}
impl ThinSequencer {
    pub fn new_with(
        uid: Uid,
        params: &ThinSequencerParams,
        track_uid: TrackUid,
        repository: &Arc<RwLock<SequenceRepository>>,
        piano_roll: &Arc<RwLock<PianoRoll>>,
    ) -> Self {
        Self {
            uid,
            inner: ensnare_cores::ThinSequencer::new_with(
                params, track_uid, repository, piano_roll,
            ),
            view_range: Default::default(),
        }
    }
}
impl
    TryFrom<(
        Uid,
        &ThinSequencerParams,
        TrackUid,
        &Arc<RwLock<SequenceRepository>>,
        &Arc<RwLock<PianoRoll>>,
    )> for ThinSequencer
{
    type Error = anyhow::Error;

    fn try_from(
        value: (
            Uid,
            &ThinSequencerParams,
            TrackUid,
            &Arc<RwLock<SequenceRepository>>,
            &Arc<RwLock<PianoRoll>>,
        ),
    ) -> Result<Self, Self::Error> {
        Ok(Self::new_with(value.0, value.1, value.2, value.3, value.4))
    }
}

#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct LfoController {
    uid: Uid,
    #[serde(skip)]
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
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(GeneratesStereoSample, Ticks)]
pub struct SignalPassthroughController {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::controllers::SignalPassthroughController,
}
impl Displays for SignalPassthroughController {}
impl SignalPassthroughController {
    #[allow(unused_variables)]
    pub fn new_with(uid: Uid, params: &SignalPassthroughControllerParams) -> Self {
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
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct Timer {
    uid: Uid,
    #[serde(skip)]
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
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct Trigger {
    uid: Uid,
    #[serde(skip)]
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
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct ControlTrip {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_core::controllers::ControlTrip,
    #[serde(skip)]
    control_router: Arc<RwLock<ControlRouter>>,
    #[serde(skip)]
    view_range: ViewRange,
}
impl Displays for ControlTrip {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let control_router = self.control_router.read().unwrap();
        ui.add(trip(
            self.uid,
            &mut self.inner,
            control_router.control_links(self.uid),
            self.view_range.clone(),
        ))
    }

    fn set_view_range(&mut self, view_range: &ViewRange) {
        self.view_range = view_range.clone();
    }
}
impl ControlTrip {
    pub fn new_with(
        uid: Uid,
        params: &ControlTripParams,
        control_router: &Arc<RwLock<ControlRouter>>,
    ) -> Self {
        Self {
            uid,
            inner: ensnare_core::controllers::ControlTrip::new_with(params),
            control_router: Arc::clone(control_router),
            view_range: Default::default(),
        }
    }
}
impl TryFrom<(Uid, &ControlTripParams, &Arc<RwLock<ControlRouter>>)> for ControlTrip {
    type Error = anyhow::Error;

    fn try_from(
        value: (Uid, &ControlTripParams, &Arc<RwLock<ControlRouter>>),
    ) -> Result<Self, Self::Error> {
        Ok(Self::new_with(value.0, value.1, value.2))
    }
}
