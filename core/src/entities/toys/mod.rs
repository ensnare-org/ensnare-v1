// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
    instruments::Synthesizer,
    midi::prelude::*,
    modulators::{Dca, DcaParams},
    prelude::*,
    traits::{prelude::*, GeneratesEnvelope},
    voices::{VoiceCount, VoiceStore},
    widgets::indicator,
};
use eframe::{
    egui::{self, Layout, Ui},
    emath::Align,
};
use ensnare_proc_macros::{Control, IsController, IsEffect, IsInstrument, Params, Uid};
use serde::{Deserialize, Serialize};
use std::{
    ops::Range,
    sync::{Arc, Mutex},
};

use super::{EntityFactory, EntityKey};

#[derive(Debug, Default)]
pub struct ToyInstrumentEphemerals {
    sample: StereoSample,
    pub is_playing: bool,
    pub received_midi_message_count: Arc<Mutex<usize>>,
    pub debug_messages: Vec<MidiMessage>,
}

/// An [IsInstrument](ensnare::traits::IsInstrument) that uses a default
/// [Oscillator] to produce sound. Its "envelope" is just a boolean that responds
/// to MIDI NoteOn/NoteOff.
#[derive(Debug, Default, Control, IsInstrument, Params, Uid, Serialize, Deserialize)]
pub struct ToyInstrument {
    uid: Uid,

    oscillator: Oscillator,

    #[control]
    #[params]
    dca: Dca,

    #[serde(skip)]
    e: ToyInstrumentEphemerals,
}
impl Generates<StereoSample> for ToyInstrument {
    fn value(&self) -> StereoSample {
        self.e.sample
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        for value in values {
            self.tick(1);
            *value = self.value();
        }
    }
}
impl Configurable for ToyInstrument {
    fn sample_rate(&self) -> SampleRate {
        self.oscillator.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.oscillator.update_sample_rate(sample_rate);
    }
}
impl Ticks for ToyInstrument {
    fn tick(&mut self, tick_count: usize) {
        self.oscillator.tick(tick_count);
        self.e.sample = if self.e.is_playing {
            self.dca
                .transform_audio_to_stereo(Sample::from(self.oscillator.value()))
        } else {
            StereoSample::SILENCE
        };
    }
}
impl HandlesMidi for ToyInstrument {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        self.e.debug_messages.push(message);
        if let Ok(mut received_count) = self.e.received_midi_message_count.lock() {
            *received_count += 1;
        }

        match message {
            MidiMessage::NoteOn { key, vel: _ } => {
                self.e.is_playing = true;
                self.oscillator.set_frequency(key.into());
            }
            MidiMessage::NoteOff { key: _, vel: _ } => {
                self.e.is_playing = false;
            }
            _ => {}
        }
    }
}
impl Serializable for ToyInstrument {}
impl ToyInstrument {
    pub fn new_with(params: &ToyInstrumentParams) -> Self {
        Self {
            uid: Default::default(),
            oscillator: Oscillator::new_with(&OscillatorParams::default_with_waveform(
                Waveform::Sine,
            )),
            dca: Dca::new_with(&params.dca),
            e: Default::default(),
        }
    }

    // If this instrument is being used in an integration test, then
    // received_midi_message_count provides insight into whether messages are
    // arriving.
    pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
        &self.e.received_midi_message_count
    }
}
impl Displays for ToyInstrument {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label(self.name())
    }
}

/// An [IsEffect](ensnare::traits::IsEffect) that negates the input signal.
#[derive(Debug, Default, Control, IsEffect, Params, Uid, Serialize, Deserialize)]
pub struct ToyEffect {
    uid: Uid,

    #[serde(skip)]
    sample_rate: SampleRate,
}
impl TransformsAudio for ToyEffect {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        -input_sample
    }
}
impl Configurable for ToyEffect {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}
impl Serializable for ToyEffect {}

impl Displays for ToyEffect {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.name())
    }
}

enum TestControllerAction {
    Nothing,
    NoteOn,
    NoteOff,
}

/// An [IsController](ensnare_core::traits::IsController) that emits a MIDI
/// note-on event on each beat, and a note-off event on each half-beat.
#[derive(Debug, Default, Control, IsController, Params, Uid, Serialize, Deserialize)]
pub struct ToyController {
    uid: Uid,

    #[serde(skip)]
    midi_channel_out: MidiChannel,

    #[serde(skip)]
    is_enabled: bool,
    #[serde(skip)]
    is_playing: bool,
    #[serde(skip)]
    is_performing: bool,

    #[serde(skip)]
    time_range: Range<MusicalTime>,

    #[serde(skip)]
    last_time_handled: MusicalTime,
}
impl Serializable for ToyController {}
impl Controls for ToyController {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.time_range = range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        match self.what_to_do() {
            TestControllerAction::Nothing => {}
            TestControllerAction::NoteOn => {
                // This is elegant, I hope. If the arpeggiator is
                // disabled during play, and we were playing a note,
                // then we still send the off note,
                if self.is_enabled && self.is_performing {
                    self.is_playing = true;
                    control_events_fn(
                        self.uid,
                        EntityEvent::Midi(self.midi_channel_out, new_note_on(60, 127)),
                    );
                }
            }
            TestControllerAction::NoteOff => {
                if self.is_playing {
                    control_events_fn(
                        self.uid,
                        EntityEvent::Midi(self.midi_channel_out, new_note_off(60, 0)),
                    );
                }
            }
        }
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn skip_to_start(&mut self) {}

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Configurable for ToyController {
    fn update_sample_rate(&mut self, _sample_rate: SampleRate) {}
}
impl HandlesMidi for ToyController {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        #[allow(unused_variables)]
        match message {
            MidiMessage::NoteOff { key, vel } => self.is_enabled = false,
            MidiMessage::NoteOn { key, vel } => self.is_enabled = true,
            _ => todo!(),
        }
    }
}
impl Displays for ToyController {
    fn ui(&mut self, ui: &mut Ui) -> eframe::egui::Response {
        ui.label(self.name())
    }
}
impl ToyController {
    pub fn new_with(_params: &ToyControllerParams, midi_channel_out: MidiChannel) -> Self {
        Self {
            midi_channel_out,
            ..Default::default()
        }
    }

    fn what_to_do(&mut self) -> TestControllerAction {
        if !self.time_range.contains(&self.last_time_handled) {
            self.last_time_handled = self.time_range.start;
            if self.time_range.start.units() == 0 {
                if self.time_range.start.parts() == 0 {
                    return TestControllerAction::NoteOn;
                }
                if self.time_range.start.parts() == 8 {
                    return TestControllerAction::NoteOff;
                }
            }
        }
        TestControllerAction::Nothing
    }
}

#[derive(Debug, Default, Control, IsInstrument, Params, Uid, Serialize, Deserialize)]
pub struct ToySynth {
    uid: Uid,

    #[params]
    voice_count: VoiceCount,

    #[control]
    #[params]
    waveform: Waveform,

    #[control]
    #[params]
    envelope: Envelope,

    #[control]
    #[params]
    dca: Dca,

    // TODO: this skip is a can of worms. I don't know whether we want to
    // serialize everything, or manually reconstitute everything. Maybe the
    // right answer is to expect that every struct gets serialized, but everyone
    // should be #[serde(skip)]ing at the leaf-field level.
    #[serde(skip)]
    inner: Synthesizer<ToyVoice>,

    #[serde(skip)]
    max_signal: Normal,
}
impl Serializable for ToySynth {}
impl Generates<StereoSample> for ToySynth {
    fn value(&self) -> StereoSample {
        self.inner.value()
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        // TODO: temp hack to avoid the pain of figuring out how deal with
        // Synthesizer sharing a single DCA across voices.
        for voice in self.inner.voices_mut() {
            voice.dca.set_pan(self.dca.pan());
        }
        self.inner.generate_batch_values(values);
        self.update_max();
    }
}
impl HandlesMidi for ToySynth {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
        self.inner
            .handle_midi_message(channel, message, midi_messages_fn)
    }
}
impl Ticks for ToySynth {
    fn tick(&mut self, tick_count: usize) {
        self.inner.tick(tick_count);

        self.update_max();
    }
}
impl Configurable for ToySynth {
    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.inner.update_sample_rate(sample_rate)
    }
}
impl ToySynth {
    pub fn new_with(params: &ToySynthParams) -> Self {
        let voice_store = VoiceStore::<ToyVoice>::new_with_voice(params.voice_count(), || {
            ToyVoice::new_with(params.waveform(), &params.envelope)
        });
        Self {
            uid: Default::default(),
            voice_count: params.voice_count(),
            waveform: params.waveform(),
            envelope: Envelope::new_with(&params.envelope),
            dca: Dca::new_with(&params.dca),
            inner: Synthesizer::<ToyVoice>::new_with(Box::new(voice_store)),
            max_signal: Normal::minimum(),
        }
    }

    fn update_max(&mut self) {
        let value = Normal::from(Sample::from(self.value()).0);
        if value > self.max_signal {
            self.max_signal = value;
        }
    }

    pub fn degrade_max(&mut self, factor: f64) {
        self.max_signal *= factor;
    }

    pub fn voice_count(&self) -> VoiceCount {
        self.voice_count
    }

    pub fn set_voice_count(&mut self, voice_count: VoiceCount) {
        self.voice_count = voice_count;
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    pub fn set_envelope(&mut self, envelope: Envelope) {
        self.envelope = envelope;
    }
}

#[derive(Debug, Default)]
struct ToyVoice {
    oscillator: Oscillator,
    envelope: Envelope,
    dca: Dca,
    value: StereoSample,
}
impl IsStereoSampleVoice for ToyVoice {}
impl IsVoice<StereoSample> for ToyVoice {}
impl PlaysNotes for ToyVoice {
    fn is_playing(&self) -> bool {
        !self.envelope.is_idle()
    }

    fn note_on(&mut self, key: u7, _velocity: u7) {
        self.envelope.trigger_attack();
        self.oscillator.set_frequency(key.into());
    }

    fn aftertouch(&mut self, _velocity: u7) {
        todo!()
    }

    fn note_off(&mut self, _velocity: u7) {
        self.envelope.trigger_release()
    }
}
impl Generates<StereoSample> for ToyVoice {
    fn value(&self) -> StereoSample {
        self.value
    }

    #[allow(unused_variables)]
    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        todo!()
    }
}
impl Ticks for ToyVoice {
    fn tick(&mut self, tick_count: usize) {
        self.oscillator.tick(tick_count);
        self.envelope.tick(tick_count);
        self.value = self.dca.transform_audio_to_stereo(
            (self.oscillator.value().value() * self.envelope.value().value()).into(),
        );
    }
}
impl Configurable for ToyVoice {
    fn sample_rate(&self) -> SampleRate {
        self.oscillator.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.oscillator.update_sample_rate(sample_rate);
        self.envelope.update_sample_rate(sample_rate);
    }
}
impl ToyVoice {
    fn new_with(waveform: Waveform, envelope: &EnvelopeParams) -> Self {
        Self {
            oscillator: Oscillator::new_with(&OscillatorParams::default_with_waveform(waveform)),
            envelope: Envelope::new_with(envelope),
            dca: Dca::default(),
            value: Default::default(),
        }
    }
}
impl Displays for ToySynth {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        let height = ui.available_height();
        ui.set_min_size(ui.available_size());
        ui.set_max_size(ui.available_size());
        if height <= 32.0 {
            self.show_small(ui)
        } else if height <= 128.0 {
            self.show_medium(ui)
        } else {
            self.show_full(ui)
        }
    }
}
impl ToySynth {
    fn show_small(&mut self, ui: &mut Ui) -> egui::Response {
        let response = ui
            .horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.label("ToySynth")
                })
                .inner
                    | ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(indicator(self.max_signal))
                    })
                    .inner
            })
            .inner;
        self.degrade_max(0.95);
        response
    }
    fn show_medium(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("ToySynth MEDIUM!");
        let value = Normal::from(0.5);
        ui.add(indicator(value))
    }
    fn show_full(&mut self, ui: &mut Ui) -> egui::Response {
        ui.label("ToySynth LARGE!!!!");
        let value = Normal::from(0.8);
        ui.add(indicator(value))
    }
}

/// Registers all [EntityFactory]'s entities. Note that the function returns a
/// EntityFactory, rather than operating on an &mut. This encourages
/// one-and-done creation, after which the factory is immutable:
///
/// ```ignore
/// let factory = register_factory_entities(EntityFactory::default());
/// ```
///
/// TODO: maybe a Builder pattern is better, so that people can compose
/// factories out of any entities they want, and still get the benefits of
/// immutability.
#[must_use]
pub fn register_toy_factory_entities(mut factory: EntityFactory) -> EntityFactory {
    factory.register_entity(EntityKey::from("toy-synth"), || {
        Box::new(ToySynth::new_with(&ToySynthParams::default()))
    });
    factory.register_entity(EntityKey::from("toy-instrument"), || {
        Box::<ToyInstrument>::default()
    });
    factory.register_entity(EntityKey::from("toy-controller"), || {
        Box::<ToyController>::default()
    });
    factory.register_entity(EntityKey::from("toy-effect"), || {
        Box::<ToyEffect>::default()
    });
    // factory.register_entity(Key::from("toy-controller-noisy"), || {
    //     Box::new(ToyControllerAlwaysSendsMidiMessage::default())
    // });

    factory.complete_registration();

    factory
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // TODO: restore tests that test basic trait behavior, then figure out how
    // to run everyone implementing those traits through that behavior. For now,
    // this one just tests that a generic instrument doesn't panic when accessed
    // for non-consecutive time slices.
    #[test]
    fn sources_audio_random_access() {
        let mut instrument = ToyInstrument::default();
        let mut rng = oorandom::Rand32::new(0);

        for _ in 0..100 {
            instrument.tick(rng.rand_range(1..10) as usize);
            let _ = instrument.value();
        }
    }
}
