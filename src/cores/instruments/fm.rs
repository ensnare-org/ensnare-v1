// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct FmVoice {
    sample: StereoSample,
    carrier: Oscillator,
    carrier_envelope: Envelope,
    modulator: Oscillator,
    modulator_envelope: Envelope,

    /// depth 0.0 means no modulation; 1.0 means maximum
    depth: Normal,

    /// modulator frequency is based on carrier frequency and modulator_ratio
    ratio: Ratio,

    /// Ranges from 0.0 to very high.
    ///
    /// - 0.0: no effect
    /// - 0.1: change is visible on scope but not audible
    /// - 1.0: audible change
    /// - 10.0: dramatic change,
    /// - 100.0: extreme.
    beta: ParameterType,

    dca: Dca,

    note_on_key: u7,
    note_on_velocity: u7,
    steal_is_underway: bool,

    sample_rate: SampleRate,
}
impl IsStereoSampleVoice for FmVoice {}
impl IsVoice<StereoSample> for FmVoice {}
impl PlaysNotes for FmVoice {
    fn is_playing(&self) -> bool {
        !self.carrier_envelope.is_idle()
    }

    fn note_on(&mut self, key: u7, velocity: u7) {
        if self.is_playing() {
            self.steal_is_underway = true;
            self.note_on_key = key;
            self.note_on_velocity = velocity;
            self.carrier_envelope.trigger_shutdown();
            self.modulator_envelope.trigger_shutdown();
        } else {
            self.set_frequency_hz(MidiNote::from_repr(key.as_int() as usize).unwrap().into());
            self.carrier_envelope.trigger_attack();
            self.modulator_envelope.trigger_attack();
        }
    }

    fn aftertouch(&mut self, _velocity: u7) {
        todo!()
    }

    fn note_off(&mut self, _velocity: u7) {
        self.carrier_envelope.trigger_release();
        self.modulator_envelope.trigger_release();
    }
}
impl Generates<StereoSample> for FmVoice {
    fn generate_next(&mut self) -> StereoSample {
        let mut r = BipolarNormal::from(0.0);
        if self.is_playing() {
            let modulator_magnitude = self.modulator.generate_next()
                * self.modulator_envelope.generate_next()
                * self.depth;
            self.carrier
                .set_linear_frequency_modulation(modulator_magnitude.0 * self.beta);
            r = self.carrier.generate_next() * self.carrier_envelope.generate_next();
            if !self.is_playing() && self.steal_is_underway {
                self.steal_is_underway = false;
                self.note_on(self.note_on_key, self.note_on_velocity);
            }
        }
        if self.is_playing() {
            self.dca.transform_audio_to_stereo(Sample::from(r))
        } else {
            StereoSample::SILENCE
        }
    }
}
impl Serializable for FmVoice {}
impl Configurable for FmVoice {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.carrier_envelope.update_sample_rate(sample_rate);
        self.modulator_envelope.update_sample_rate(sample_rate);
        self.carrier.update_sample_rate(sample_rate);
        self.modulator.update_sample_rate(sample_rate);
    }
}
impl FmVoice {
    pub fn new_with(
        carrier: &Oscillator,
        carrier_envelope: &Envelope,
        modulator: &Oscillator,
        modulator_envelope: &Envelope,
        depth: Normal,
        ratio: Ratio,
        beta: ParameterType,
        dca: &Dca,
    ) -> Self {
        Self {
            carrier: carrier.make_another(),
            carrier_envelope: carrier_envelope.make_another(),
            modulator: modulator.make_another(),
            modulator_envelope: modulator_envelope.make_another(),
            depth,
            ratio,
            beta,
            dca: dca.make_another(),
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn modulator_frequency(&self) -> FrequencyHz {
        self.modulator.frequency()
    }

    #[allow(dead_code)]
    pub fn set_modulator_frequency(&mut self, value: FrequencyHz) {
        self.modulator.set_frequency(value);
    }

    fn set_frequency_hz(&mut self, frequency_hz: FrequencyHz) {
        self.carrier.set_frequency(frequency_hz);
        self.modulator.set_frequency(frequency_hz * self.ratio);
    }

    pub fn depth(&self) -> Normal {
        self.depth
    }

    pub fn ratio(&self) -> Ratio {
        self.ratio
    }

    pub fn beta(&self) -> f64 {
        self.beta
    }

    pub fn set_depth(&mut self, depth: Normal) {
        self.depth = depth;
    }

    pub fn set_ratio(&mut self, ratio: Ratio) {
        self.ratio = ratio;
    }

    pub fn set_beta(&mut self, beta: ParameterType) {
        self.beta = beta;
    }

    // TODO: we'll have to be smarter about subbing in a new envelope, possibly
    // while the voice is playing.
    pub fn set_carrier_envelope(&mut self, envelope: Envelope) {
        self.carrier_envelope = envelope;
    }

    pub fn set_modulator_envelope(&mut self, envelope: Envelope) {
        self.modulator_envelope = envelope;
    }

    fn set_gain(&mut self, gain: Normal) {
        self.dca.set_gain(gain);
    }

    fn set_pan(&mut self, pan: BipolarNormal) {
        self.dca.set_pan(pan);
    }
}

#[derive(Debug, Default, Builder, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct FmSynthCore {
    #[control]
    pub carrier: Oscillator,

    #[control]
    pub carrier_envelope: Envelope,

    #[control]
    pub modulator: Oscillator,

    #[control]
    pub modulator_envelope: Envelope,

    #[control]
    depth: Normal,

    #[control]
    ratio: Ratio,

    #[control]
    beta: ParameterType,

    #[control]
    pub dca: Dca,

    #[serde(skip)]
    #[builder(setter(skip))]
    pub inner: Synthesizer<FmVoice>,
}
impl FmSynthCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<FmSynthCore, FmSynthCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl Generates<StereoSample> for FmSynthCore {
    fn generate(&mut self, values: &mut [StereoSample]) {
        self.inner.generate(values);
    }

    fn generate_next(&mut self) -> StereoSample {
        self.inner.generate_next()
    }
}
impl Serializable for FmSynthCore {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.inner = Synthesizer::<FmVoice>::new_with(Box::new(Self::make_voice_store(
            &self.carrier,
            &self.carrier_envelope,
            &self.modulator,
            &self.modulator_envelope,
            self.depth,
            self.ratio,
            self.beta,
            &self.dca,
        )))
    }
}
impl Configurable for FmSynthCore {
    delegate! {
        to self.inner {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl HandlesMidi for FmSynthCore {
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
impl FmSynthCore {
    fn make_voice_store(
        carrier_oscillator: &Oscillator,
        carrier_envelope: &Envelope,
        modulator_oscillator: &Oscillator,
        modulator_envelope: &Envelope,
        depth: Normal,
        ratio: Ratio,
        beta: f64,
        dca: &Dca,
    ) -> StealingVoiceStore<FmVoice> {
        const VOICE_CAPACITY: usize = 8;
        StealingVoiceStore::<FmVoice>::new_with_voice(VOICE_CAPACITY, || {
            FmVoice::new_with(
                carrier_oscillator,
                carrier_envelope,
                modulator_oscillator,
                modulator_envelope,
                depth,
                ratio,
                beta,
                dca,
            )
        })
    }

    pub fn set_depth(&mut self, depth: Normal) {
        self.depth = depth;
        self.inner.voices_mut().for_each(|v| v.set_depth(depth));
    }

    pub fn set_ratio(&mut self, ratio: Ratio) {
        self.ratio = ratio;
        self.inner.voices_mut().for_each(|v| v.set_ratio(ratio));
    }

    pub fn set_beta(&mut self, beta: ParameterType) {
        self.beta = beta;
        self.inner.voices_mut().for_each(|v| v.set_beta(beta));
    }

    pub fn depth(&self) -> Normal {
        self.depth
    }

    pub fn ratio(&self) -> Ratio {
        self.ratio
    }

    pub fn beta(&self) -> f64 {
        self.beta
    }

    pub fn notify_change_carrier(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.carrier.update_from_prototype(&self.carrier);
        });
    }

    pub fn notify_change_carrier_envelope(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.carrier_envelope
                .update_from_prototype(&self.carrier_envelope);
        });
    }

    pub fn notify_change_modulator(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.modulator.update_from_prototype(&self.modulator);
        });
    }

    pub fn notify_change_modulator_envelope(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.modulator_envelope
                .update_from_prototype(&self.modulator_envelope);
        });
    }

    pub fn set_gain(&mut self, gain: Normal) {
        self.dca.set_gain(gain);
        self.inner.voices_mut().for_each(|v| v.set_gain(gain));
    }

    pub fn set_pan(&mut self, pan: BipolarNormal) {
        self.dca.set_pan(pan);
        self.inner.voices_mut().for_each(|v| v.set_pan(pan));
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
