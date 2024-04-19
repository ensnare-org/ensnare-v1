// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{elements::OscillatorBuilder, prelude::*};
use delegate::delegate;
use derivative::Derivative;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Uses an internal LFO as a control source.
#[derive(Clone, Builder, Derivative, Debug, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default)]
#[derivative(Default)]
pub struct LfoControllerCore {
    #[control]
    #[derivative(Default(
        value = "OscillatorBuilder::default().waveform(Waveform::Sine).frequency(1.0.into()).build().unwrap()"
    ))]
    pub oscillator: Oscillator,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: LfoControllerCoreEphemerals,
}
#[derive(Clone, Debug, Default)]
pub struct LfoControllerCoreEphemerals {
    is_performing: bool,
    time_range: TimeRange,
    last_frame: usize,
    pub osc_buffer: GenerationBuffer<BipolarNormal>,
}
impl Serializable for LfoControllerCore {}
impl Configurable for LfoControllerCore {
    delegate! {
        to self.oscillator {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl Controls for LfoControllerCore {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.e.time_range = range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        let frames = self
            .e
            .time_range
            .0
            .start
            .as_frames(Tempo::from(120), self.oscillator.sample_rate());

        let mut last_value = BipolarNormal::default();
        if frames != self.e.last_frame {
            let tick_count = if frames >= self.e.last_frame {
                // normal case; oscillator should advance the calculated number
                // of frames
                //
                // TODO: this is unlikely to be frame-accurate, because
                // Orchestrator is currently going from frames -> beats
                // (inaccurate), and then we're going from beats -> frames. We
                // could include frame count in update_time(), as discussed in
                // #132, which would mean we don't have to be smart at all about
                // it.
                frames - self.e.last_frame
            } else {
                self.e.last_frame = frames;
                0
            };
            self.e.last_frame += tick_count;

            self.e.osc_buffer.set_buffer_size(tick_count);
            self.oscillator.generate(self.e.osc_buffer.buffer_mut());
            if tick_count != 0 {
                last_value = *self.e.osc_buffer.buffer().last().unwrap();
            }
        }
        control_events_fn(WorkEvent::Control(last_value.into()));
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.e.is_performing = true;
    }

    fn stop(&mut self) {
        self.e.is_performing = false;
    }

    fn skip_to_start(&mut self) {
        // TODO: think how important it is for LFO oscillator to start at zero
    }

    fn is_performing(&self) -> bool {
        self.e.is_performing
    }
}
impl HandlesMidi for LfoControllerCore {}
impl LfoControllerCore {
    pub fn notify_change_oscillator(&mut self) {}

    pub const fn frequency_range() -> core::ops::RangeInclusive<ParameterType> {
        0.0..=100.0
    }
}
