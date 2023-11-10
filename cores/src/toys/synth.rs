// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
    instruments::Synthesizer,
    modulators::{Dca, DcaParams},
    prelude::*,
    traits::GeneratesEnvelope,
    voices::{VoiceCount, VoiceStore},
};
use ensnare_proc_macros::{Control, Params};

/// Implements a very small, but complete, synthesizer.
#[derive(Control, Debug, Default, Params)]
pub struct ToySynth {
    #[params]
    voice_count: VoiceCount,

    #[control]
    #[params]
    pub waveform: Waveform,

    #[control]
    #[params]
    pub envelope: Envelope,

    #[control]
    #[params]
    dca: Dca,

    pub inner: Synthesizer<ToyVoice>,

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
pub struct ToyVoice {
    pub oscillator: Oscillator,
    pub envelope: Envelope,
    pub dca: Dca,
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
