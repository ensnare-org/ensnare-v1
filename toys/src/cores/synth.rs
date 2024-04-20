// Copyright (c) 2023 Mike Tsao. All rights reserved.

use delegate::delegate;
use ensnare::{
    prelude::*,
    traits::{CanPrototype, GeneratesEnvelope, GenerationBuffer},
};
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct ToyVoice {
    pub oscillator: Oscillator,
    pub envelope: Envelope,
    pub dca: Dca,
    oscillator_buffer: GenerationBuffer<BipolarNormal>,
    envelope_buffer: GenerationBuffer<Normal>,
    mono_buffer: GenerationBuffer<Sample>,
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
    fn generate(&mut self, values: &mut [StereoSample]) {
        self.oscillator_buffer.set_buffer_size(values.len());
        self.envelope_buffer.set_buffer_size(values.len());
        self.mono_buffer.set_buffer_size(values.len());
        self.oscillator
            .generate(self.oscillator_buffer.buffer_mut());
        self.envelope.generate(self.envelope_buffer.buffer_mut());
        self.mono_buffer
            .buffer_mut()
            .iter_mut()
            .zip(
                self.oscillator_buffer
                    .buffer()
                    .iter()
                    .zip(self.envelope_buffer.buffer().iter()),
            )
            .for_each(|(dst, (osc, env))| *dst = (*osc * *env).into());
        self.dca
            .transform_batch_to_stereo(self.mono_buffer.buffer(), values);
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
    fn new_with(oscillator: &Oscillator, envelope: &Envelope, dca: &Dca) -> Self {
        Self {
            oscillator: oscillator.make_another(),
            envelope: envelope.make_another(),
            dca: dca.make_another(),
            oscillator_buffer: Default::default(),
            envelope_buffer: Default::default(),
            mono_buffer: Default::default(),
        }
    }
}

/// Implements a small but complete synthesizer.
#[derive(Control, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ToySynthCore {
    voice_count: VoiceCount,

    #[control]
    pub oscillator: Oscillator,

    #[control]
    pub envelope: Envelope,

    #[control]
    dca: Dca,

    #[serde(skip)]
    pub inner: Synthesizer<ToyVoice>,
}
impl Serializable for ToySynthCore {}
impl Generates<StereoSample> for ToySynthCore {
    delegate! {
        to self.inner {
            fn generate(&mut self, values: &mut [StereoSample]);
        }
    }
}
impl HandlesMidi for ToySynthCore {
    delegate! {
        to self.inner {
            fn handle_midi_message(
                &mut self,
                channel: MidiChannel,
                message: MidiMessage,
                midi_messages_fn: &mut MidiMessagesFn,
            );
        }
    }
}
impl Configurable for ToySynthCore {
    delegate! {
        to self.inner {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
        }
    }
}
impl ToySynthCore {
    pub fn new_with(oscillator: Oscillator, envelope: Envelope, dca: Dca) -> Self {
        let voice_store = VoiceStore::<ToyVoice>::new_with_voice(VoiceCount::default(), || {
            ToyVoice::new_with(&oscillator, &envelope, &dca)
        });
        Self {
            voice_count: Default::default(),
            oscillator,
            envelope,
            dca,
            inner: Synthesizer::<ToyVoice>::new_with(Box::new(voice_store)),
        }
    }

    pub fn voice_count(&self) -> VoiceCount {
        self.voice_count
    }

    pub fn set_voice_count(&mut self, voice_count: VoiceCount) {
        self.voice_count = voice_count;
    }

    pub fn oscillator(&self) -> &Oscillator {
        &self.oscillator
    }

    pub fn notify_change_oscillator(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.oscillator.update_from_prototype(&self.oscillator);
        });
    }

    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    pub fn notify_change_envelope(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.envelope.update_from_prototype(&self.envelope);
        });
    }

    pub fn dca(&self) -> &Dca {
        &self.dca
    }

    pub fn notify_change_dca(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.dca.update_from_prototype(&self.dca);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cores::tests::{check_instrument, GeneratesStereoSampleAndHandlesMidi};
    use ensnare::elements::OscillatorBuilder;

    #[test]
    fn toy_synth_control() {
        let mut synth = ToySynthCore::new_with(
            OscillatorBuilder::default().build().unwrap(),
            EnvelopeBuilder::safe_default().build().unwrap(),
            Dca::default(),
        );

        assert_eq!(
            synth.inner.voice_count(),
            VoiceCount::default().0,
            "New synth should have some voices"
        );

        synth.inner.voices().for_each(|v| {
            assert_eq!(
                v.dca.gain(),
                synth.dca().gain(),
                "Master DCA gain is same as all voice DCA gain"
            );
        });

        let param_index = synth.control_index_for_name("dca-gain").unwrap();
        assert_ne!(
            synth.dca().gain().0,
            0.22,
            "we're about to set DCA gain to something different from its current value"
        );
        synth.control_set_param_by_index(param_index, ControlValue(0.22));
        assert_eq!(synth.dca().gain().0, 0.22);
        synth.inner.voices().for_each(|v| {
            assert_eq!(
                synth.dca().gain(),
                v.dca.gain(),
                "all voices update gain after setting master"
            );
        });
    }

    impl GeneratesStereoSampleAndHandlesMidi for ToySynthCore {}

    #[test]
    fn toy_synth_works() {
        let mut instrument = ToySynthCore::new_with(
            OscillatorBuilder::default().build().unwrap(),
            EnvelopeBuilder::safe_default().build().unwrap(),
            Dca::default(),
        );
        check_instrument(&mut instrument);
    }
}
