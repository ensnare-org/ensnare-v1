// Copyright (c) 2023 Mike Tsao. All rights reserved.

use delegate::delegate;
use ensnare::{
    prelude::*,
    traits::{CanPrototype, GeneratesEnvelope},
};
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

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
    fn generate(&mut self, values: &mut [StereoSample]) {
        todo!()
    }
}
impl Ticks for ToyVoice {
    fn tick(&mut self, tick_count: usize) {
        self.oscillator.tick(tick_count);
        self.envelope.tick(tick_count);
        self.value = self
            .dca
            .transform_audio_to_stereo((self.oscillator.value() * self.envelope.value()).into());
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
            value: Default::default(),
        }
    }
}

/// Implements a small but complete synthesizer.
#[derive(Control, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ToySynth {
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
impl Serializable for ToySynth {}
impl Generates<StereoSample> for ToySynth {
    delegate! {
        to self.inner {
            fn value(&self) -> StereoSample;
            fn generate(&mut self, values: &mut [StereoSample]);
        }
    }
}
impl HandlesMidi for ToySynth {
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
impl Ticks for ToySynth {
    delegate! {
        to self.inner {
            fn tick(&mut self, tick_count: usize);
        }
    }
}
impl Configurable for ToySynth {
    delegate! {
        to self.inner {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
        }
    }
}
impl ToySynth {
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

    #[test]
    fn toy_synth_control() {
        let mut synth = ToySynth::new_with(
            Oscillator::new_with_waveform(Waveform::Sine),
            Envelope::safe_default(),
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
}
