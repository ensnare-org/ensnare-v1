// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
    prelude::*,
    traits::GeneratesEnvelope,
};
use ensnare_proc_macros::{Control, Params};
use midly::num::u7;

/// An effect that negates the input.
#[derive(Debug, Default)]
pub struct TestEffectNegatesInput {}
impl TransformsAudio for TestEffectNegatesInput {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        -input_sample
    }
}

#[derive(Debug)]
pub struct TestVoice {
    sample_rate: SampleRate,
    oscillator: Oscillator,
    envelope: Envelope,

    sample: StereoSample,

    note_on_key: u7,
    note_on_velocity: u7,
    steal_is_underway: bool,
}
impl IsStereoSampleVoice for TestVoice {}
impl IsVoice<StereoSample> for TestVoice {}
impl PlaysNotes for TestVoice {
    fn is_playing(&self) -> bool {
        !self.envelope.is_idle()
    }

    fn note_on(&mut self, key: u7, velocity: u7) {
        if self.is_playing() {
            self.steal_is_underway = true;
            self.note_on_key = key;
            self.note_on_velocity = velocity;
            self.envelope.trigger_shutdown();
        } else {
            self.set_frequency_hz(key.into());
            self.envelope.trigger_attack();
        }
    }

    fn aftertouch(&mut self, _velocity: u7) {
        todo!()
    }

    fn note_off(&mut self, _velocity: u7) {
        self.envelope.trigger_release();
    }
}
impl Generates<StereoSample> for TestVoice {
    fn value(&self) -> StereoSample {
        self.sample
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        for sample in values {
            self.tick(1);
            *sample = self.value();
        }
    }
}
impl Configurable for TestVoice {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.oscillator.update_sample_rate(sample_rate);
        self.envelope.update_sample_rate(sample_rate);
    }
}
impl Ticks for TestVoice {
    fn tick(&mut self, tick_count: usize) {
        for _ in 0..tick_count {
            if self.is_playing() {
                self.oscillator.tick(1);
                self.envelope.tick(1);
                if !self.is_playing() && self.steal_is_underway {
                    self.steal_is_underway = false;
                    self.note_on(self.note_on_key, self.note_on_velocity);
                }
            }
        }
        self.sample = if self.is_playing() {
            StereoSample::from(self.oscillator.value() * self.envelope.value())
        } else {
            StereoSample::SILENCE
        };
    }
}

impl TestVoice {
    pub(crate) fn new() -> Self {
        Self {
            sample_rate: Default::default(),
            oscillator: Oscillator::new_with(&OscillatorParams::default_with_waveform(
                Waveform::Sine,
            )),
            envelope: Envelope::new_with(&EnvelopeParams {
                attack: Normal::minimum(),
                decay: Normal::minimum(),
                sustain: Normal::maximum(),
                release: Normal::minimum(),
            }),
            sample: Default::default(),
            note_on_key: Default::default(),
            note_on_velocity: Default::default(),
            steal_is_underway: Default::default(),
        }
    }
    fn set_frequency_hz(&mut self, frequency_hz: FrequencyHz) {
        self.oscillator.set_frequency(frequency_hz);
    }

    pub fn debug_is_shutting_down(&self) -> bool {
        self.envelope.debug_is_shutting_down()
    }

    pub fn debug_oscillator_frequency(&self) -> FrequencyHz {
        self.oscillator.frequency()
    }
}
