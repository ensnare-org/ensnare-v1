// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
    instruments::Synthesizer,
    midi::prelude::*,
    modulators::{Dca, DcaParams},
    prelude::*,
    traits::{prelude::*, GeneratesEnvelope},
    voices::{VoiceCount, VoiceStore},
    widgets::{audio::waveform, parts::UiSize},
};
use eframe::{
    egui::{self, Layout, Ui},
    emath::Align,
};
use ensnare_egui_widgets::level_indicator;
use ensnare_proc_macros::{Control, IsInstrument, Params, Uid};
use serde::{Deserialize, Serialize};

/// Implements a very small, but complete, synthesizer. This means it implements
/// a polyphonic [IsInstrument] with [Controllable] parameters.
#[derive(Control, Debug, Default, IsInstrument, Params, Uid, Serialize, Deserialize)]
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

    #[serde(skip)]
    ui_size: UiSize,
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
impl Displays for ToySynth {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        let height = ui.available_height();
        self.ui_size = UiSize::from_height(height);
        match self.ui_size {
            UiSize::Small => self.show_small(ui),
            UiSize::Medium => self.show_medium(ui),
            UiSize::Large => self.show_full(ui),
        }
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
            ui_size: Default::default(),
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

    fn handle_ui_waveform(&mut self, ui: &mut Ui) -> egui::Response {
        let response = ui.add(waveform(&mut self.waveform));
        if response.changed() {
            self.inner
                .voices_mut()
                .for_each(|v| v.oscillator.set_waveform(self.waveform));
        }
        response
    }

    fn handle_ui_envelope(&mut self, ui: &mut Ui) -> egui::Response {
        let response = ui
            .scope(|ui| {
                ui.set_max_size(eframe::epaint::vec2(256.0, 64.0));
                let response = self.envelope.ui(ui);
                response
            })
            .inner;
        if response.changed() {
            self.inner.voices_mut().for_each(|v| {
                v.envelope.set_attack(self.envelope.attack());
                v.envelope.set_decay(self.envelope.decay());
                v.envelope.set_sustain(self.envelope.sustain());
                v.envelope.set_release(self.envelope.release());
            });
        }
        response
    }

    fn show_small(&mut self, ui: &mut Ui) -> egui::Response {
        let response = ui
            .horizontal(|ui| {
                ui.set_max_width(192.0);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    self.handle_ui_waveform(ui)
                })
                .inner
                    | ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(level_indicator(self.max_signal.into()))
                    })
                    .inner
            })
            .inner;
        self.degrade_max(0.95);
        response
    }
    fn show_medium(&mut self, ui: &mut Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.heading("ToySynth");
            let waveform_response = self.handle_ui_waveform(ui);
            let envelope_response = self.handle_ui_envelope(ui);
            waveform_response | envelope_response
        })
        .inner
    }
    fn show_full(&mut self, ui: &mut Ui) -> egui::Response {
        ui.heading("ToySynth LARGE!!!!");
        let value = Normal::from(0.8);
        ui.add(level_indicator(value.into()))
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
            (self.oscillator.value().0 * self.envelope.value().0).into(),
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
