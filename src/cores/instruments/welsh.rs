// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use crate::{
    core::traits::{CanPrototype, GeneratesEnvelope},
    cores::effects::BiQuadFilterLowPass24db,
};
use core::fmt::Debug;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumCount, FromRepr};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    EnumCount,
    FromRepr,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum LfoRouting {
    #[default]
    None,
    Amplitude,
    Pitch,
    PulseWidth,
    FilterCutoff,
}

#[derive(Debug, Default)]
pub struct WelshVoice {
    //#[control]
    pub oscillator_1: Oscillator,
    //#[control]
    pub oscillator_2: Oscillator,
    //#[control]
    pub oscillator_2_sync: bool,
    //#[control]
    pub oscillator_mix: Normal, // 1.0 = entirely osc 0, 0.0 = entirely osc 1.
    //#[control]
    pub amp_envelope: Envelope,
    //#[control]
    pub dca: Dca,

    //#[control]
    pub lfo: Oscillator,
    pub lfo_routing: LfoRouting,
    //#[control]
    pub lfo_depth: Normal,

    //#[control]
    pub filter: BiQuadFilterLowPass24db,
    //#[control]
    pub filter_cutoff_start: Normal,
    //#[control]
    pub filter_cutoff_end: Normal,
    //#[control]
    pub filter_envelope: Envelope,

    note_on_key: u7,
    note_on_velocity: u7,
    steal_is_underway: bool,

    sample: StereoSample,
    ticks: usize,
}
impl IsStereoSampleVoice for WelshVoice {}
impl IsVoice<StereoSample> for WelshVoice {}
impl PlaysNotes for WelshVoice {
    fn is_playing(&self) -> bool {
        !self.amp_envelope.is_idle()
    }
    fn note_on(&mut self, key: u7, velocity: u7) {
        if self.is_playing() {
            self.steal_is_underway = true;
            self.note_on_key = key;
            self.note_on_velocity = velocity;
            self.amp_envelope.trigger_shutdown();
        } else {
            self.amp_envelope.trigger_attack();
            self.filter_envelope.trigger_attack();
            self.set_frequency_hz(MidiNote::from_repr(key.as_int() as usize).unwrap().into());
        }
    }
    fn aftertouch(&mut self, _velocity: u7) {
        // TODO: do something
    }
    fn note_off(&mut self, _velocity: u7) {
        self.amp_envelope.trigger_release();
        self.filter_envelope.trigger_release();
    }
}
impl Generates<StereoSample> for WelshVoice {
    fn value(&self) -> StereoSample {
        self.sample
    }
    fn generate(&mut self, _samples: &mut [StereoSample]) {
        todo!()
    }
}
impl Serializable for WelshVoice {}
impl Configurable for WelshVoice {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.ticks = 0;
        self.lfo.update_sample_rate(sample_rate);
        self.amp_envelope.update_sample_rate(sample_rate);
        self.filter_envelope.update_sample_rate(sample_rate);
        self.filter.update_sample_rate(sample_rate);
        self.oscillator_1.update_sample_rate(sample_rate);
        self.oscillator_2.update_sample_rate(sample_rate);
    }
}
impl Ticks for WelshVoice {
    fn tick(&mut self, tick_count: usize) {
        for _ in 0..tick_count {
            self.ticks += 1;
            // It's important for the envelope tick() methods to be called after
            // their handle_note_* methods are called, but before we check whether
            // amp_envelope.is_idle(), because the tick() methods are what determine
            // the current idle state.
            //
            // TODO: this seems like an implementation detail that maybe should be
            // hidden from the caller.
            let (amp_env_amplitude, filter_env_amplitude) = self.tick_envelopes();

            // TODO: various parts of this loop can be precalculated.

            self.sample = if self.is_playing() {
                // TODO: ideally, these entities would get a tick() on every
                // voice tick(), but they are surprisingly expensive. So we will
                // skip calling them unless we're going to look at their output.
                // This means that they won't get a time slice as often as the
                // voice will. If this becomes a problem, we can add something
                // like an empty_tick() method to the Ticks trait that lets
                // entities stay in sync, but skipping any real work that would
                // cost time.
                if !matches!(self.lfo_routing, LfoRouting::None) {
                    self.lfo.tick(1);
                }
                self.oscillator_1.tick(1);
                self.oscillator_2.tick(1);

                // LFO
                let lfo = self.lfo.value();
                if matches!(self.lfo_routing, LfoRouting::Pitch) {
                    let lfo_for_pitch = lfo * self.lfo_depth;
                    self.oscillator_1.set_frequency_modulation(lfo_for_pitch);
                    self.oscillator_2.set_frequency_modulation(lfo_for_pitch);
                }

                // Oscillators
                let osc_sum = {
                    if self.oscillator_2_sync && self.oscillator_1.should_sync() {
                        self.oscillator_2.sync();
                    }
                    self.oscillator_1.value() * self.oscillator_mix
                        + self.oscillator_2.value() * (Normal::maximum() - self.oscillator_mix)
                };

                // Filters
                //
                // https://aempass.blogspot.com/2014/09/analog-and-welshs-synthesizer-cookbook.html
                if self.filter_cutoff_end != Normal::zero() {
                    let new_cutoff_percentage = self.filter_cutoff_start
                        + (1.0 - self.filter_cutoff_start)
                            * self.filter_cutoff_end
                            * filter_env_amplitude;
                    self.filter.set_cutoff(new_cutoff_percentage.into());
                } else if matches!(self.lfo_routing, LfoRouting::FilterCutoff) {
                    let lfo_for_cutoff = lfo * self.lfo_depth;
                    self.filter
                        .set_cutoff((self.filter_cutoff_start * (lfo_for_cutoff.0 + 1.0)).into());
                }
                let filtered_mix = self.filter.transform_channel(0, Sample::from(osc_sum)).0;

                // LFO amplitude modulation
                let lfo_for_amplitude =
                    Normal::from(if matches!(self.lfo_routing, LfoRouting::Amplitude) {
                        lfo * self.lfo_depth
                    } else {
                        BipolarNormal::zero()
                    });

                // Final
                self.dca.transform_audio_to_stereo(Sample(
                    filtered_mix * amp_env_amplitude.0 * lfo_for_amplitude.0,
                ))
            } else {
                StereoSample::SILENCE
            };
        }
    }
}
impl WelshVoice {
    pub fn new_with(
        oscillator_1: &Oscillator,
        oscillator_2: &Oscillator,
        oscillator_2_sync: bool,
        oscillator_mix: Normal,
        amp_envelope: &Envelope,
        dca: &Dca,
        lfo: &Oscillator,
        lfo_routing: LfoRouting,
        lfo_depth: Normal,
        filter: &BiQuadFilterLowPass24db,
        filter_cutoff_start: Normal,
        filter_cutoff_end: Normal,
        filter_envelope: &Envelope,
    ) -> Self {
        Self {
            oscillator_1: oscillator_1.make_another(),
            oscillator_2: oscillator_2.make_another(),
            oscillator_2_sync,
            oscillator_mix,
            amp_envelope: amp_envelope.make_another(),
            dca: dca.make_another(),
            lfo: lfo.make_another(),
            lfo_routing,
            lfo_depth,
            filter: filter.make_another(),
            filter_cutoff_start,
            filter_cutoff_end,
            filter_envelope: filter_envelope.make_another(),
            note_on_key: Default::default(),
            note_on_velocity: Default::default(),
            steal_is_underway: Default::default(),
            sample: Default::default(),
            ticks: Default::default(),
        }
    }

    fn tick_envelopes(&mut self) -> (Normal, Normal) {
        if self.is_playing() {
            self.amp_envelope.tick(1);
            self.filter_envelope.tick(1);
            if self.is_playing() {
                return (self.amp_envelope.value(), self.filter_envelope.value());
            }

            if self.steal_is_underway {
                self.steal_is_underway = false;
                self.note_on(self.note_on_key, self.note_on_velocity);
            }
        }
        (Normal::zero(), Normal::zero())
    }

    fn set_frequency_hz(&mut self, frequency_hz: FrequencyHz) {
        // It's safe to set the frequency on a fixed-frequency oscillator; the
        // fixed frequency is stored separately and takes precedence.
        self.oscillator_1.set_frequency(frequency_hz);
        self.oscillator_2.set_frequency(frequency_hz);
    }

    pub fn set_lfo_depth(&mut self, lfo_depth: Normal) {
        self.lfo_depth = lfo_depth;
    }

    pub fn set_filter_cutoff_start(&mut self, filter_cutoff_start: Normal) {
        self.filter_cutoff_start = filter_cutoff_start;
    }

    pub fn set_filter_cutoff_end(&mut self, filter_cutoff_end: Normal) {
        self.filter_cutoff_end = filter_cutoff_end;
    }

    pub fn set_oscillator_2_sync(&mut self, oscillator_2_sync: bool) {
        self.oscillator_2_sync = oscillator_2_sync;
    }

    pub fn set_oscillator_mix(&mut self, oscillator_mix: Normal) {
        self.oscillator_mix = oscillator_mix;
    }

    pub fn amp_envelope_mut(&mut self) -> &mut Envelope {
        &mut self.amp_envelope
    }

    pub fn filter_mut(&mut self) -> &mut BiQuadFilterLowPass24db {
        &mut self.filter
    }

    pub fn oscillator_2_sync(&self) -> bool {
        self.oscillator_2_sync
    }

    pub fn oscillator_mix(&self) -> Normal {
        self.oscillator_mix
    }

    pub fn lfo_routing(&self) -> LfoRouting {
        self.lfo_routing
    }

    pub fn lfo_depth(&self) -> Normal {
        self.lfo_depth
    }

    pub fn filter_cutoff_start(&self) -> Normal {
        self.filter_cutoff_start
    }

    pub fn filter_cutoff_end(&self) -> Normal {
        self.filter_cutoff_end
    }

    pub fn clone_fresh(&self) -> Self {
        todo!(" fill in the important fields... note that this likely matches the serde fields.");
        // let r = Self::default();
        // r
    }
}

/// A synthesizer inspired by Fred Welsh's [Welsh's Synthesizer
/// Cookbook](https://www.amazon.com/dp/B000ERHA4S/).
#[derive(Debug, Control, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WelshSynth {
    #[control]
    pub oscillator_1: Oscillator,
    #[control]
    pub oscillator_2: Oscillator,
    #[control]
    pub oscillator_2_sync: bool,
    #[control]
    pub oscillator_mix: Normal, // 1.0 = entirely osc 0, 0.0 = entirely osc 1.
    #[control]
    pub amp_envelope: Envelope,
    #[control]
    pub dca: Dca,

    #[control]
    pub lfo: Oscillator,
    pub lfo_routing: LfoRouting,
    #[control]
    pub lfo_depth: Normal,

    #[control]
    pub filter: BiQuadFilterLowPass24db,
    #[control]
    pub filter_cutoff_start: Normal,
    #[control]
    pub filter_cutoff_end: Normal,
    #[control]
    pub filter_envelope: Envelope,

    #[serde(skip)]
    pub inner: Synthesizer<WelshVoice>,
}
impl WelshSynth {
    const VOICE_CAPACITY: usize = 8;

    pub fn new_with(
        oscillator_1: Oscillator,
        oscillator_2: Oscillator,
        oscillator_2_sync: bool,
        oscillator_mix: Normal,
        amp_envelope: Envelope,
        dca: Dca,
        lfo: Oscillator,
        lfo_routing: LfoRouting,
        lfo_depth: Normal,
        filter: BiQuadFilterLowPass24db,
        filter_cutoff_start: Normal,
        filter_cutoff_end: Normal,
        filter_envelope: Envelope,
    ) -> Self {
        let voice_store =
            StealingVoiceStore::<WelshVoice>::new_with_voice(Self::VOICE_CAPACITY, || {
                WelshVoice::new_with(
                    &oscillator_1,
                    &oscillator_2,
                    oscillator_2_sync,
                    oscillator_mix,
                    &amp_envelope,
                    &dca,
                    &lfo,
                    lfo_routing,
                    lfo_depth,
                    &filter,
                    filter_cutoff_start,
                    filter_cutoff_end,
                    &filter_envelope,
                )
            });
        Self {
            inner: Synthesizer::<WelshVoice>::new_with(Box::new(voice_store)),
            oscillator_1,
            oscillator_2,
            oscillator_2_sync,
            oscillator_mix,
            amp_envelope,
            dca,
            lfo,
            lfo_routing,
            lfo_depth,
            filter,
            filter_cutoff_start,
            filter_cutoff_end,
            filter_envelope,
        }
    }

    fn new_voice_store(&self) -> StealingVoiceStore<WelshVoice> {
        StealingVoiceStore::<WelshVoice>::new_with_voice(Self::VOICE_CAPACITY, || {
            WelshVoice::new_with(
                &self.oscillator_1,
                &self.oscillator_2,
                self.oscillator_2_sync,
                self.oscillator_mix,
                &self.amp_envelope,
                &self.dca,
                &self.lfo,
                self.lfo_routing,
                self.lfo_depth,
                &self.filter,
                self.filter_cutoff_start,
                self.filter_cutoff_end,
                &self.filter_envelope,
            )
        })
    }
}
impl Generates<StereoSample> for WelshSynth {
    fn value(&self) -> StereoSample {
        self.inner.value()
    }

    fn generate(&mut self, values: &mut [StereoSample]) {
        self.inner.generate(values);
    }
}
impl Serializable for WelshSynth {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.inner = Synthesizer::<WelshVoice>::new_with(Box::new(self.new_voice_store()));
    }
}
impl Configurable for WelshSynth {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.inner.update_sample_rate(sample_rate);
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }
}
impl Ticks for WelshSynth {
    fn tick(&mut self, tick_count: usize) {
        self.inner.tick(tick_count);
    }
}
impl HandlesMidi for WelshSynth {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
        match message {
            #[allow(unused_variables)]
            MidiMessage::ProgramChange { program } => {
                todo!()
                // if let Some(program) = GeneralMidiProgram::from_u8(program.as_int()) {
                //     if let Ok(_preset) = WelshSynth::general_midi_preset(&program) {
                //         //  self.preset = preset;
                //     } else {
                //         println!("unrecognized patch from MIDI program change: {}", &program);
                //     }
                // }
                // None
            }
            _ => self
                .inner
                .handle_midi_message(channel, message, midi_messages_fn),
        }
    }
}
impl WelshSynth {
    pub fn preset_name(&self) -> &str {
        "none"
        //        self.preset.name.as_str()
    }

    pub fn notify_change_oscillator_1(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.oscillator_1.update_from_prototype(&self.oscillator_1);
        });
    }
    pub fn notify_change_oscillator_2(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.oscillator_2.update_from_prototype(&self.oscillator_2);
        });
    }
    pub fn notify_change_amp_envelope(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.amp_envelope.update_from_prototype(&self.amp_envelope);
        });
    }
    pub fn notify_change_dca(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.dca.update_from_prototype(&self.dca);
        });
    }
    pub fn notify_change_lfo(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.lfo.update_from_prototype(&self.lfo);
        });
    }
    pub fn notify_change_filter(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.filter.update_from_prototype(&self.filter);
        });
    }
    pub fn notify_change_filter_envelope(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.filter_envelope
                .update_from_prototype(&self.filter_envelope);
        });
    }

    pub fn set_oscillator_2_sync(&mut self, oscillator_2_sync: bool) {
        self.oscillator_2_sync = oscillator_2_sync;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_oscillator_2_sync(self.oscillator_2_sync));
    }

    pub fn set_oscillator_mix(&mut self, oscillator_mix: Normal) {
        self.oscillator_mix = oscillator_mix;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_oscillator_mix(self.oscillator_mix));
    }

    pub fn set_lfo_depth(&mut self, lfo_depth: Normal) {
        self.lfo_depth = lfo_depth;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_lfo_depth(self.lfo_depth));
    }

    pub fn set_filter_cutoff_start(&mut self, filter_cutoff_start: Normal) {
        self.filter_cutoff_start = filter_cutoff_start;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_filter_cutoff_start(self.filter_cutoff_start));
    }

    pub fn set_filter_cutoff_end(&mut self, filter_cutoff_end: Normal) {
        self.filter_cutoff_end = filter_cutoff_end;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_filter_cutoff_end(self.filter_cutoff_end));
    }
}

#[cfg(test)]
mod tests {
    // use convert_case::{Case, Casing};
    // use crate::util::tests::TestOnlyPaths;

    // // TODO dedup
    // pub fn canonicalize_output_filename_and_path(filename: &str) -> String {
    //     let mut path = TestOnlyPaths::data_path();
    //     let snake_filename = format!("{}.wav", filename.to_case(Case::Snake)).to_string();
    //     path.push(snake_filename);
    //     if let Some(path) = path.to_str() {
    //         path.to_string()
    //     } else {
    //         panic!("trouble creating output path")
    //     }
    // }

    #[cfg(obsolete)]
    // TODO: refactor out to common test utilities
    #[allow(dead_code)]
    fn write_voice(voice: &mut WelshVoice, duration: f64, basename: &str) {
        let mut clock = Clock::new_with(&ClockParams {
            bpm: DEFAULT_BPM,
            midi_ticks_per_second: DEFAULT_MIDI_TICKS_PER_SECOND,
            time_signature: TimeSignatureParams { top: 4, bottom: 4 },
        });

        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: clock.sample_rate().value() as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        const AMPLITUDE: SampleType = i16::MAX as SampleType;
        let mut writer =
            hound::WavWriter::create(canonicalize_output_filename_and_path(basename), spec)
                .unwrap();

        let mut last_recognized_time_point = -1.;
        let time_note_off = duration / 2.0;
        while clock.seconds() < duration {
            if clock.seconds() >= 0.0 && last_recognized_time_point < 0.0 {
                last_recognized_time_point = clock.seconds();
                voice.note_on(60, 127);
                voice.tick_envelopes();
            } else if clock.seconds() >= time_note_off && last_recognized_time_point < time_note_off
            {
                last_recognized_time_point = clock.seconds();
                voice.note_off(127);
                voice.tick_envelopes();
            }

            voice.tick(1);
            let sample = voice.value();
            let _ = writer.write_sample((sample.0 .0 * AMPLITUDE) as i16);
            let _ = writer.write_sample((sample.1 .0 * AMPLITUDE) as i16);
            clock.tick(1);
        }
    }
}
