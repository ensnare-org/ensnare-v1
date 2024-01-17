// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{generators::Oscillator, prelude::*};
use ensnare_cores_egui::controllers::{
    arpeggiator, lfo_controller, note_sequencer_widget, pattern_sequencer_widget,
};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable,
    InnerTransformsAudio, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum SequencerInput {
    AddPattern(PatternUid, MusicalTime),
}

#[derive(Debug, Default, Control, InnerHandlesMidi, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controls,
    GeneratesStereoSample,
    Serializable,
    Ticks,
    TransformsAudio
)]
pub struct Arpeggiator {
    uid: Uid,
    inner: ensnare_cores::Arpeggiator,
}
impl Displays for Arpeggiator {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(arpeggiator(&mut self.inner))
    }
}
impl Arpeggiator {
    pub fn new_with(uid: Uid, bpm: Tempo) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Arpeggiator::new_with(bpm, MidiChannel::default()),
        }
    }
}

#[derive(Debug, Default, InnerControls, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
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

#[cfg(obsolete)]
mod obsolete {
    #[derive(
        Debug,
        Default,
        InnerConfigurable,
        InnerHandlesMidi,
        InnerSerializable,
        IsEntity,
        Metadata,
        Serialize,
        Deserialize,
    )]
    #[entity(Controllable, GeneratesStereoSample, Ticks, TransformsAudio)]
    pub struct LivePatternSequencer {
        uid: Uid,
        inner: ensnare_cores::LivePatternSequencer,
        #[serde(skip)]
        input_channels: ChannelPair<SequencerInput>,
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
        pub fn new_with(uid: Uid, composer: &Arc<RwLock<Composer>>) -> Self {
            Self {
                uid,
                inner: ensnare_cores::LivePatternSequencer::new_with(composer),
                view_range: Default::default(),
                input_channels: Default::default(),
            }
        }

        pub fn sender(&self) -> &Sender<SequencerInput> {
            &self.input_channels.sender
        }
    }
    impl Controls for LivePatternSequencer {
        fn update_time_range(&mut self, range: &TimeRange) {
            self.inner.update_time_range(range)
        }

        fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
            while let Ok(input) = self.input_channels.receiver.try_recv() {
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
}
#[derive(Debug, Default, InnerControls, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
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
#[entity(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct LfoController {
    uid: Uid,
    inner: ensnare_cores::LfoController,
}
impl LfoController {
    pub fn new_with(uid: Uid, oscillator: Oscillator) -> Self {
        Self {
            uid,
            inner: ensnare_cores::LfoController::new_with(oscillator),
        }
    }
}
impl Displays for LfoController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(lfo_controller(
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
#[entity(GeneratesStereoSample, Ticks)]
pub struct SignalPassthroughController {
    uid: Uid,
    inner: ensnare_cores::controllers::SignalPassthroughController,
}
impl Displays for SignalPassthroughController {}
impl SignalPassthroughController {
    #[allow(unused_variables)]
    pub fn new_with(uid: Uid) -> Self {
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
    Serialize,
    Deserialize,
)]
#[entity(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct Timer {
    uid: Uid,
    inner: ensnare_core::controllers::Timer,
}
impl Displays for Timer {}
impl Timer {
    pub fn new_with(uid: Uid, duration: MusicalTime) -> Self {
        Self {
            uid,
            inner: ensnare_core::controllers::Timer::new_with(duration),
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
#[entity(GeneratesStereoSample, Ticks, TransformsAudio)]
pub struct Trigger {
    uid: Uid,
    inner: ensnare_core::controllers::Trigger,
}
impl Displays for Trigger {}
impl Trigger {
    pub fn new_with(
        uid: Uid,
        timer: ensnare_core::controllers::Timer,
        value: ControlValue,
    ) -> Self {
        Self {
            uid,
            inner: ensnare_core::controllers::Trigger::new_with(timer, value),
        }
    }
}
