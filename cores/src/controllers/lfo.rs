// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    generators::{Oscillator, OscillatorParams, Waveform},
    prelude::*,
};
use ensnare_proc_macros::{Control, Params};

/// Uses an internal LFO as a control source.
#[derive(Debug, Default, Control, Params)]
pub struct LfoController {
    #[control]
    #[params]
    pub waveform: Waveform,
    #[control]
    #[params]
    pub frequency: FrequencyHz,

    oscillator: Oscillator,

    is_performing: bool,

    time_range: TimeRange,

    last_frame: usize,
}
impl Serializable for LfoController {}
impl Configurable for LfoController {
    fn sample_rate(&self) -> SampleRate {
        self.oscillator.sample_rate()
    }
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.oscillator.update_sample_rate(sample_rate);
    }
}
impl Controls for LfoController {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.time_range = range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        let frames = self
            .time_range
            .0
            .start
            .as_frames(Tempo::from(120), self.oscillator.sample_rate());

        if frames != self.last_frame {
            let tick_count = if frames >= self.last_frame {
                // normal case; oscillator should advance the calculated number
                // of frames
                //
                // TODO: this is unlikely to be frame-accurate, because
                // Orchestrator is currently going from frames -> beats
                // (inaccurate), and then we're going from beats -> frames. We
                // could include frame count in update_time(), as discussed in
                // #132, which would mean we don't have to be smart at all about
                // it.
                frames - self.last_frame
            } else {
                self.last_frame = frames;
                0
            };
            self.last_frame += tick_count;
            self.oscillator.tick(tick_count);
        }
        control_events_fn(WorkEvent::Control(self.oscillator.value().into()));
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

    fn skip_to_start(&mut self) {
        // TODO: think how important it is for LFO oscillator to start at zero
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl HandlesMidi for LfoController {}
impl LfoController {
    pub fn new_with(params: &LfoControllerParams) -> Self {
        Self {
            oscillator: Oscillator::new_with(&OscillatorParams {
                waveform: params.waveform,
                frequency: params.frequency,
                ..Default::default()
            }),
            waveform: params.waveform(),
            frequency: params.frequency(),
            is_performing: false,

            time_range: Default::default(),
            last_frame: Default::default(),
        }
    }

    pub const fn frequency_range() -> std::ops::RangeInclusive<ParameterType> {
        0.0..=100.0
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
        self.oscillator.set_waveform(waveform);
    }

    pub fn frequency(&self) -> FrequencyHz {
        self.frequency
    }

    pub fn set_frequency(&mut self, frequency: FrequencyHz) {
        self.frequency = frequency;
        self.oscillator.set_frequency(frequency);
    }
}
